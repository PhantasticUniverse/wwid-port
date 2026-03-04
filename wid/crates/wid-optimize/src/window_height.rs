//! Window height calibration.
//!
//! Calibrates the mouthpiece window height using all fingerings with
//! `frequencyMax` targets. Uses Brent's univariate minimizer with the
//! FmaxEvaluator error vector.
//!
//! Port of `WindowHeightObjectiveFunction` from WIDesigner.

use wid_compile::{compile, get_window_height, set_window_height};
use wid_eval::evaluators::calculate_fmax_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{WhistleCalibrationResult, brent_min, calc_norm, fingering_weights};

/// Default window height bounds (metres).
pub const DEFAULT_WH_LOWER: f64 = 0.0001;
pub const DEFAULT_WH_UPPER: f64 = 0.020;

/// Calibrate window height to minimize fmax error.
///
/// The instrument is modified in place with the optimal window height.
pub fn calibrate_window_height(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    lower_bound: f64,
    upper_bound: f64,
    calc_params: &CalculatorParams,
) -> WhistleCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_wh = get_window_height(instrument).unwrap_or(0.005);
    let initial_norm = evaluate_fmax_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let start = initial_wh.clamp(lower_bound, upper_bound);

    let mut work_inst = instrument.clone();
    let result = brent_min::brent_minimize(
        &mut |wh| {
            set_window_height(&mut work_inst, wh);
            evaluate_fmax_norm(&work_inst, &tuning.fingerings, &weights, params, calc_params)
        },
        lower_bound,
        upper_bound,
        start,
        1e-6,
        1e-14,
        50_000,
    );

    let (final_wh, final_norm) = result.unwrap_or((initial_wh, initial_norm));

    set_window_height(instrument, final_wh);

    WhistleCalibrationResult {
        initial_window_height: Some(initial_wh),
        final_window_height: Some(final_wh),
        initial_beta: None,
        final_beta: None,
        initial_norm,
        final_norm,
    }
}

fn evaluate_fmax_norm(
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
    let errors = calculate_fmax_error_vector(&compiled, fingerings, params, calc_params);
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

    // Golden: calib_window_height.json
    const GOLDEN_INITIAL_WH: f64 = 0.0029;
    const GOLDEN_FINAL_WH: f64 = 0.00245802722837824;
    const GOLDEN_INITIAL_NORM: f64 = 1979.3508870977525;
    const GOLDEN_FINAL_NORM: f64 = 1639.3462863942636;

    #[test]
    fn initial_window_height_matches() {
        let inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let wh = get_window_height(&inst).unwrap();
        assert!(
            (wh - GOLDEN_INITIAL_WH).abs() < 1e-10,
            "initial WH: expected {GOLDEN_INITIAL_WH}, got {wh}"
        );
    }

    #[test]
    fn initial_norm_matches() {
        let inst = parse_instrument_xml(FEADOG_XML).unwrap();
        let tuning = parse_tuning_xml(FEADOG_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_fmax_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE);
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

        let result = calibrate_window_height(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_WH_LOWER,
            DEFAULT_WH_UPPER,
            &CalculatorParams::WHISTLE,
        );

        // Window height: within 5% of golden
        let final_wh = result.final_window_height.unwrap();
        assert!(
            (final_wh - GOLDEN_FINAL_WH).abs() / GOLDEN_FINAL_WH < 0.05,
            "final WH: expected {GOLDEN_FINAL_WH}, got {final_wh}, diff {:.6}",
            (final_wh - GOLDEN_FINAL_WH).abs()
        );

        // Final norm: should be no worse than golden (+ small tolerance)
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 1.1,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }
}
