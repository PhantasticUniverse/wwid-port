//! LinearV predicted frequency model for Whistle instruments.
//!
//! Upstream references:
//! - `LinearVInstrumentTuner.java` — velocity interpolation, blowing level tables
//! - `WhistleCalculator.java` — Strouhal number constants
//! - `PlayingRange.java` — bracket finding, findFmin, findZRatio
//!
//! # Physical model
//!
//! The whistle's playing frequency is determined by the Strouhal number coupling
//! between the air jet and the acoustic resonance. The Strouhal number relates
//! the jet velocity, window length, and frequency:
//!
//! ```text
//! St = f * windowLength / velocity
//! St ≈ 0.26 - 0.037 * Im(Z)/Re(Z)
//! ```
//!
//! # Algorithm
//!
//! 1. **Setup** (`LinearVTuner::new`): Pre-compute a linear velocity model from
//!    the full tuning. Find playing ranges (fmax, fmin) for the lowest and highest
//!    target notes, compute velocities at those extremes, then interpolate:
//!    `v_nom = slope * freq + intercept`.
//!
//! 2. **Prediction** (`predicted_frequency_linear_v`): For each fingering:
//!    - Compute `v_nom = slope * target + intercept`
//!    - Compute `target_ratio = z_ratio(target, windowLength, v_nom)`
//!    - Find frequency where `Im(Z)/Re(Z) = target_ratio` (Brent root-finding)
//!
//! # Constants
//!
//! | Name | Value | Description |
//! |------|-------|-------------|
//! | STROUHAL_ZERO | 0.26 | Strouhal number at Im(Z) = 0 |
//! | STROUHAL_SLOPE | -0.037 | Change per unit Im(Z)/Re(Z) |
//! | STROUHAL_MAX | 0.50 | Max St (below fmin) |
//! | STROUHAL_MIN | 0.01 | Min St (above fmax) |

use num_complex::Complex64;
use wid_compile::{InstrumentCompiled, MouthpieceType};
use wid_physics::PhysicalParameters;
use wid_types::Fingering;

use crate::{CalculatorParams, calc_z, brent};

// ── Strouhal model constants ────────────────────────────────────────

const STROUHAL_ZERO: f64 = 0.26;
const STROUHAL_SLOPE: f64 = -0.037;

/// Blowing level interpolation tables from LinearVInstrumentTuner.java.
/// Index 0 = blowing level 0, index 10 = blowing level 10.
const BOTTOM_FRACTIONS: [f64; 11] = [
    0.35, 0.35, 0.30, 0.30, 0.25, 0.25, 0.20, 0.15, 0.10, 0.10, 0.05,
];
const TOP_FRACTIONS: [f64; 11] = [
    0.80, 0.85, 0.90, 0.95, 0.90, 0.95, 0.95, 0.95, 0.95, 0.99, 0.99,
];

// ── Velocity / Strouhal helpers ─────────────────────────────────────

/// Estimate air velocity from frequency, window length, and impedance.
///
/// Uses the Strouhal number relationship:
/// ```text
/// St = f * windowLength / velocity
/// St = St_zero - |St_slope| * Im(Z)/Re(Z)
/// ```
/// Clamps St to [0.13, 0.75] to stay within reasonable physical bounds
/// (matching `LinearVInstrumentTuner.velocity`).
pub fn velocity(freq: f64, window_length: f64, z: Complex64) -> f64 {
    if z.re == 0.0 {
        return 0.0;
    }
    let strouhal = STROUHAL_ZERO - STROUHAL_SLOPE.abs() * z.im / z.re;
    let strouhal = strouhal.clamp(0.13, 0.75);
    freq * window_length / strouhal
}

/// Predict the expected Im(Z)/Re(Z) ratio for a given air velocity.
fn z_ratio(freq: f64, window_length: f64, vel: f64) -> f64 {
    (STROUHAL_ZERO - freq * window_length / vel) / STROUHAL_SLOPE.abs()
}

/// Extract window_length (airstream length) from a compiled mouthpiece.
///
/// Matches Java `Mouthpiece.getAirstreamLength()`:
/// - Fipple: returns `windowLength`
/// - EmbouchureHole: returns `airstreamLength`
/// - SimpleReed: panics (LinearV not used with reed)
pub fn window_length(instrument: &InstrumentCompiled) -> f64 {
    match &instrument.mouthpiece.mouthpiece_type {
        MouthpieceType::Fipple { window_length, .. } => *window_length,
        MouthpieceType::EmbouchureHole { airstream_length, .. } => *airstream_length,
        MouthpieceType::SimpleReed { .. } => {
            unreachable!("LinearV tuner not used with reed mouthpieces")
        }
    }
}

fn bottom_fraction(blowing_level: u8) -> f64 {
    let level = (blowing_level as usize).min(10);
    BOTTOM_FRACTIONS[level]
}

fn top_fraction(blowing_level: u8) -> f64 {
    let level = (blowing_level as usize).min(10);
    TOP_FRACTIONS[level]
}

// ── LinearV tuner ───────────────────────────────────────────────────

/// Pre-computed linear velocity interpolation model.
///
/// Built from the full set of fingerings, provides `v_nom = slope * f + intercept`
/// for any target frequency in the instrument's range.
#[derive(Debug, Clone)]
pub struct LinearVTuner {
    slope: f64,
    intercept: f64,
}

impl LinearVTuner {
    /// Pre-compute the velocity interpolation line from a full set of fingerings.
    ///
    /// Finds the lowest and highest target frequencies, locates playing ranges
    /// (fmax, fmin) for each, computes velocities, and builds the linear model.
    pub fn new(
        instrument: &InstrumentCompiled,
        fingerings: &[Fingering],
        params: &PhysicalParameters,
        calc_params: &CalculatorParams,
        blowing_level: u8,
    ) -> Self {
        if fingerings.is_empty() {
            return LinearVTuner { slope: 0.0, intercept: 1.0 };
        }

        let wl = window_length(instrument);

        // Find lowest and highest target frequencies (with optimization_weight > 0)
        let mut f_low_target = f64::MAX;
        let mut f_high_target = 0.0_f64;
        let mut note_low_idx = 0;
        let mut note_high_idx = 0;

        for (i, f) in fingerings.iter().enumerate() {
            let weight = f.optimization_weight.unwrap_or(1);
            if weight <= 0 {
                continue;
            }
            if let Some(freq) = f.note.frequency {
                if freq < f_low_target {
                    f_low_target = freq;
                    note_low_idx = i;
                }
                if freq > f_high_target {
                    f_high_target = freq;
                    note_high_idx = i;
                }
            } else if let Some(fmax) = f.note.frequency_max {
                if fmax < f_low_target {
                    f_low_target = fmax;
                    note_low_idx = i;
                }
                if fmax > f_high_target {
                    f_high_target = fmax;
                    note_high_idx = i;
                }
            }
        }

        // No usable fingering found — return identity tuner
        if f_low_target == f64::MAX {
            return LinearVTuner { slope: 0.0, intercept: 1.0 };
        }

        let bottom_frac = bottom_fraction(blowing_level);
        let top_frac = top_fraction(blowing_level);

        // Compute velocity at lowest note
        let (f_low_nom, v_low) = compute_velocity_at_note(
            instrument, &fingerings[note_low_idx], f_low_target,
            wl, params, calc_params, bottom_frac, true,
        );

        // Compute velocity at highest note
        let (f_high_nom, v_high) = compute_velocity_at_note(
            instrument, &fingerings[note_high_idx], f_high_target,
            wl, params, calc_params, top_frac, false,
        );

        // Linear interpolation: v_nom = slope * f + intercept
        let slope = if (f_high_nom - f_low_nom).abs() > f64::EPSILON {
            (v_high - v_low) / (f_high_nom - f_low_nom)
        } else {
            0.0
        };
        let intercept = v_low - slope * f_low_nom;

        LinearVTuner { slope, intercept }
    }

    /// Get the nominal velocity at a given frequency.
    fn nominal_v(&self, freq: f64) -> f64 {
        self.slope * freq + self.intercept
    }
}

/// Compute velocity at a note's playing range boundary.
///
/// Returns `(nominal_freq, velocity)` where:
/// - For the low note: nominal_freq = fmax, velocity = vMax - fraction*(vMax-vMin)
/// - For the high note: nominal_freq = fmin, velocity = vMax - fraction*(vMax-vMin)
#[allow(clippy::too_many_arguments)]
fn compute_velocity_at_note(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    target_freq: f64,
    wl: f64,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    fraction: f64,
    is_low_note: bool,
) -> (f64, f64) {
    // Find fmax (reactance zero near target)
    let fmax = match find_x_zero_for_fingering(instrument, fingering, params, calc_params, target_freq) {
        Some(f) => f,
        None => {
            // Fallback: use target_freq as fmax, velocity with Z = 1+0i (Im(Z)=0)
            let v = target_freq * wl / STROUHAL_ZERO; // St = 0.26 when Im(Z)/Re(Z)=0
            return (target_freq, v);
        }
    };

    // Find fmin (local minimum of Im(Z)/Re(Z) below fmax)
    let fmin = match find_fmin(instrument, fingering, params, calc_params, fmax) {
        Some(f) => f,
        None => {
            let v = target_freq * wl / STROUHAL_ZERO;
            return (target_freq, v);
        }
    };

    // Compute velocities at fmax and fmin
    let z_at_fmax = calc_z(instrument, fmax, fingering, params, calc_params);
    let z_at_fmin = calc_z(instrument, fmin, fingering, params, calc_params);
    let v_max = velocity(fmax, wl, z_at_fmax);
    let v_min = velocity(fmin, wl, z_at_fmin);

    let v = v_max - fraction * (v_max - v_min);

    // For low note: use fmax as nominal. For high note: use fmin as nominal.
    if is_low_note {
        (fmax, v)
    } else {
        (fmin, v)
    }
}

// ── Playing range helpers ───────────────────────────────────────────

/// Find the reactance zero (fmax) near a target frequency for a specific fingering.
pub fn find_x_zero_for_fingering(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    near_freq: f64,
) -> Option<f64> {
    let reactance = |f: f64| -> f64 {
        calc_z(instrument, f, fingering, params, calc_params).im
    };
    let bracket = crate::find_bracket(near_freq, &reactance)?;
    brent::find_root(&reactance, bracket.0, bracket.1, 1e-6, 1e-6, 50)
}

/// Compute loop gain: G = gain_factor * freq * rho / |Z|.
/// Returns 1.0 if gain_factor is None (no gain model).
pub fn calc_gain(gain_factor: Option<f64>, freq: f64, z: Complex64, rho: f64) -> f64 {
    match gain_factor {
        Some(g0) => g0 * freq * rho / z.norm(),
        None => 1.0,
    }
}

/// Find fmin for a playing range, given fmax.
///
/// fmin is the highest frequency <= fmax that satisfies either:
/// - gain(fmin) == MinimumGain (1.0), or
/// - fmin is a local minimum of Im(Z)/Re(Z).
///
/// Matches the Java `PlayingRange.findFmin()` behavior including gain check.
pub fn find_fmin(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    fmax: f64,
) -> Option<f64> {
    let step_size = fmax * crate::GRANULARITY;
    let lower_bound = fmax / crate::SEARCH_BOUND_RATIO;
    let gain_factor = instrument.mouthpiece.gain_factor;
    let rho = params.rho();

    // Step down from fmax until gain < 1.0 OR Im(Z)/Re(Z) starts increasing
    let z_fmax = calc_z(instrument, fmax, fingering, params, calc_params);
    let mut g_lo = calc_gain(gain_factor, fmax, z_fmax, rho);
    let mut ratio = z_fmax.im / z_fmax.re;
    let mut min_ratio = ratio + 1.0; // ensures first iteration passes
    let mut lower_freq = fmax;

    if g_lo < 1.0 {
        // Gain too small even at fmax — no playing range
        return None;
    }

    while g_lo >= 1.0 && ratio < min_ratio {
        min_ratio = ratio;
        lower_freq -= step_size;
        if lower_freq < lower_bound {
            return None;
        }
        let z = calc_z(instrument, lower_freq, fingering, params, calc_params);
        g_lo = calc_gain(gain_factor, lower_freq, z, rho);
        ratio = z.im / z.re;
    }

    // Find freqGain: frequency where gain == 1.0
    let freq_gain = if g_lo < 1.0 {
        // Gain dropped below 1.0 — find exact crossing
        let gain_fn = |f: f64| -> f64 {
            let z = calc_z(instrument, f, fingering, params, calc_params);
            calc_gain(gain_factor, f, z, rho) - 1.0
        };
        brent::find_root(&gain_fn, lower_freq, fmax, 1e-6, 1e-6, 50)
            .unwrap_or(lower_freq)
    } else {
        lower_freq
    };

    // Find freqRatio: local minimum of Im(Z)/Re(Z)
    let ratio_fn = |f: f64| -> f64 {
        let z = calc_z(instrument, f, fingering, params, calc_params);
        z.im / z.re
    };

    let (freq_ratio, _) = brent::find_minimum(
        &ratio_fn,
        lower_freq,
        fmax,
        0.0001,
        0.0001,
        50,
    );

    // Return max(freqGain, freqRatio) — matching Java
    Some(freq_ratio.max(freq_gain))
}

/// Find the frequency where Im(Z)/Re(Z) = target_ratio.
///
/// Uses the same bracket-finding logic as `find_bracket` for reactance,
/// but with the Z ratio function instead.
fn find_z_ratio(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    near_freq: f64,
    target_ratio: f64,
) -> Option<f64> {
    let ratio_fn = |f: f64| -> f64 {
        let z = calc_z(instrument, f, fingering, params, calc_params);
        z.im / z.re - target_ratio
    };

    let bracket = crate::find_bracket(near_freq, &ratio_fn)?;
    brent::find_root(&ratio_fn, bracket.0, bracket.1, 1e-6, 1e-6, 50)
}

// ── Public predicted frequency ──────────────────────────────────────

/// Find predicted frequency for a fingering using the LinearV model.
///
/// Uses the pre-computed velocity interpolation line to determine the
/// target Im(Z)/Re(Z) ratio, then finds the frequency at which the
/// instrument achieves that ratio.
pub fn predicted_frequency_linear_v(
    tuner: &LinearVTuner,
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> Option<f64> {
    let target_freq = fingering.note.frequency?;
    let wl = window_length(instrument);
    let v_nom = tuner.nominal_v(target_freq);
    let target_ratio = z_ratio(target_freq, wl, v_nom);

    find_z_ratio(instrument, fingering, params, calc_params, target_freq, target_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use wid_physics::{PhysicalParameters, TemperatureType};
    use wid_types::parse_instrument_xml;

    const NAF_0HOLE_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-FF-02_instrument_0hole.xml");

    #[test]
    fn empty_fingerings_does_not_panic() {
        let raw = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        let compiled = wid_compile::compile(&raw).unwrap();
        let params = PhysicalParameters::new(72.0, TemperatureType::F);
        let tuner = LinearVTuner::new(&compiled, &[], &params, &CalculatorParams::NAF, 5);
        // Should return identity tuner, not panic
        assert_eq!(tuner.slope, 0.0);
        assert_eq!(tuner.intercept, 1.0);
    }

    #[test]
    fn no_frequency_fingerings_does_not_panic() {
        use wid_types::{Fingering, Note};
        let raw = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        let compiled = wid_compile::compile(&raw).unwrap();
        let params = PhysicalParameters::new(72.0, TemperatureType::F);
        let fingerings = vec![Fingering {
            note: Note { name: "X".to_string(), frequency: None, frequency_min: None, frequency_max: None },
            open_holes: vec![],
            open_end: None,
            optimization_weight: None,
        }];
        let tuner = LinearVTuner::new(&compiled, &fingerings, &params, &CalculatorParams::NAF, 5);
        assert_eq!(tuner.slope, 0.0);
        assert_eq!(tuner.intercept, 1.0);
    }
}
