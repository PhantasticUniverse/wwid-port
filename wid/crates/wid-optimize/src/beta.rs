//! Beta factor calibration.
//!
//! Calibrates the mouthpiece beta factor using all fingerings with
//! `frequencyMin` targets. Uses Brent's univariate minimizer with the
//! FminEvaluator error vector.
//!
//! Port of `BetaObjectiveFunction` from WIDesigner.

use wid_compile::{compile, get_beta, set_beta};
use wid_eval::evaluators::calculate_fmin_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{WhistleCalibrationResult, brent_min, calc_norm, fingering_weights};

/// Default beta bounds (dimensionless).
pub const DEFAULT_BETA_LOWER: f64 = 0.2;
pub const DEFAULT_BETA_UPPER: f64 = 1.0;

/// Calibrate beta to minimize fmin error.
///
/// The instrument is modified in place with the optimal beta.
pub fn calibrate_beta(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    lower_bound: f64,
    upper_bound: f64,
    calc_params: &CalculatorParams,
) -> WhistleCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_beta = get_beta(instrument).unwrap_or(0.35);
    let initial_norm = evaluate_fmin_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let start = initial_beta.clamp(lower_bound, upper_bound);

    let mut work_inst = instrument.clone();
    let result = brent_min::brent_minimize(
        &mut |b| {
            set_beta(&mut work_inst, b);
            evaluate_fmin_norm(&work_inst, &tuning.fingerings, &weights, params, calc_params)
        },
        lower_bound,
        upper_bound,
        start,
        1e-6,
        1e-14,
        50_000,
    );

    let (final_beta, final_norm) = result.unwrap_or((initial_beta, initial_norm));

    set_beta(instrument, final_beta);

    WhistleCalibrationResult {
        initial_window_height: None,
        final_window_height: None,
        initial_beta: Some(initial_beta),
        final_beta: Some(final_beta),
        initial_norm,
        final_norm,
    }
}

fn evaluate_fmin_norm(
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
    let errors = calculate_fmin_error_vector(&compiled, fingerings, params, calc_params);
    calc_norm(&errors, weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wid_physics::TemperatureType;
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    const FEADOG_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/FeadogMk1.xml");
    const FEADOG_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/tunings/FeadogMk1-tuning.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // Golden: calib_beta.json
    const GOLDEN_INITIAL_BETA: f64 = 0.522;
    const GOLDEN_FINAL_BETA: f64 = 0.5101134359331868;
    const GOLDEN_INITIAL_NORM: f64 = 7456.295495871915;
    const GOLDEN_FINAL_NORM: f64 = 7157.908437424858;

    #[test]
    fn initial_beta_matches() {
        let inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let beta = get_beta(&inst).unwrap();
        assert!(
            (beta - GOLDEN_INITIAL_BETA).abs() < 1e-10,
            "initial beta: expected {GOLDEN_INITIAL_BETA}, got {beta}"
        );
    }

    #[test]
    fn initial_norm_matches() {
        let inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let tuning = parse_tuning_xml(FEADOG_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_fmin_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE);
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn calibration_matches_golden() {
        let mut inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let tuning = parse_tuning_xml(FEADOG_TUNING_XML).unwrap();
        let params = default_params();

        let result = calibrate_beta(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_BETA_LOWER,
            DEFAULT_BETA_UPPER,
            &CalculatorParams::WHISTLE,
        );

        // Beta: within 5% of golden
        let final_beta = result.final_beta.unwrap();
        assert!(
            (final_beta - GOLDEN_FINAL_BETA).abs() / GOLDEN_FINAL_BETA < 0.05,
            "final beta: expected {GOLDEN_FINAL_BETA}, got {final_beta}, diff {:.6}",
            (final_beta - GOLDEN_FINAL_BETA).abs()
        );

        // Final norm: should be no worse than golden (+ small tolerance)
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 1.1,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }
}
