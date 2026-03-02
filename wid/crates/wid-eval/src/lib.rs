//! Impedance evaluation, frequency prediction, and cents deviation.
//!
//! This crate ties together the full acoustic pipeline:
//!
//! 1. [`calc_z`] — impedance at a frequency for a given fingering
//! 2. [`predicted_frequency`] — finds the playing frequency (Im(Z)=0 crossing)
//! 3. [`calculate_error_vector`] — cents deviation per fingering
//!
//! The impedance pipeline walks the compiled instrument from termination
//! to mouthpiece, cascading transfer matrices through the state vector.

mod brent;

use num_complex::Complex64;
use wid_acoustics::{bore, hole, mouthpiece, termination};
use wid_compile::{Component, InstrumentCompiled};
use wid_physics::PhysicalParameters;
use wid_types::Fingering;

/// NAF hole size multiplier.
const NAF_HOLE_SIZE_MULT: f64 = 0.9605;

/// NAF finger adjustment (no adjustment for NAF).
const NAF_FINGER_ADJ: f64 = 0.0;

/// Compute the input impedance at a given frequency for a fingering.
///
/// Walks the compiled instrument from termination to mouthpiece:
/// 1. Initialize state vector at termination
/// 2. Walk components in reverse, applying transfer matrices
/// 3. Apply mouthpiece transfer matrix
/// 4. Return Z = P/U
pub fn calc_z(
    instrument: &InstrumentCompiled,
    freq: f64,
    fingering: &Fingering,
    params: &PhysicalParameters,
) -> Complex64 {
    let wave_number = params.calc_wave_number(freq);

    // Step 1: Termination state vector
    let is_open_end = fingering.open_end.unwrap_or(true);
    let mut sv =
        termination::calc_termination_sv(&instrument.termination, is_open_end, wave_number, params);

    // Step 2: Walk components in reverse (termination → mouthpiece)
    // Holes are matched to fingering in reverse order:
    // last hole in component list → openHole[n-1], first hole → openHole[0]
    let n_holes = fingering.open_holes.len();
    let mut next_hole_index: isize = n_holes as isize - 1;

    for component in instrument.components.iter().rev() {
        let tm = match component {
            Component::Bore(section) => {
                bore::calc_bore_section_tm(section, wave_number, params)
            }
            Component::Hole(h) => {
                let is_open = fingering.open_holes[next_hole_index as usize];
                next_hole_index -= 1;
                hole::calc_hole_tm(h, is_open, wave_number, params, NAF_HOLE_SIZE_MULT, NAF_FINGER_ADJ)
            }
        };
        sv = tm.multiply_sv(&sv);
    }

    // Step 3: Apply mouthpiece
    let mp_tm = mouthpiece::calc_fipple_mouthpiece_tm(
        &instrument.mouthpiece,
        wave_number,
        params,
    );
    sv = mp_tm.multiply_sv(&sv);

    // Step 4: Z = P/U
    sv.impedance()
}

/// Compute impedance at multiple frequencies for a fingering (Z-sample).
pub fn calc_z_samples(
    instrument: &InstrumentCompiled,
    frequencies: &[f64],
    fingering: &Fingering,
    params: &PhysicalParameters,
) -> Vec<Complex64> {
    frequencies
        .iter()
        .map(|&f| calc_z(instrument, f, fingering, params))
        .collect()
}

// ── Playing range and frequency prediction ──────────────────────

/// Search constants for bracket finding.
const SEARCH_BOUND_RATIO: f64 = 2.0; // within an octave
const PREFERRED_SOLUTION_RATIO: f64 = 1.12; // within ~200 cents
const GRANULARITY: f64 = 0.012; // ~20 cents step

/// Find the predicted playing frequency for a fingering.
///
/// Searches for the frequency where Im(Z) crosses zero with positive
/// slope (reactance zero), near the target frequency from the note.
/// Returns `None` if no playing frequency is found.
pub fn predicted_frequency(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
) -> Option<f64> {
    let target_freq = fingering.note.frequency?;
    find_x_zero(instrument, fingering, params, target_freq)
}

/// Find a zero of Im(Z) near `near_freq`.
fn find_x_zero(
    instrument: &InstrumentCompiled,
    fingering: &Fingering,
    params: &PhysicalParameters,
    near_freq: f64,
) -> Option<f64> {
    let reactance = |f: f64| -> f64 {
        calc_z(instrument, f, fingering, params).im
    };

    let bracket = find_bracket(near_freq, &reactance)?;

    // Brent solver: find root in bracket
    brent::find_root(&reactance, bracket.0, bracket.1, 1e-6, 1e-6, 50)
}

/// Find a bracket [lower, upper] where reactance changes sign with positive slope.
///
/// Searches in the primary direction first, then tries the opposite direction
/// if the primary result is outside the preferred range. Prefers the fallback
/// direction unconditionally when found (matching upstream logic).
fn find_bracket(
    near_freq: f64,
    reactance: &dyn Fn(f64) -> f64,
) -> Option<(f64, f64)> {
    let mut freq = near_freq;
    let mut val = reactance(freq);

    // If exactly zero, nudge slightly
    while val == 0.0 {
        freq *= 0.999;
        val = reactance(freq);
    }

    if val < 0.0 {
        // Reactance is negative: search upward first
        let up = find_bracket_above(freq, val, reactance, near_freq * SEARCH_BOUND_RATIO);

        let up_failed = up.is_none();
        let up_too_far = up.map_or(false, |(_, hi)| hi > near_freq * PREFERRED_SOLUTION_RATIO);

        if up_failed || up_too_far {
            let limit = if up_failed {
                near_freq / SEARCH_BOUND_RATIO
            } else {
                near_freq * near_freq / up.unwrap().1
            };
            if let Some(down) = find_bracket_below(freq, val, reactance, limit) {
                return Some(down);
            }
        }

        up
    } else {
        // Reactance is positive: search downward first
        let down = find_bracket_below(freq, val, reactance, near_freq / SEARCH_BOUND_RATIO);

        let down_failed = down.is_none();
        let down_too_far = down.map_or(false, |(lo, _)| lo < near_freq / PREFERRED_SOLUTION_RATIO);

        if down_failed || down_too_far {
            let limit = if down_failed {
                near_freq * SEARCH_BOUND_RATIO
            } else {
                near_freq * near_freq / down.unwrap().0
            };
            if let Some(up) = find_bracket_above(freq, val, reactance, limit) {
                return Some(up);
            }
        }

        down
    }
}

/// Search upward from `start_freq` for a sign change bracket.
fn find_bracket_above(
    start_freq: f64,
    start_val: f64,
    reactance: &dyn Fn(f64) -> f64,
    upper_bound: f64,
) -> Option<(f64, f64)> {
    let step = start_freq * GRANULARITY;
    let mut lower_freq = start_freq;
    let mut lower_val = start_val;

    // Ensure lower_val < 0
    while lower_val >= 0.0 {
        lower_freq += step;
        if lower_freq >= upper_bound {
            return None;
        }
        lower_val = reactance(lower_freq);
    }

    // Search upward for upper_val > 0
    let mut upper_freq = lower_freq + step;
    let mut upper_val = reactance(upper_freq);

    while upper_val <= 0.0 {
        if upper_val < 0.0 {
            lower_freq = upper_freq;
        }
        upper_freq += step;
        if upper_freq > upper_bound {
            return None;
        }
        upper_val = reactance(upper_freq);
    }

    Some((lower_freq, upper_freq))
}

/// Search downward from `start_freq` for a sign change bracket.
fn find_bracket_below(
    start_freq: f64,
    start_val: f64,
    reactance: &dyn Fn(f64) -> f64,
    lower_bound: f64,
) -> Option<(f64, f64)> {
    let step = start_freq * GRANULARITY;
    let mut upper_freq = start_freq;
    let mut upper_val = start_val;

    // Ensure upper_val > 0
    while upper_val <= 0.0 {
        upper_freq -= step;
        if upper_freq <= lower_bound {
            return None;
        }
        upper_val = reactance(upper_freq);
    }

    // Search downward for lower_val < 0
    let mut lower_freq = upper_freq - step;
    let mut lower_val = reactance(lower_freq);

    while lower_val >= 0.0 {
        if lower_val > 0.0 {
            upper_freq = lower_freq;
        }
        lower_freq -= step;
        if lower_freq < lower_bound {
            return None;
        }
        lower_val = reactance(lower_freq);
    }

    Some((lower_freq, upper_freq))
}

// ── Error vector ────────────────────────────────────────────────

/// Compute cents deviation for each fingering in a tuning.
///
/// Returns one value per fingering: 1200 × log2(predicted/target).
/// Returns 1200.0 (huge error) if prediction fails.
pub fn calculate_error_vector(
    instrument: &InstrumentCompiled,
    fingerings: &[Fingering],
    params: &PhysicalParameters,
) -> Vec<f64> {
    fingerings
        .iter()
        .map(|f| {
            if let Some(target) = f.note.frequency {
                if let Some(predicted) = predicted_frequency(instrument, f, params) {
                    cents(target, predicted)
                } else {
                    1200.0
                }
            } else {
                1200.0
            }
        })
        .collect()
}

/// Cents deviation: 1200 × log2(f_predicted / f_target).
pub fn cents(target_freq: f64, predicted_freq: f64) -> f64 {
    (predicted_freq / target_freq).ln() / 2.0_f64.ln() * 1200.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use wid_compile::compile;
    use wid_physics::TemperatureType;
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );

    fn setup() -> (InstrumentCompiled, wid_types::Tuning, PhysicalParameters) {
        let raw = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let inst = compile(&raw).unwrap();
        let tuning = parse_tuning_xml(TUNING_XML).unwrap();
        let params = PhysicalParameters::new(72.0, TemperatureType::F);
        (inst, tuning, params)
    }

    // ── Z-sample golden tests ───────────────────────────────────

    #[derive(serde::Deserialize)]
    struct ZSample {
        frequency: f64,
        #[serde(rename = "zReal")]
        z_real: f64,
        #[serde(rename = "zImag")]
        z_imag: f64,
    }

    const ZSAMPLE_JSON: &str = include_str!("../../../../golden/expected/NAF-FF-01/zsample_0.json");

    #[test]
    fn zsample_matches_golden() {
        let (inst, tuning, params) = setup();
        let samples: Vec<ZSample> = serde_json::from_str(ZSAMPLE_JSON).unwrap();

        // zsample_0 uses the first fingering (F#4, all closed)
        let fingering = &tuning.fingerings[0];

        for sample in &samples {
            let z = calc_z(&inst, sample.frequency, fingering, &params);

            // Tolerance: abs_err <= A + R * max(|expected|, |actual|)
            let a = 1.0; // absolute tolerance
            let r = 1e-6; // relative tolerance

            let tol_re = a + r * sample.z_real.abs().max(z.re.abs());
            let tol_im = a + r * sample.z_imag.abs().max(z.im.abs());

            assert!(
                (z.re - sample.z_real).abs() <= tol_re,
                "Re(Z) mismatch at {}Hz: expected {}, got {}, tol {}",
                sample.frequency, sample.z_real, z.re, tol_re
            );
            assert!(
                (z.im - sample.z_imag).abs() <= tol_im,
                "Im(Z) mismatch at {}Hz: expected {}, got {}, tol {}",
                sample.frequency, sample.z_imag, z.im, tol_im
            );
        }
    }

    // ── Eval golden tests ───────────────────────────────────────

    #[derive(serde::Deserialize)]
    #[allow(dead_code)]
    struct EvalResult {
        note: String,
        #[serde(rename = "targetFreq")]
        target_freq: f64,
        #[serde(rename = "predictedFreq")]
        predicted_freq: f64,
        cents: f64,
    }

    const EVAL_JSON: &str = include_str!("../../../../golden/expected/NAF-FF-01/eval_0.json");

    #[test]
    fn eval_matches_golden_within_half_cent() {
        let (inst, tuning, params) = setup();
        let expected: Vec<EvalResult> = serde_json::from_str(EVAL_JSON).unwrap();

        for (i, exp) in expected.iter().enumerate() {
            let fingering = &tuning.fingerings[i];
            let predicted = predicted_frequency(&inst, fingering, &params);

            assert!(
                predicted.is_some(),
                "No predicted frequency for {} (target {}Hz)",
                exp.note, exp.target_freq
            );

            let pred = predicted.unwrap();
            let cent_dev = cents(exp.target_freq, pred);

            assert!(
                (cent_dev - exp.cents).abs() <= 0.5,
                "Cents deviation for {} (fingering {}): expected {:.4}, got {:.4} (diff {:.4})",
                exp.note, i, exp.cents, cent_dev, (cent_dev - exp.cents).abs()
            );
        }
    }

    #[test]
    fn error_vector_matches_golden() {
        let (inst, tuning, params) = setup();
        let expected: Vec<EvalResult> = serde_json::from_str(EVAL_JSON).unwrap();

        let errors = calculate_error_vector(&inst, &tuning.fingerings, &params);

        assert_eq!(errors.len(), expected.len());
        for (i, (err, exp)) in errors.iter().zip(expected.iter()).enumerate() {
            assert!(
                (err - exp.cents).abs() <= 0.5,
                "Error vector mismatch at index {}: expected {:.4}, got {:.4}",
                i, exp.cents, err
            );
        }
    }

    // ── Cents utility ───────────────────────────────────────────

    #[test]
    fn cents_octave_is_1200() {
        assert_abs_diff_eq!(cents(440.0, 880.0), 1200.0, epsilon = 1e-10);
    }

    #[test]
    fn cents_unison_is_zero() {
        assert_abs_diff_eq!(cents(440.0, 440.0), 0.0, epsilon = 1e-10);
    }
}
