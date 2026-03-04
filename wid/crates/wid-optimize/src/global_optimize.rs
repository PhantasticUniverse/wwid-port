//! Two-stage global optimization: DIRECT-C followed by BOBYQA refinement.
//!
//! Ports the pattern from Java `ObjectiveFunctionOptimizer.runDirect()` +
//! `runBobyqa()`:
//!
//! 1. **Stage 1**: DIRECT-C global search with 2x the budget and a
//!    target function value guard of 0.001
//! 2. **Stage 2**: BOBYQA local refinement from DIRECT-C's best point
//!
//! The DIRECT-C stage finds a good basin, then BOBYQA converges precisely.

use bobyqa_impl::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use direct_impl::{direct_minimize, direct_minimize_with_callback};
use wid_compile::{
    compile, get_hole_diameters, get_hole_geometry_position,
    set_hole_diameters, set_hole_geometry_position,
};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, compute_trust_radius, fingering_weights};

/// Constants matching Java `ObjectiveFunctionOptimizer`.
const DIRECT_CONVERGENCE_STANDALONE: f64 = 7.0e-8;
const DIRECT_TARGET_VALUE: f64 = 0.001;
const DIRECT_BUDGET_MULTIPLIER: usize = 2;

/// Result of a two-stage global optimization.
#[derive(Debug, Clone)]
pub struct GlobalOptResult {
    /// Best point found.
    pub point: Vec<f64>,
    /// Function value at that point.
    pub value: f64,
    /// Function calls used by the DIRECT-C stage.
    pub direct_evaluations: usize,
    /// Function calls used by the BOBYQA stage.
    pub bobyqa_evaluations: usize,
}

/// Two-stage global optimization: DIRECT-C global search then BOBYQA refinement.
///
/// This matches the Java `ObjectiveFunctionOptimizer` pattern for
/// `DIRECTOptimizer` type objectives:
/// - DIRECT-C with convergence threshold 7e-8, target value 0.001,
///   and 2x the max function calls budget
/// - BOBYQA refinement from DIRECT-C's best point
/// - Returns whichever stage found the better result
pub fn optimize_global(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial_point: &[f64],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    max_evaluations: usize,
) -> Option<GlobalOptResult> {
    let n = lower_bounds.len();
    if n < 1 {
        return None;
    }

    // Stage 1: DIRECT-C global search
    let direct_budget = DIRECT_BUDGET_MULTIPLIER * max_evaluations;
    let direct_result = direct_minimize(
        f,
        lower_bounds,
        upper_bounds,
        DIRECT_CONVERGENCE_STANDALONE,
        direct_budget,
        Some(DIRECT_TARGET_VALUE),
    );

    let (direct_point, direct_value, direct_evals) = match direct_result {
        Some(r) => (r.point, r.value, r.evaluations),
        None => (initial_point.to_vec(), f64::MAX, 0),
    };

    // Stage 2: BOBYQA refinement from DIRECT-C's best point
    let n_interp = 2 * n + 1;

    // Compute trust radius from bounds, matching Java
    // BaseObjectiveFunction.getInitialTrustRegionRadius():
    //   initial = max(0.1 * max_bound_range, min_half_bound_range)
    //   stopping = 1e-8 * initial
    let (initial_trust, stopping_trust) = compute_trust_radius(lower_bounds, upper_bounds);

    // Clamp DIRECT result to bounds (should already be within)
    let start: Vec<f64> = direct_point
        .iter()
        .enumerate()
        .map(|(i, &v)| v.clamp(lower_bounds[i], upper_bounds[i]))
        .collect();

    let mut bobyqa_evals = 0usize;
    let bobyqa_result = bobyqa_minimize(
        &mut |point: &[f64]| {
            bobyqa_evals += 1;
            f(point)
        },
        &start,
        lower_bounds,
        upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_evaluations,
    );

    // Return whichever stage found the better result
    match bobyqa_result {
        Some(br) => {
            if direct_value < br.value {
                Some(GlobalOptResult {
                    point: direct_point,
                    value: direct_value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            } else {
                Some(GlobalOptResult {
                    point: br.point,
                    value: br.value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            }
        }
        None => {
            if direct_evals > 0 {
                Some(GlobalOptResult {
                    point: direct_point,
                    value: direct_value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            } else {
                None
            }
        }
    }
}

/// Two-stage global optimization with progress callback and cancellation.
///
/// The callback receives `BobyqaProgress` updates during both the DIRECT-C
/// and BOBYQA stages. Return `false` from the callback to cancel.
pub fn optimize_global_with_progress(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial_point: &[f64],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    max_evaluations: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> Option<GlobalOptResult> {
    let n = lower_bounds.len();
    if n < 1 {
        return None;
    }

    // Stage 1: DIRECT-C global search with progress forwarding
    let direct_budget = DIRECT_BUDGET_MULTIPLIER * max_evaluations;
    let mut cancelled = false;
    let direct_result = direct_minimize_with_callback(
        f,
        lower_bounds,
        upper_bounds,
        DIRECT_CONVERGENCE_STANDALONE,
        direct_budget,
        Some(DIRECT_TARGET_VALUE),
        &mut |progress| {
            let keep = on_progress(BobyqaProgress {
                evaluations: progress.evaluations,
                best_value: progress.best_value,
            });
            if !keep {
                cancelled = true;
            }
            keep
        },
    );

    if cancelled {
        return direct_result.map(|r| GlobalOptResult {
            point: r.point,
            value: r.value,
            direct_evaluations: r.evaluations,
            bobyqa_evaluations: 0,
        });
    }

    let (direct_point, direct_value, direct_evals) = match direct_result {
        Some(r) => (r.point, r.value, r.evaluations),
        None => (initial_point.to_vec(), f64::MAX, 0),
    };

    // Stage 2: BOBYQA refinement with progress forwarding
    let n_interp = 2 * n + 1;
    let (initial_trust, stopping_trust) = compute_trust_radius(lower_bounds, upper_bounds);

    let start: Vec<f64> = direct_point
        .iter()
        .enumerate()
        .map(|(i, &v)| v.clamp(lower_bounds[i], upper_bounds[i]))
        .collect();

    let mut bobyqa_evals = 0usize;
    let bobyqa_result = bobyqa_minimize_with_callback(
        &mut |point: &[f64]| {
            bobyqa_evals += 1;
            f(point)
        },
        &start,
        lower_bounds,
        upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_evaluations,
        &mut |bp| {
            on_progress(BobyqaProgress {
                evaluations: direct_evals + bp.evaluations,
                best_value: bp.best_value.min(direct_value),
            })
        },
    );

    match bobyqa_result {
        Some(br) => {
            if direct_value < br.value {
                Some(GlobalOptResult {
                    point: direct_point,
                    value: direct_value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            } else {
                Some(GlobalOptResult {
                    point: br.point,
                    value: br.value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            }
        }
        None => {
            if direct_evals > 0 {
                Some(GlobalOptResult {
                    point: direct_point,
                    value: direct_value,
                    direct_evaluations: direct_evals,
                    bobyqa_evaluations: bobyqa_evals,
                })
            } else {
                None
            }
        }
    }
}

// ── Instrument-level global optimization wrappers ────────────────────

/// Max function calls for GlobalHoleObjectiveFunction (position + size).
const GLOBAL_HOLE_MAX_EVALS: usize = 40_000;

/// Max function calls for GlobalHolePositionObjectiveFunction (position only).
const GLOBAL_HOLE_POSITION_MAX_EVALS: usize = 30_000;

/// Get merged geometry vector: [position..., diameters...].
fn get_merged_geometry(raw: &InstrumentRaw) -> Vec<f64> {
    let mut geometry = get_hole_geometry_position(raw);
    geometry.extend(get_hole_diameters(raw));
    geometry
}

/// Set merged geometry vector.
///
/// Panics if `geometry.len() != 2 * n_holes + 1` (matching Java's
/// `DimensionMismatchException`).
fn set_merged_geometry(raw: &mut InstrumentRaw, geometry: &[f64]) {
    let n_holes = raw.holes.len();
    let n_position = n_holes + 1;
    let expected = n_position + n_holes;
    assert_eq!(
        geometry.len(), expected,
        "merged geometry length mismatch: expected {expected}, got {}",
        geometry.len()
    );
    set_hole_geometry_position(raw, &geometry[..n_position]);
    set_hole_diameters(raw, &geometry[n_position..]);
}

fn compute_norm(
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

/// Global optimization of hole position + size (DIRECT-C then BOBYQA).
///
/// Port of Java `GlobalHoleObjectiveFunction`.
pub fn optimize_global_holes_combined(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_global_holes_combined_with_progress(
        instrument, tuning, constraints, params, calc_params,
        &mut |_| true,
    )
}

/// Global optimization of hole position + size with progress callback.
pub fn optimize_global_holes_combined_with_progress(
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

    let raw_geometry = get_merged_geometry(instrument);
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

    let initial_norm = compute_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = optimize_global_with_progress(
        &mut |point: &[f64]| {
            set_merged_geometry(&mut work_inst, point);
            compute_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry,
        &lower_bounds,
        &upper_bounds,
        GLOBAL_HOLE_MAX_EVALS,
        on_progress,
    );

    match result {
        Some(opt_result) => {
            set_merged_geometry(instrument, &opt_result.point);
            OptimizationResult {
                initial_norm,
                final_norm: opt_result.value,
                evaluations: opt_result.direct_evaluations + opt_result.bobyqa_evaluations,
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

/// Global optimization of hole position only (DIRECT-C then BOBYQA).
///
/// Port of Java `GlobalHolePositionObjectiveFunction`.
pub fn optimize_global_holes_position(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_global_holes_position_with_progress(
        instrument, tuning, constraints, params, calc_params,
        &mut |_| true,
    )
}

/// Global optimization of hole position only with progress callback.
pub fn optimize_global_holes_position_with_progress(
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

    let raw_geometry = get_hole_geometry_position(instrument);
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

    let initial_norm = compute_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = optimize_global_with_progress(
        &mut |point: &[f64]| {
            set_hole_geometry_position(&mut work_inst, point);
            compute_norm(&work_inst, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry,
        &lower_bounds,
        &upper_bounds,
        GLOBAL_HOLE_POSITION_MAX_EVALS,
        on_progress,
    );

    match result {
        Some(opt_result) => {
            set_hole_geometry_position(instrument, &opt_result.point);
            OptimizationResult {
                initial_norm,
                final_norm: opt_result.value,
                evaluations: opt_result.direct_evaluations + opt_result.bobyqa_evaluations,
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

#[cfg(test)]
mod tests {
    use super::*;

    // ── Standard test function tests ────────────────────────────────────

    #[test]
    fn global_opt_rosenbrock_2d() {
        let result = optimize_global(
            &mut |x: &[f64]| {
                let t = x[0] * x[0] - x[1];
                100.0 * t * t + (x[0] - 1.0) * (x[0] - 1.0)
            },
            &[0.0, 0.0],
            &[-5.0, -5.0],
            &[5.0, 5.0],
            20000,
        ).unwrap();

        assert!(
            result.value < 0.001,
            "DIRECT+BOBYQA should converge on Rosenbrock: value={}",
            result.value
        );
        assert!(
            (result.point[0] - 1.0).abs() < 0.01,
            "x0 should be near 1: {}",
            result.point[0]
        );
        assert!(result.direct_evaluations > 0, "should have used DIRECT");
        assert!(result.bobyqa_evaluations > 0, "should have used BOBYQA");
    }

    #[test]
    fn global_opt_six_hump_camel() {
        let result = optimize_global(
            &mut |x: &[f64]| {
                let x1 = x[0]; let x2 = x[1];
                (4.0 - 2.1*x1*x1 + x1*x1*x1*x1/3.0) * x1*x1
                    + x1*x2
                    + (-4.0 + 4.0*x2*x2) * x2*x2
            },
            &[0.0, 0.0],
            &[-3.0, -2.0],
            &[3.0, 2.0],
            10000,
        ).unwrap();

        assert!(
            (result.value - (-1.0316)).abs() < 0.001,
            "should find global min ~ -1.0316: got {}",
            result.value
        );
    }

    #[test]
    fn global_opt_sphere_5d() {
        let result = optimize_global(
            &mut |x: &[f64]| x.iter().map(|xi| xi * xi).sum(),
            &[0.0; 5],
            &[-5.0; 5],
            &[5.0; 5],
            50000,
        ).unwrap();

        assert!(
            result.value < 1e-6,
            "DIRECT+BOBYQA should solve 5D sphere precisely: value={}",
            result.value
        );
    }
}

/// Golden fixture parity tests: DIRECT-C → BOBYQA global optimization
/// on Whistle instruments, compared against Java oracle outputs.
#[cfg(test)]
mod golden_tests {
    use super::*;
    use wid_eval::CalculatorParams;
    use wid_physics::TemperatureType;
    use wid_types::{Constraint, ConstraintType, parse_instrument_xml, parse_tuning_xml};

    const PVC_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/SamplePVC-Whistle.xml");
    const PVC_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/tunings/SamplePVC-tuning.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    fn merged_constraints() -> Constraints {
        // 7 position + 6 size constraints from DefaultHoleConstraints.xml
        let pos_lower = [0.2, 0.012, 0.012, 0.012, 0.012, 0.012, 0.012];
        let pos_upper = [0.7, 0.04, 0.04, 0.1, 0.04, 0.04, 0.2];

        let mut constraints = Vec::new();
        for i in 0..7 {
            constraints.push(Constraint {
                display_name: "position".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(pos_lower[i]),
                upper_bound: Some(pos_upper[i]),
            });
        }
        for _ in 0..6 {
            constraints.push(Constraint {
                display_name: "diameter".to_string(),
                category: "Hole size".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.004),
                upper_bound: Some(0.0091),
            });
        }

        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Hole size+spacing (global)".to_string(),
            objective_function_name: "GlobalHoleObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
            hole_groups: None,
        }
    }

    fn position_constraints() -> Constraints {
        // 7 position-only constraints from DefaultHoleConstraints.xml
        let pos_lower = [0.2, 0.012, 0.012, 0.012, 0.012, 0.012, 0.012];
        let pos_upper = [0.7, 0.04, 0.04, 0.1, 0.04, 0.04, 0.2];

        let mut constraints = Vec::new();
        for i in 0..7 {
            constraints.push(Constraint {
                display_name: "position".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(pos_lower[i]),
                upper_bound: Some(pos_upper[i]),
            });
        }

        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Hole spacing (global)".to_string(),
            objective_function_name: "GlobalHolePositionObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
            hole_groups: None,
        }
    }

    // Golden values from DIRECT-01 fixture (Java oracle)
    const GOLDEN_GLOBAL_HOLE_INITIAL_NORM: f64 = 15900.000398470573;
    const GOLDEN_GLOBAL_HOLE_FINAL_NORM: f64 = 1899.2452798664265;

    #[test]
    fn global_hole_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = compute_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );
        assert!(
            (norm - GOLDEN_GLOBAL_HOLE_INITIAL_NORM).abs() / GOLDEN_GLOBAL_HOLE_INITIAL_NORM < 0.001,
            "initial norm: expected {GOLDEN_GLOBAL_HOLE_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn global_hole_optimization_improves_significantly() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = merged_constraints();

        let result = optimize_global_holes_combined(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        // Must improve significantly (golden goes from ~15900 → ~1899)
        assert!(
            result.final_norm < result.initial_norm * 0.25,
            "global hole optimization should reduce norm by >75%: initial={}, final={}",
            result.initial_norm, result.final_norm
        );

        // Final norm within 3x of golden (allow slack since DIRECT-C is
        // deterministic but our implementation may take different paths)
        assert!(
            result.final_norm < GOLDEN_GLOBAL_HOLE_FINAL_NORM * 3.0,
            "final norm should be in same ballpark as golden: expected ~{}, got {}",
            GOLDEN_GLOBAL_HOLE_FINAL_NORM, result.final_norm
        );
    }

    #[test]
    fn global_hole_uses_both_stages() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = merged_constraints();

        let result = optimize_global_holes_combined(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        // Should use a reasonable number of evaluations
        assert!(
            result.evaluations > 100,
            "should do substantial work: {} evaluations",
            result.evaluations
        );
    }

    #[test]
    fn global_hole_position_produces_result() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = position_constraints();

        let result = optimize_global_holes_position(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        // Should produce a result with evaluations
        assert!(
            result.evaluations > 100,
            "should do substantial work: {} evaluations",
            result.evaluations
        );

        // Final geometry should have correct dimensions
        assert_eq!(
            result.final_geometry.len(), 7,
            "position-only geometry should have 7 elements (N+1 for 6 holes)"
        );
    }

    #[test]
    fn global_hole_geometry_changes_instrument() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = merged_constraints();

        let geometry_before = get_merged_geometry(&inst);

        let result = optimize_global_holes_combined(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        let geometry_after = get_merged_geometry(&inst);

        // Geometry should have changed
        assert_ne!(
            geometry_before, geometry_after,
            "optimization should modify instrument geometry"
        );

        // Final geometry should match what's returned in the result (within
        // floating-point roundtrip tolerance from position↔spacing conversion)
        for (i, (a, b)) in result.final_geometry.iter().zip(geometry_after.iter()).enumerate() {
            assert!(
                (a - b).abs() < 1e-14,
                "geometry[{i}] mismatch: result={a}, instrument={b}"
            );
        }
    }

    #[test]
    fn global_hole_with_progress_works() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = merged_constraints();

        let mut progress_count = 0;
        let result = optimize_global_holes_combined_with_progress(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
            &mut |_p| {
                progress_count += 1;
                true
            },
        );

        assert!(progress_count > 0, "should receive progress callbacks");
        assert!(
            result.final_norm < result.initial_norm,
            "should improve: initial={}, final={}",
            result.initial_norm, result.final_norm
        );
    }
}
