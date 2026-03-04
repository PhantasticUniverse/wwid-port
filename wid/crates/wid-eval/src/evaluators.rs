//! Specialized evaluator functions for Whistle calibration.
//!
//! These compute error vectors based on fmax and fmin predictions,
//! used by the Whistle calibrator objective functions:
//! - [`calculate_fmax_error_vector`] — for `WindowHeightObjectiveFunction`
//! - [`calculate_fmin_error_vector`] — for `BetaObjectiveFunction`
//! - [`calculate_fminmax_error_vector`] — for `WhistleCalibrationObjectiveFunction`

use wid_compile::InstrumentCompiled;
use wid_physics::PhysicalParameters;
use wid_types::Fingering;

use crate::linear_v::{self, find_fmin, find_x_zero_for_fingering, LinearVTuner};
use crate::{CalculatorParams, cents};

/// Default penalty when fmax/fmin prediction fails.
const FMAX_PENALTY: f64 = 400.0;
const FMIN_PENALTY: f64 = 400.0;
const FMINMAX_PENALTY: f64 = 1200.0;

/// Weights for the FminmaxEvaluator (matching Java FminmaxEvaluator).
const FMAX_WEIGHT: f64 = 4.0;
const FMIN_WEIGHT: f64 = 1.0;
const FPLAYING_WEIGHT: f64 = 1.0;

/// Predict fmax (Im(Z)=0 crossing near target) for a single fingering.
///
/// Uses `frequencyMax` as the target frequency, falling back to `frequency`.
pub fn predicted_fmax(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Option<f64> {
    let target = fingering
        .note
        .frequency_max
        .or(fingering.note.frequency)?;
    find_x_zero_for_fingering(instrument, fingering, params, calc_params, target)
}

/// Predict fmin (gain-aware minimum) for a single fingering.
pub fn predicted_fmin(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Option<f64> {
    let fmax = predicted_fmax(instrument, fingering, params, calc_params)?;
    find_fmin(instrument, fingering, params, calc_params, fmax)
}

/// Compute error vector based on fmax deviation.
///
/// Used by `WindowHeightObjectiveFunction` via `FmaxEvaluator`.
/// For each fingering with a `frequencyMax` target, returns
/// `cents(target_fmax, predicted_fmax)`. Fingerings without
/// `frequencyMax` contribute 0.0.
pub fn calculate_fmax_error_vector(
    instrument: &InstrumentCompiled,
    fingerings: &[Fingering],
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Vec<f64> {
    fingerings
        .iter()
        .map(|f| {
            let target_fmax = match f.note.frequency_max {
                Some(fmax) => fmax,
                None => return 0.0,
            };
            match predicted_fmax(instrument, f, params, calc_params) {
                Some(pred_fmax) => cents(target_fmax, pred_fmax),
                None => FMAX_PENALTY,
            }
        })
        .collect()
}

/// Compute error vector based on fmin deviation.
///
/// Used by `BetaObjectiveFunction` via `FminEvaluator`.
/// For each fingering with a `frequencyMin` target, returns
/// `cents(target_fmin, predicted_fmin)`. Fingerings without
/// `frequencyMin` contribute 0.0.
pub fn calculate_fmin_error_vector(
    instrument: &InstrumentCompiled,
    fingerings: &[Fingering],
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Vec<f64> {
    fingerings
        .iter()
        .map(|f| {
            let target_fmin = match f.note.frequency_min {
                Some(fmin) => fmin,
                None => return 0.0,
            };
            match predicted_fmin(instrument, f, params, calc_params) {
                Some(pred_fmin) => cents(target_fmin, pred_fmin),
                None => FMIN_PENALTY,
            }
        })
        .collect()
}

/// Compute error vector combining fmin and fmax with RMS weighting.
///
/// Used by `WhistleCalibrationObjectiveFunction` and
/// `FluteCalibrationObjectiveFunction` via `FminmaxEvaluator`.
///
/// For each fingering:
/// - If `frequencyMax` is available: `dev = FMAX_WEIGHT * cents(fmax)`
///   - If also `frequencyMin`: `dev = sqrt(dev^2 + (FMIN_WEIGHT * cents(fmin))^2)`
/// - Else if `frequencyMin` only: `dev = FMIN_WEIGHT * cents(fmin)`
/// - Else if `frequency` only: `dev = FPLAYING_WEIGHT * cents(freq, predicted_playing_freq)`
///   (uses LinearV tuner for predicted playing frequency, matching Java)
/// - Else: `dev = 0.0`
pub fn calculate_fminmax_error_vector(
    instrument: &InstrumentCompiled,
    fingerings: &[Fingering],
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Vec<f64> {
    // Build LinearV tuner once for the frequency-only fallback path.
    let tuner = LinearVTuner::new(
        instrument,
        fingerings,
        params,
        calc_params,
        calc_params.blowing_level,
    );

    fingerings
        .iter()
        .map(|f| {
            if let Some(target_fmax) = f.note.frequency_max {
                // Has frequencyMax
                let fmax_dev = match predicted_fmax(instrument, f, params, calc_params) {
                    Some(pred) => FMAX_WEIGHT * cents(target_fmax, pred),
                    None => FMINMAX_PENALTY,
                };

                if let Some(target_fmin) = f.note.frequency_min {
                    // Also has frequencyMin — combine with RMS
                    let fmin_dev = match predicted_fmin(instrument, f, params, calc_params) {
                        Some(pred) => FMIN_WEIGHT * cents(target_fmin, pred),
                        None => FMINMAX_PENALTY,
                    };
                    (fmax_dev * fmax_dev + fmin_dev * fmin_dev).sqrt()
                } else {
                    fmax_dev
                }
            } else if let Some(target_fmin) = f.note.frequency_min {
                // Only frequencyMin
                match predicted_fmin(instrument, f, params, calc_params) {
                    Some(pred) => FMIN_WEIGHT * cents(target_fmin, pred),
                    None => FMINMAX_PENALTY,
                }
            } else if let Some(target_freq) = f.note.frequency {
                // Only frequency — use LinearV tuner for predicted playing frequency,
                // matching Java FminmaxEvaluator's `predicted.getFrequency()` path.
                match linear_v::predicted_frequency_linear_v(
                    &tuner, instrument, f, params, calc_params,
                ) {
                    Some(pred) => FPLAYING_WEIGHT * cents(target_freq, pred),
                    None => FMINMAX_PENALTY,
                }
            } else {
                0.0
            }
        })
        .collect()
}
