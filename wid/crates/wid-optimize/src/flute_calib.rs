//! Joint flute calibration (2D BOBYQA).
//!
//! Matches Java `FluteCalibrationObjectiveFunction` — simultaneously
//! optimizes airstream length and beta to minimize the FminmaxEvaluator
//! error norm. Analogous to `whistle_calib.rs` but uses airstream length
//! instead of window height.
//!
//! Geometry vector: `[airstream_length, beta]`.
//!
//! Used by: Flute study model.

use bobyqa_impl::bobyqa_minimize;
use wid_compile::{compile, get_airstream_length, get_beta, set_airstream_length, set_beta};
use wid_eval::evaluators::calculate_fminmax_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{FluteCalibrationResult, calc_norm, fingering_weights};

/// Calibrate airstream length and beta jointly to minimize combined fmin+fmax error.
///
/// The instrument is modified in place with the optimal values.
pub fn calibrate_flute(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    al_bounds: (f64, f64),
    beta_bounds: (f64, f64),
    calc_params: &CalculatorParams,
) -> FluteCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_al = get_airstream_length(instrument).unwrap_or(0.004);
    let initial_beta = get_beta(instrument).unwrap_or(0.35);
    let initial_norm = evaluate_fminmax_norm(
        instrument,
        &tuning.fingerings,
        &weights,
        params,
        calc_params,
    );

    let initial_point = [
        initial_al.clamp(al_bounds.0, al_bounds.1),
        initial_beta.clamp(beta_bounds.0, beta_bounds.1),
    ];
    let lower_bounds = [al_bounds.0, beta_bounds.0];
    let upper_bounds = [al_bounds.1, beta_bounds.1];

    let n_dims = 2;
    let n_interp = 2 * n_dims + 1;
    let (initial_trust, stopping_trust) = crate::compute_trust_radius(&lower_bounds, &upper_bounds);
    let max_eval = crate::max_evaluations(n_dims);

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = bobyqa_minimize(
        &mut |point: &[f64]| {
            set_airstream_length(&mut work_inst, point[0]);
            set_beta(&mut work_inst, point[1]);
            evaluate_fminmax_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_point,
        &lower_bounds,
        &upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_eval,
    );

    let (final_al, final_beta, final_norm) = match result {
        Some(opt) => (opt.point[0], opt.point[1], opt.value),
        None => (initial_al, initial_beta, initial_norm),
    };

    set_airstream_length(instrument, final_al);
    set_beta(instrument, final_beta);

    FluteCalibrationResult {
        initial_airstream_length: Some(initial_al),
        final_airstream_length: Some(final_al),
        initial_beta: Some(initial_beta),
        final_beta: Some(final_beta),
        initial_norm,
        final_norm,
    }
}

fn evaluate_fminmax_norm(
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
    let errors = calculate_fminmax_error_vector(&compiled, fingerings, params, calc_params);
    calc_norm(&errors, weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wid_physics::TemperatureType;
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    const FLUTE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/instruments/SamplePVC-Flute.xml");
    const FLUTE_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/tunings/D4-Equal.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    fn default_bounds() -> ((f64, f64), (f64, f64)) {
        let al_bounds = (
            crate::airstream_length::DEFAULT_AL_LOWER,
            crate::airstream_length::DEFAULT_AL_UPPER,
        );
        let beta_bounds = (
            crate::beta::DEFAULT_BETA_LOWER,
            crate::beta::DEFAULT_BETA_UPPER,
        );
        (al_bounds, beta_bounds)
    }

    // Golden: FL-CAL/calib_joint.json
    const GOLDEN_FINAL_BETA: f64 = 0.2;
    const GOLDEN_INITIAL_NORM: f64 = 2649.612447927295;
    const GOLDEN_FINAL_NORM: f64 = 1313.405490636519;

    #[test]
    fn initial_norm_matches_golden() {
        let inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = crate::fingering_weights(&tuning.fingerings);
        let norm = evaluate_fminmax_norm(
            &inst,
            &tuning.fingerings,
            &weights,
            &params,
            &CalculatorParams::FLUTE,
        );
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn joint_calibration_matches_golden() {
        let mut inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let (al_bounds, beta_bounds) = default_bounds();

        let result = calibrate_flute(
            &mut inst,
            &tuning,
            &params,
            al_bounds,
            beta_bounds,
            &CalculatorParams::FLUTE,
        );

        // Norm should reduce significantly
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
            result.final_norm
        );

        // Beta should move toward lower bound (golden: 0.2)
        let final_beta = result.final_beta.unwrap();
        assert!(
            (final_beta - GOLDEN_FINAL_BETA).abs() / GOLDEN_FINAL_BETA < 0.10,
            "final beta: expected ~{GOLDEN_FINAL_BETA}, got {final_beta}"
        );
    }

    #[test]
    fn fife_joint_calibration_does_not_worsen_norm() {
        let fife_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/instruments/fife.xml"
        );
        let fife_tuning_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/tunings/fife-tuning.xml"
        );
        let mut inst = parse_instrument_xml(fife_xml).unwrap();
        let tuning = parse_tuning_xml(fife_tuning_xml).unwrap();
        let params = default_params();
        let (al_bounds, beta_bounds) = default_bounds();

        let result = calibrate_flute(
            &mut inst,
            &tuning,
            &params,
            al_bounds,
            beta_bounds,
            &CalculatorParams::FLUTE,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "fife joint calibration should not worsen norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm,
        );
    }
}
