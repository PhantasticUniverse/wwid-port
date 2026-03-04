//! Joint reed calibration (2D BOBYQA).
//!
//! Matches Java `ReedCalibratorObjectiveFunction` — simultaneously
//! optimizes alpha and beta to minimize the CentDeviationEvaluator
//! error norm. Uses `calculate_error_vector` (NOT fminmax).
//!
//! Geometry vector: `[alpha, beta]`.
//!
//! Default bounds: `[0.0, 10.0]` for both alpha and beta.
//!
//! Used by: Reed study model.

use bobyqa_impl::bobyqa_minimize;
use wid_compile::{compile, get_alpha, get_beta, set_alpha, set_beta};
use wid_eval::calculate_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{ReedCalibrationResult, calc_norm, fingering_weights};

/// Default alpha bounds (matching Java ReedStudyModel).
pub const DEFAULT_ALPHA_LOWER: f64 = 0.0;
pub const DEFAULT_ALPHA_UPPER: f64 = 10.0;

/// Default beta bounds (matching Java ReedStudyModel).
pub const DEFAULT_BETA_LOWER: f64 = 0.0;
pub const DEFAULT_BETA_UPPER: f64 = 10.0;

/// Calibrate alpha and beta jointly to minimize cent deviation error.
///
/// The instrument is modified in place with the optimal values.
pub fn calibrate_reed(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    alpha_bounds: (f64, f64),
    beta_bounds: (f64, f64),
    calc_params: &CalculatorParams,
) -> ReedCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_alpha = get_alpha(instrument).unwrap_or(0.0);
    let initial_beta = get_beta(instrument).unwrap_or(0.0);
    let initial_norm = evaluate_cent_norm(
        instrument,
        &tuning.fingerings,
        &weights,
        params,
        calc_params,
    );

    let initial_point = [
        initial_alpha.clamp(alpha_bounds.0, alpha_bounds.1),
        initial_beta.clamp(beta_bounds.0, beta_bounds.1),
    ];
    let lower_bounds = [alpha_bounds.0, beta_bounds.0];
    let upper_bounds = [alpha_bounds.1, beta_bounds.1];

    let n_dims = 2;
    let n_interp = 2 * n_dims + 1;
    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = 20000 + (n_dims - 1) * 5000;

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = bobyqa_minimize(
        &mut |point: &[f64]| {
            set_alpha(&mut work_inst, point[0]);
            set_beta(&mut work_inst, point[1]);
            evaluate_cent_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_point,
        &lower_bounds,
        &upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_eval,
    );

    let (final_alpha, final_beta, final_norm) = match result {
        Some(opt) => (opt.point[0], opt.point[1], opt.value),
        None => (initial_alpha, initial_beta, initial_norm),
    };

    set_alpha(instrument, final_alpha);
    set_beta(instrument, final_beta);

    ReedCalibrationResult {
        initial_alpha: Some(initial_alpha),
        final_alpha: Some(final_alpha),
        initial_beta: Some(initial_beta),
        final_beta: Some(final_beta),
        initial_norm,
        final_norm,
    }
}

fn evaluate_cent_norm(
    instrument: &InstrumentRaw,
    fingerings: &[wid_types::Fingering],
    weights: &[i32],
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> f64 {
    let compiled = match compile(instrument) {
        Ok(c) => c,
        Err(_) => return f64::MAX,
    };
    let errors = calculate_error_vector(&compiled, fingerings, params, calc_params);
    calc_norm(&errors, weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wid_physics::TemperatureType;
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    const CHANTER_XML: &str =
        include_str!("../../../../oracle/v2.6.0/ReedStudy/instruments/SampleChanter.xml");
    const CHANTER_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/ReedStudy/tunings/A3-ClosedFingering.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // Golden: RD-CAL/calib_joint.json
    const GOLDEN_INITIAL_ALPHA: f64 = 1.8;
    const GOLDEN_FINAL_ALPHA: f64 = 1.7774234423361617;
    const GOLDEN_INITIAL_BETA: f64 = 0.09;
    const GOLDEN_FINAL_BETA: f64 = 0.09466823953486997;
    const GOLDEN_INITIAL_NORM: f64 = 54.760049498277766;
    const GOLDEN_FINAL_NORM: f64 = 26.49485923552761;

    #[test]
    fn initial_norm_matches_golden() {
        let inst = parse_instrument_xml(CHANTER_XML).unwrap();
        let tuning = parse_tuning_xml(CHANTER_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_cent_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::REED,
        );
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn reed_calibration_matches_golden() {
        let mut inst = parse_instrument_xml(CHANTER_XML).unwrap();
        let tuning = parse_tuning_xml(CHANTER_TUNING_XML).unwrap();
        let params = default_params();

        let alpha_bounds = (DEFAULT_ALPHA_LOWER, DEFAULT_ALPHA_UPPER);
        let beta_bounds = (DEFAULT_BETA_LOWER, DEFAULT_BETA_UPPER);

        let result = calibrate_reed(
            &mut inst,
            &tuning,
            &params,
            alpha_bounds,
            beta_bounds,
            &CalculatorParams::REED,
        );

        // Norm should reduce
        assert!(
            result.final_norm < result.initial_norm,
            "optimization should reduce norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm,
        );

        // Final norm within 10% of golden
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 1.1,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm,
        );

        // Alpha within 10% of golden
        let final_alpha = result.final_alpha.unwrap();
        assert!(
            (final_alpha - GOLDEN_FINAL_ALPHA).abs() / GOLDEN_FINAL_ALPHA < 0.10,
            "final alpha: expected ~{GOLDEN_FINAL_ALPHA}, got {final_alpha}"
        );

        // Beta within 10% of golden
        let final_beta = result.final_beta.unwrap();
        assert!(
            (final_beta - GOLDEN_FINAL_BETA).abs() / GOLDEN_FINAL_BETA < 0.10,
            "final beta: expected ~{GOLDEN_FINAL_BETA}, got {final_beta}"
        );

        // Initial values should match
        assert!(
            (result.initial_alpha.unwrap() - GOLDEN_INITIAL_ALPHA).abs() < 1e-10,
            "initial alpha should match golden"
        );
        assert!(
            (result.initial_beta.unwrap() - GOLDEN_INITIAL_BETA).abs() < 1e-10,
            "initial beta should match golden"
        );
    }
}
