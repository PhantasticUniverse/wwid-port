//! Airstream length calibration (1D Brent minimizer).
//!
//! Matches Java `AirstreamLengthObjectiveFunction` — a single-parameter
//! calibrator that adjusts `embouchureHole.airstreamLength` (or
//! `fipple.windowLength` for fipple instruments) to minimize the
//! FmaxEvaluator error norm.
//!
//! Used by: Flute study model (embouchure path).
//! Algorithm: Brent's univariate minimizer within `[lower, upper]` bounds.

use wid_compile::{compile, get_airstream_length, set_airstream_length};
use wid_eval::evaluators::calculate_fmax_error_vector;
use wid_eval::CalculatorParams;
use wid_physics::PhysicalParameters;
use wid_types::{InstrumentRaw, Tuning};

use crate::{FluteCalibrationResult, brent_min, calc_norm, fingering_weights};

/// Default airstream length lower bound (metres).
/// From `AirstreamLengthDefaultConstraints.xml`: `1.0E-4`.
pub const DEFAULT_AL_LOWER: f64 = 0.0001;

/// Default airstream length upper bound (metres).
/// From `AirstreamLengthDefaultConstraints.xml`: `0.02`.
pub const DEFAULT_AL_UPPER: f64 = 0.020;

/// Calibrate airstream length to minimize fmax error.
///
/// The instrument is modified in place with the optimal airstream length.
/// Returns initial/final values and norms for parity verification.
pub fn calibrate_airstream_length(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    lower_bound: f64,
    upper_bound: f64,
    calc_params: &CalculatorParams,
) -> FluteCalibrationResult {
    let weights = fingering_weights(&tuning.fingerings);

    let initial_al = get_airstream_length(instrument).unwrap_or(0.004);
    let initial_norm = evaluate_fmax_norm(
        instrument,
        &tuning.fingerings,
        &weights,
        params,
        calc_params,
    );

    let start = initial_al.clamp(lower_bound, upper_bound);

    let mut work_inst = instrument.clone();
    let result = brent_min::brent_minimize(
        &mut |al| {
            set_airstream_length(&mut work_inst, al);
            evaluate_fmax_norm(&work_inst, &tuning.fingerings, &weights, params, calc_params)
        },
        lower_bound,
        upper_bound,
        start,
        1e-6,
        1e-14,
        50_000,
    );

    let (final_al, final_norm) = result.unwrap_or((initial_al, initial_norm));

    set_airstream_length(instrument, final_al);

    FluteCalibrationResult {
        initial_airstream_length: Some(initial_al),
        final_airstream_length: Some(final_al),
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

    const FLUTE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/instruments/SamplePVC-Flute.xml");
    const FLUTE_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/tunings/D4-Equal.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // Golden: FL-CAL/calib_airstream_length.json
    // FmaxEvaluator with frequency-only tuning: all errors are 0 (no frequencyMax targets).
    // initial_norm=0.0, final_norm=0.0, AL stays at 0.004.
    const GOLDEN_INITIAL_AL: f64 = 0.004;

    #[test]
    fn initial_airstream_length_from_flute() {
        let inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let al = get_airstream_length(&inst).unwrap();
        assert!(
            (al - GOLDEN_INITIAL_AL).abs() < 1e-10,
            "initial AL: expected {GOLDEN_INITIAL_AL}, got {al}"
        );
    }

    #[test]
    fn set_airstream_length_roundtrips() {
        let mut inst = parse_instrument_xml(FLUTE_XML).unwrap();
        set_airstream_length(&mut inst, 0.010);
        let al = get_airstream_length(&inst).unwrap();
        assert!(
            (al - 0.010).abs() < 1e-10,
            "roundtrip AL: expected 0.010, got {al}"
        );
    }

    #[test]
    fn initial_fmax_norm_is_zero_for_frequency_only_tuning() {
        // D4-Equal has only <frequency> targets, no <frequencyMax>.
        // FmaxEvaluator returns 0 for all fingerings → norm = 0.
        let inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_fmax_norm(
            &inst,
            &tuning.fingerings,
            &weights,
            &params,
            &CalculatorParams::FLUTE,
        );
        assert!(
            norm < 1e-10,
            "fmax norm with frequency-only tuning should be 0.0, got {norm}"
        );
    }

    #[test]
    fn calibration_preserves_zero_norm() {
        // Golden: initial_norm=0.0, final_norm=0.0. Calibration on a flat
        // objective should not increase the norm (Brent may wander, but the
        // norm stays 0.0 because fmax evaluator returns 0 for all fingerings).
        let mut inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();

        let result = calibrate_airstream_length(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_AL_LOWER,
            DEFAULT_AL_UPPER,
            &CalculatorParams::FLUTE,
        );

        // Norm should stay at 0.0
        assert!(
            result.final_norm < 1e-10,
            "final norm should be 0.0, got {}",
            result.final_norm
        );
    }

    #[test]
    fn fife_calibration_does_not_worsen_norm() {
        let fife_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/instruments/fife.xml"
        );
        let fife_tuning_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/tunings/fife-tuning.xml"
        );
        let mut inst = parse_instrument_xml(fife_xml).unwrap();
        let tuning = parse_tuning_xml(fife_tuning_xml).unwrap();
        let params = default_params();

        let result = calibrate_airstream_length(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_AL_LOWER,
            DEFAULT_AL_UPPER,
            &CalculatorParams::FLUTE,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "fife calibration should not worsen norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm,
        );
    }
}
