//! Joint window height + beta calibration.
//!
//! Calibrates both mouthpiece window height and beta simultaneously using
//! 2D BOBYQA with the FminmaxEvaluator error vector.
//!
//! Port of `WhistleCalibrationObjectiveFunction` from WIDesigner.

use bobyqa_impl::bobyqa_minimize;
use wid_compile::{compile, get_beta, get_window_height, set_beta, set_window_height};
use wid_eval::evaluators::calculate_fminmax_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{WhistleCalibrationResult, calc_norm, fingering_weights};

/// Calibrate window height and beta jointly to minimize combined fmin+fmax error.
///
/// The instrument is modified in place with the optimal values.
pub fn calibrate_whistle(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    wh_bounds: (f64, f64),
    beta_bounds: (f64, f64),
    calc_params: &CalculatorParams,
) -> WhistleCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_wh = get_window_height(instrument).unwrap_or(0.005);
    let initial_beta = get_beta(instrument).unwrap_or(0.35);
    let initial_norm = evaluate_fminmax_norm(
        instrument,
        &tuning.fingerings,
        &weights,
        params,
        calc_params,
    );

    let initial_point = [
        initial_wh.clamp(wh_bounds.0, wh_bounds.1),
        initial_beta.clamp(beta_bounds.0, beta_bounds.1),
    ];
    let lower_bounds = [wh_bounds.0, beta_bounds.0];
    let upper_bounds = [wh_bounds.1, beta_bounds.1];

    let n_dims = 2;
    let n_interp = 2 * n_dims + 1;
    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = 20000 + (n_dims - 1) * 5000;

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = bobyqa_minimize(
        &mut |point: &[f64]| {
            set_window_height(&mut work_inst, point[0]);
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

    let (final_wh, final_beta, final_norm) = match result {
        Some(opt) => (opt.point[0], opt.point[1], opt.value),
        None => (initial_wh, initial_beta, initial_norm),
    };

    set_window_height(instrument, final_wh);
    set_beta(instrument, final_beta);

    WhistleCalibrationResult {
        initial_window_height: Some(initial_wh),
        final_window_height: Some(final_wh),
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

    const FEADOG_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/FeadogMk1.xml");
    const FEADOG_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/tunings/FeadogMk1-tuning.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // Golden: calib_joint.json
    const GOLDEN_FINAL_WH: f64 = 0.002460124433667274;
    const GOLDEN_FINAL_BETA: f64 = 0.5182118511250104;
    const GOLDEN_INITIAL_NORM: f64 = 39125.90968943595;
    const GOLDEN_FINAL_NORM: f64 = 33390.32121843042;

    #[test]
    fn initial_norm_matches() {
        let inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let tuning = parse_tuning_xml(FEADOG_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_fminmax_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE);
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn joint_calibration_matches_golden() {
        let mut inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let tuning = parse_tuning_xml(FEADOG_TUNING_XML).unwrap();
        let params = default_params();

        let wh_bounds = (
            crate::window_height::DEFAULT_WH_LOWER,
            crate::window_height::DEFAULT_WH_UPPER,
        );
        let beta_bounds = (
            crate::beta::DEFAULT_BETA_LOWER,
            crate::beta::DEFAULT_BETA_UPPER,
        );

        let result = calibrate_whistle(
            &mut inst,
            &tuning,
            &params,
            wh_bounds,
            beta_bounds,
            &CalculatorParams::WHISTLE,
        );

        // Window height: within 10% of golden (BOBYQA paths can diverge)
        let final_wh = result.final_window_height.unwrap();
        assert!(
            (final_wh - GOLDEN_FINAL_WH).abs() / GOLDEN_FINAL_WH < 0.10,
            "final WH: expected {GOLDEN_FINAL_WH}, got {final_wh}"
        );

        // Beta: within 10% of golden
        let final_beta = result.final_beta.unwrap();
        assert!(
            (final_beta - GOLDEN_FINAL_BETA).abs() / GOLDEN_FINAL_BETA < 0.10,
            "final beta: expected {GOLDEN_FINAL_BETA}, got {final_beta}"
        );

        // Final norm: should improve from initial and be in same ballpark as golden
        assert!(
            result.final_norm < result.initial_norm,
            "optimization should reduce norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm
        );
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 1.5,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }
}
