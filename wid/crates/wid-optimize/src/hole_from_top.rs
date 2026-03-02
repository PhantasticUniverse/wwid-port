//! HoleFromTop geometry optimization.
//!
//! Optimizes bore length, hole positions, and hole diameters simultaneously
//! using BOBYQA. This is the primary multi-variable optimization for NAF
//! instruments.
//!
//! Port of WIDesigner's `HoleFromTopObjectiveFunction`, which is a
//! `MergedObjectiveFunction` combining `HolePositionFromTopObjectiveFunction`
//! and `HoleSizeObjectiveFunction`.
//!
//! # Geometry vector layout (for N holes)
//!
//! | Index     | Meaning                                   | Unit        |
//! |-----------|-------------------------------------------|-------------|
//! | `0`       | Bore end position                         | metres      |
//! | `1`       | Top hole position as fraction of bore     | dimensionless |
//! | `2..N`    | Inter-hole spacings (top→bottom)          | metres      |
//! | `N+1..2N` | Hole diameters (top→bottom)               | metres      |
//!
//! Total dimensions: `2N + 1` (e.g. 13 for a 6-hole NAF).
//!
//! # Trust region parameters
//!
//! Matching the Java `HoleFromTopObjectiveFunction`:
//! - Initial trust region radius: 10.0
//! - Stopping trust region radius: 1e-8
//! - Max evaluations: `20000 + (n_dims - 1) * 5000`

use bobyqa::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use wid_compile::{compile, get_hole_geometry_from_top, set_hole_geometry_from_top};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, fingering_weights};

/// Optimize hole positions and diameters using BOBYQA.
///
/// The instrument is modified in place with the optimized geometry.
/// Returns an [`OptimizationResult`] with initial/final norms, geometry,
/// and evaluation count.
///
/// # Arguments
///
/// * `instrument` — The instrument to optimize (modified in place).
/// * `tuning` — Target tuning with fingerings and frequencies.
/// * `constraints` — Optimization bounds (from constraints XML).
/// * `params` — Physical parameters (temperature, humidity).
pub fn optimize_holes(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower_bounds = constraints.lower_bounds();
    let upper_bounds = constraints.upper_bounds();
    let n_dims = lower_bounds.len();

    // Get initial geometry and clamp to bounds (matching Java getInitialPoint)
    let raw_geometry = get_hole_geometry_from_top(instrument);
    let initial_geometry: Vec<f64> = raw_geometry
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            if i < lower_bounds.len() {
                v.clamp(lower_bounds[i], upper_bounds[i])
            } else {
                v
            }
        })
        .collect();

    // Compute initial norm
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    // Trust region parameters (matching Java HoleFromTopObjectiveFunction)
    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = 20000 + (n_dims.saturating_sub(1)) * 5000;
    let n_interp = 2 * n_dims + 1;

    // Clone instrument for the objective closure
    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = bobyqa_minimize(
        &mut |point: &[f64]| {
            set_hole_geometry_from_top(&mut work_inst, point);
            evaluate_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry,
        &lower_bounds,
        &upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_eval,
    );

    match result {
        Some(opt_result) => {
            // Apply the optimal geometry to the original instrument
            set_hole_geometry_from_top(instrument, &opt_result.point);

            OptimizationResult {
                initial_norm,
                final_norm: opt_result.value,
                evaluations: opt_result.evaluations,
                initial_geometry,
                final_geometry: opt_result.point,
            }
        }
        None => {
            // BOBYQA failed — return initial state unchanged
            OptimizationResult {
                initial_norm,
                final_norm: initial_norm,
                evaluations: 0,
                initial_geometry: initial_geometry.clone(),
                final_geometry: initial_geometry,
            }
        }
    }
}

/// Like [`optimize_holes`], but with a progress callback for monitoring and cancellation.
///
/// The `on_progress` callback receives a [`BobyqaProgress`] and should return
/// `true` to continue or `false` to cancel. On cancellation, the best result
/// found so far is applied to the instrument.
pub fn optimize_holes_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower_bounds = constraints.lower_bounds();
    let upper_bounds = constraints.upper_bounds();
    let n_dims = lower_bounds.len();

    let raw_geometry = get_hole_geometry_from_top(instrument);
    let initial_geometry: Vec<f64> = raw_geometry
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            if i < lower_bounds.len() {
                v.clamp(lower_bounds[i], upper_bounds[i])
            } else {
                v
            }
        })
        .collect();

    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = 20000 + (n_dims.saturating_sub(1)) * 5000;
    let n_interp = 2 * n_dims + 1;

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = bobyqa_minimize_with_callback(
        &mut |point: &[f64]| {
            set_hole_geometry_from_top(&mut work_inst, point);
            evaluate_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry,
        &lower_bounds,
        &upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_eval,
        on_progress,
    );

    match result {
        Some(opt_result) => {
            set_hole_geometry_from_top(instrument, &opt_result.point);
            OptimizationResult {
                initial_norm,
                final_norm: opt_result.value,
                evaluations: opt_result.evaluations,
                initial_geometry,
                final_geometry: opt_result.point,
            }
        }
        None => OptimizationResult {
            initial_norm,
            final_norm: initial_norm,
            evaluations: 0,
            initial_geometry: initial_geometry.clone(),
            final_geometry: initial_geometry,
        },
    }
}

/// Evaluate the weighted norm for the given fingerings.
fn evaluate_norm(
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
    use wid_types::{parse_constraints_xml, parse_instrument_xml, parse_tuning_xml};

    // ── Test data ──────────────────────────────────────────────────

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );
    const TUNING_WEIGHT0_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-OPT-02_tuning_weight0.xml");
    const CONSTRAINTS_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.25_max_hole_spacing.xml"
    );

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // ── Golden reference values ────────────────────────────────────

    // NAF-OPT-01/optimize_0.json
    const OPT01_INITIAL_NORM: f64 = 1324815.0036351632;
    const OPT01_FINAL_NORM: f64 = 975.1391419747237;
    const OPT01_EVALUATIONS: usize = 2018;
    const OPT01_INITIAL_GEOMETRY: [f64; 13] = [
        0.3248902169679828,
        0.26393387003800606,
        0.02084975171698325,
        0.020849751716983278,
        0.04085938293871649,
        0.02865934261586897,
        0.028659342615868943,
        0.0057100938065062215,
        0.006327228446346466,
        0.006056222214560144,
        0.007836036154750887,
        0.007616195298537355,
        0.007846589456097008,
    ];
    const OPT01_FINAL_GEOMETRY: [f64; 13] = [
        0.3956490026450572,
        0.25,
        0.02797592451081435,
        0.03175,
        0.04694691681877369,
        0.03175,
        0.03175,
        0.005101709351600449,
        0.005411456606325593,
        0.005222196148323513,
        0.006089570003706119,
        0.006160154293949429,
        0.006139368008863565,
    ];

    // NAF-OPT-02/optimize_0.json
    const OPT02_INITIAL_NORM: f64 = 1244615.3018577166;
    const OPT02_FINAL_NORM: f64 = 963.9340165496156;
    const OPT02_EVALUATIONS: usize = 1409;
    const OPT02_FINAL_GEOMETRY: [f64; 13] = [
        0.39575287131735376,
        0.25,
        0.027926919771172162,
        0.03175,
        0.04708139344665635,
        0.03175,
        0.03175,
        0.005135198659710525,
        0.0054096216393787885,
        0.005231725998132534,
        0.006098947196250168,
        0.006168541552889942,
        0.006144692634613424,
    ];

    // ── NAF-OPT-01: Full optimization (all weights=1) ─────────────

    #[test]
    fn opt01_initial_geometry_matches_golden() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let geometry = get_hole_geometry_from_top(&inst);

        assert_eq!(geometry.len(), 13, "expected 13-element geometry for 6-hole NAF");
        for i in 0..13 {
            assert!(
                (geometry[i] - OPT01_INITIAL_GEOMETRY[i]).abs() < 1e-10,
                "geometry[{i}]: expected {}, got {}, diff {}",
                OPT01_INITIAL_GEOMETRY[i],
                geometry[i],
                (geometry[i] - OPT01_INITIAL_GEOMETRY[i]).abs()
            );
        }
    }

    #[test]
    fn opt01_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::NAF);
        let rel_err = (norm - OPT01_INITIAL_NORM).abs() / OPT01_INITIAL_NORM;
        assert!(
            rel_err < 0.01,
            "initial norm: expected {OPT01_INITIAL_NORM}, got {norm}, rel_err {rel_err}"
        );
    }

    #[test]
    fn opt01_optimization_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let params = default_params();

        let result = optimize_holes(&mut inst, &tuning, &constraints, &params, &CalculatorParams::NAF);

        // Final norm must be low (optimization succeeded)
        assert!(
            result.final_norm < OPT01_FINAL_NORM * 1.2,
            "final norm too high: expected ~{OPT01_FINAL_NORM}, got {}",
            result.final_norm
        );

        // Check geometry — each element within tolerance
        for i in 0..13 {
            assert!(
                (result.final_geometry[i] - OPT01_FINAL_GEOMETRY[i]).abs() < 5e-3,
                "final_geometry[{i}]: expected {}, got {}, diff {}",
                OPT01_FINAL_GEOMETRY[i],
                result.final_geometry[i],
                (result.final_geometry[i] - OPT01_FINAL_GEOMETRY[i]).abs()
            );
        }

        // Evaluation count within reasonable range
        let eval_ratio = result.evaluations as f64 / OPT01_EVALUATIONS as f64;
        assert!(
            (0.5..2.0).contains(&eval_ratio),
            "evaluations: {} (golden: {OPT01_EVALUATIONS}, ratio {eval_ratio:.2})",
            result.evaluations
        );
    }

    #[test]
    fn opt01_post_optimization_eval_within_tolerance() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let params = default_params();

        optimize_holes(&mut inst, &tuning, &constraints, &params, &CalculatorParams::NAF);

        // After optimization, evaluate all 15 fingerings
        let compiled = compile(&inst).unwrap();
        let errors = calculate_error_vector(&compiled, &tuning.fingerings, &params, &CalculatorParams::NAF);

        // Load golden eval results
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
            "../../../../golden/expected/NAF-OPT-01/eval_1.json"
        ))
        .unwrap();

        // All notes should be within ~20 cents after optimization
        for (i, err) in errors.iter().enumerate() {
            assert!(
                err.abs() < 25.0,
                "fingering {i} ({}): {:.1} cents (too high after optimization)",
                golden[i].note,
                err
            );
        }
    }

    // ── NAF-OPT-02: Optimization with weight=0 exclusion ──────────

    #[test]
    fn opt02_initial_norm_differs_from_opt01() {
        // OPT-02 has weight=0 for G5(open), so initial norm is different
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_WEIGHT0_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::NAF);
        let rel_err = (norm - OPT02_INITIAL_NORM).abs() / OPT02_INITIAL_NORM;
        assert!(
            rel_err < 0.01,
            "initial norm: expected {OPT02_INITIAL_NORM}, got {norm}, rel_err {rel_err}"
        );
    }

    #[test]
    fn opt02_optimization_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_WEIGHT0_XML).unwrap();
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let params = default_params();

        let result = optimize_holes(&mut inst, &tuning, &constraints, &params, &CalculatorParams::NAF);

        // Final norm must be low
        assert!(
            result.final_norm < OPT02_FINAL_NORM * 1.2,
            "final norm too high: expected ~{OPT02_FINAL_NORM}, got {}",
            result.final_norm
        );

        // Fewer evaluations than OPT-01 (one fewer weighted note)
        // Allow generous tolerance since optimizer path can diverge
        let eval_ratio = result.evaluations as f64 / OPT02_EVALUATIONS as f64;
        assert!(
            (0.5..2.0).contains(&eval_ratio),
            "evaluations: {} (golden: {OPT02_EVALUATIONS}, ratio {eval_ratio:.2})",
            result.evaluations
        );

        // Check key geometry elements
        for i in 0..13 {
            assert!(
                (result.final_geometry[i] - OPT02_FINAL_GEOMETRY[i]).abs() < 5e-3,
                "final_geometry[{i}]: expected {}, got {}, diff {}",
                OPT02_FINAL_GEOMETRY[i],
                result.final_geometry[i],
                (result.final_geometry[i] - OPT02_FINAL_GEOMETRY[i]).abs()
            );
        }
    }

    // ── Constraints bounds verification ──────────────────────────

    #[test]
    fn constraints_bounds_have_correct_length() {
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        assert_eq!(constraints.total_constraints(), 13, "expected 13 constraints for 6-hole NAF");
        assert_eq!(constraints.lower_bounds().len(), 13);
        assert_eq!(constraints.upper_bounds().len(), 13);
    }

    #[test]
    fn constraints_bounds_match_xml_values() {
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let lo = constraints.lower_bounds();
        let hi = constraints.upper_bounds();

        // Bore length
        assert!((lo[0] - 0.1905).abs() < 1e-10);
        assert!((hi[0] - 0.6985).abs() < 1e-10);

        // Top hole fraction (dimensionless)
        assert!((lo[1] - 0.25).abs() < 1e-10);
        assert!((hi[1] - 0.5).abs() < 1e-10);

        // H4→H3 spacing (relaxed upper bound)
        assert!((lo[4] - 0.02032).abs() < 1e-10);
        assert!((hi[4] - 0.06985).abs() < 1e-10);

        // H6 diameter (smallest lower bound)
        assert!((lo[7] - 0.0015875).abs() < 1e-10);
        assert!((hi[7] - 0.0127).abs() < 1e-10);
    }
}
