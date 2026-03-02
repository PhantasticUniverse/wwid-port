//! Fipple factor calibration.
//!
//! Calibrates the mouthpiece fipple factor using the lowest-frequency note
//! from the tuning. Uses Brent's univariate minimizer.
//!
//! Port of `FippleFactorObjectiveFunction` from WIDesigner.

use wid_compile::{compile, get_fipple_factor, set_fipple_factor};
use wid_eval::calculate_error_vector;
use wid_physics::PhysicalParameters;
use wid_types::{Fingering, InstrumentRaw, Tuning};

use crate::{CalibrationResult, brent_min, calc_norm, fingering_weights};

/// Default fipple factor bounds for NAF study model.
pub const DEFAULT_FF_LOWER: f64 = 0.2;
pub const DEFAULT_FF_UPPER: f64 = 1.5;

/// Calibrate the fipple factor to minimize cents deviation at the lowest note.
///
/// Only the lowest-frequency fingering from the tuning is used. The fipple
/// factor is adjusted within [lower_bound, upper_bound] using Brent minimization.
///
/// The instrument is modified in place with the optimal fipple factor.
pub fn calibrate_fipple(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    lower_bound: f64,
    upper_bound: f64,
) -> CalibrationResult {
    // Extract the lowest-frequency fingering (matching Java getLowestNote)
    let lowest = get_lowest_fingering(&tuning.fingerings);
    let lowest_fingerings = vec![lowest.clone()];
    let weights = fingering_weights(&lowest_fingerings);

    // Get initial fipple factor
    let initial_ff = get_fipple_factor(instrument).unwrap_or(0.75);

    // Compute initial norm
    let initial_norm = evaluate_norm(instrument, &lowest_fingerings, &weights, params);

    // Clamp start point to bounds
    let start = initial_ff.clamp(lower_bound, upper_bound);

    // Clone the instrument for the objective closure (we mutate it on each eval)
    let mut work_inst = instrument.clone();

    let mut eval_count = 0_usize;
    let result = brent_min::brent_minimize(
        &mut |ff| {
            eval_count += 1;
            set_fipple_factor(&mut work_inst, ff);
            evaluate_norm(&work_inst, &lowest_fingerings, &weights, params)
        },
        lower_bound,
        upper_bound,
        start,
        1e-6,  // rel_tol, matching Java BrentOptimizer(1e-6, 1e-14)
        1e-14, // abs_tol
        50_000,
    );

    let (final_ff, final_norm) = result.unwrap_or((initial_ff, initial_norm));

    // Apply the optimal fipple factor to the original instrument
    set_fipple_factor(instrument, final_ff);

    CalibrationResult {
        initial_fipple_factor: initial_ff,
        final_fipple_factor: final_ff,
        initial_norm,
        final_norm,
    }
}

/// Find the fingering with the lowest target frequency.
fn get_lowest_fingering(fingerings: &[Fingering]) -> &Fingering {
    fingerings
        .iter()
        .filter(|f| f.note.frequency.is_some())
        .min_by(|a, b| {
            a.note
                .frequency
                .unwrap()
                .partial_cmp(&b.note.frequency.unwrap())
                .unwrap()
        })
        .expect("tuning must have at least one fingering with a frequency")
}

/// Evaluate the weighted norm for the given fingerings.
fn evaluate_norm(
    instrument: &InstrumentRaw,
    fingerings: &[Fingering],
    weights: &[i32],
    params: &PhysicalParameters,
) -> f64 {
    let compiled = match compile(instrument) {
        Ok(c) => c,
        Err(_) => return f64::MAX,
    };
    let errors = calculate_error_vector(&compiled, fingerings, params);
    calc_norm(&errors, weights)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use wid_physics::TemperatureType;
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    // ── Test data ────────────────────────────────────────────────

    const NAF_0HOLE_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-FF-02_instrument_0hole.xml");
    const TUNING_0HOLE_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-FF-02_tuning_0hole.xml");
    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // ── Golden calibration fixtures ─────────────────────────────

    // NAF-FF-02/calibrate_0.json
    const FF02_INITIAL_FF: f64 = 0.75;
    const FF02_FINAL_FF: f64 = 0.26617358020430504;
    const FF02_INITIAL_NORM: f64 = 97743.0548874793;
    const FF02_FINAL_NORM: f64 = 1.1051632673390108e-4;

    // NAF-FF-03/calibrate_0.json
    const FF03_INITIAL_FF: f64 = 0.75;
    const FF03_FINAL_FF: f64 = 0.2744461496645853;
    const FF03_INITIAL_NORM: f64 = 90010.41593410687;
    const FF03_FINAL_NORM: f64 = 8.940864613648533e-4;

    // ── NAF-FF-02: 0-hole calibration ───────────────────────────

    #[test]
    fn ff02_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_0HOLE_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(&inst, &tuning.fingerings, &weights, &params);
        assert!(
            (norm - FF02_INITIAL_NORM).abs() / FF02_INITIAL_NORM < 0.01,
            "initial norm: expected {FF02_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn ff02_calibration_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_0HOLE_XML).unwrap();
        let params = default_params();

        let result = calibrate_fipple(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_FF_LOWER,
            DEFAULT_FF_UPPER,
        );

        // Fipple factor (tolerance 1e-4: slight shift due to ≤0.5 cent eval tolerance)
        assert_abs_diff_eq!(result.initial_fipple_factor, FF02_INITIAL_FF, epsilon = 1e-10);
        assert!(
            (result.final_fipple_factor - FF02_FINAL_FF).abs() < 1e-4,
            "final FF: expected {FF02_FINAL_FF}, got {}, diff {}",
            result.final_fipple_factor,
            (result.final_fipple_factor - FF02_FINAL_FF).abs()
        );

        // Norms
        assert!(
            (result.initial_norm - FF02_INITIAL_NORM).abs() / FF02_INITIAL_NORM < 0.01,
            "initial norm: expected {FF02_INITIAL_NORM}, got {}",
            result.initial_norm
        );
        assert!(
            result.final_norm < FF02_FINAL_NORM * 2.0 + 1e-3,
            "final norm too high: expected ~{FF02_FINAL_NORM}, got {}",
            result.final_norm
        );
    }

    #[test]
    fn ff02_post_calibration_eval_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_0HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_0HOLE_XML).unwrap();
        let params = default_params();

        calibrate_fipple(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_FF_LOWER,
            DEFAULT_FF_UPPER,
        );

        // After calibration, eval the single note
        // Golden eval_1.json: predictedFreq=369.99666945740086, cents=0.010512674575668225
        let compiled = compile(&inst).unwrap();
        let errors = calculate_error_vector(&compiled, &tuning.fingerings, &params);
        assert!(
            errors[0].abs() < 1.0,
            "post-calibration cents error too high: {}",
            errors[0]
        );
    }

    // ── NAF-FF-03: 6-hole calibration ───────────────────────────

    #[test]
    fn ff03_calibration_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let params = default_params();

        let result = calibrate_fipple(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_FF_LOWER,
            DEFAULT_FF_UPPER,
        );

        // Fipple factor (tolerance 1e-4: slight shift due to ≤0.5 cent eval tolerance)
        assert_abs_diff_eq!(result.initial_fipple_factor, FF03_INITIAL_FF, epsilon = 1e-10);
        assert!(
            (result.final_fipple_factor - FF03_FINAL_FF).abs() < 1e-4,
            "final FF: expected {FF03_FINAL_FF}, got {}, diff {}",
            result.final_fipple_factor,
            (result.final_fipple_factor - FF03_FINAL_FF).abs()
        );

        // Norms
        assert!(
            (result.initial_norm - FF03_INITIAL_NORM).abs() / FF03_INITIAL_NORM < 0.01,
            "initial norm: expected {FF03_INITIAL_NORM}, got {}",
            result.initial_norm
        );
        assert!(
            result.final_norm < FF03_FINAL_NORM * 2.0 + 1e-3,
            "final norm too high: expected ~{FF03_FINAL_NORM}, got {}",
            result.final_norm
        );
    }

    #[test]
    fn ff03_post_calibration_eval_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let params = default_params();

        calibrate_fipple(
            &mut inst,
            &tuning,
            &params,
            DEFAULT_FF_LOWER,
            DEFAULT_FF_UPPER,
        );

        // After calibration, evaluate all 15 fingerings
        // Golden eval_0.json: first note F#4 has cents=-0.02990
        let compiled = compile(&inst).unwrap();
        let errors = calculate_error_vector(&compiled, &tuning.fingerings, &params);

        // F#4 (all-closed) should be near zero after calibration
        assert!(
            errors[0].abs() < 1.0,
            "F#4 post-calibration error: {}",
            errors[0]
        );

        // Load golden eval results for comparison
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

        let golden: Vec<EvalResult> = serde_json::from_str(include_str!(
            "../../../../golden/expected/NAF-FF-03/eval_0.json"
        ))
        .unwrap();

        for (i, (err, exp)) in errors.iter().zip(golden.iter()).enumerate() {
            assert!(
                (err - exp.cents).abs() < 1.0,
                "fingering {i} ({}): expected {:.2} cents, got {:.2} cents",
                exp.note,
                exp.cents,
                err
            );
        }
    }
}
