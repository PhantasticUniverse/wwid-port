//! HoleGroupFromTop geometry optimization.
//!
//! Optimizes bore length, grouped hole positions, and hole diameters
//! simultaneously using BOBYQA. This is the NAF grouped-hole optimization
//! where holes within a group share equal spacing.
//!
//! Port of WIDesigner's `HoleGroupFromTopObjectiveFunction`, which is a
//! `MergedObjectiveFunction` combining `HoleGroupPositionFromTopObjectiveFunction`
//! and `HoleSizeObjectiveFunction`.
//!
//! # Geometry vector layout (for 6-hole NAF with groups [[0,1,2],[3,4,5]])
//!
//! | Index | Meaning                                    | Unit          |
//! |-------|--------------------------------------------|---------------|
//! | `0`   | Bore end position                          | metres        |
//! | `1`   | Top hole ratio (fraction of bore from top) | dimensionless |
//! | `2`   | Upper group spacing                        | metres        |
//! | `3`   | Inter-group gap                            | metres        |
//! | `4`   | Lower group spacing                        | metres        |
//! | `5..10`| Hole diameters (top→bottom)               | metres        |
//!
//! Total dimensions: `n_position_dims + n_holes` (11 for 6-hole 2-group NAF).

use bobyqa_impl::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use wid_compile::{
    compile, get_hole_group_geometry_from_top, set_hole_group_geometry_from_top,
};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, fingering_weights};

/// Default hole groups for 6-hole NAF: two groups of 3.
pub const DEFAULT_GROUPS_6: &[&[u32]] = &[&[0, 1, 2], &[3, 4, 5]];

/// Default hole groups for 7-hole NAF: two groups of 3 + one singleton.
pub const DEFAULT_GROUPS_7: &[&[u32]] = &[&[0, 1, 2], &[3, 4, 5], &[6]];

/// Get the default hole groups for a given number of holes.
///
/// Matches Java `NafStudyModel.setDefaultHoleGroups()`:
/// - 6 holes: `[[0,1,2],[3,4,5]]`
/// - 7 holes: `[[0,1,2],[3,4,5],[6]]`
/// - Other: each hole in its own group (degenerates to ungrouped)
pub fn default_hole_groups(n_holes: usize) -> Vec<Vec<u32>> {
    match n_holes {
        6 => DEFAULT_GROUPS_6
            .iter()
            .map(|g| g.to_vec())
            .collect(),
        7 => DEFAULT_GROUPS_7
            .iter()
            .map(|g| g.to_vec())
            .collect(),
        _ => (0..n_holes as u32).map(|i| vec![i]).collect(),
    }
}

/// Optimize grouped hole positions and diameters using BOBYQA.
///
/// The instrument is modified in place with the optimized geometry.
pub fn optimize_hole_group(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_hole_group_impl(instrument, tuning, constraints, params, calc_params, None)
}

/// Like [`optimize_hole_group`], but with a progress callback.
pub fn optimize_hole_group_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_group_impl(
        instrument,
        tuning,
        constraints,
        params,
        calc_params,
        Some(on_progress),
    )
}

fn optimize_hole_group_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();

    // Get hole groups from constraints, or use defaults
    let hole_groups = constraints
        .hole_groups_array()
        .unwrap_or_else(|| default_hole_groups(n_holes));

    let weights = fingering_weights(&tuning.fingerings);
    let lower_bounds = constraints.lower_bounds();
    let upper_bounds = constraints.upper_bounds();
    let n_dims = lower_bounds.len();

    let raw_geometry = get_hole_group_geometry_from_top(instrument, &hole_groups);
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

    let initial_norm =
        evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    // Java HoleGroupFromTopObjectiveFunction overrides trust radius to 10.0 / 1e-8
    // (not the bounds-based default from BaseObjectiveFunction).
    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = crate::max_evaluations(n_dims);
    let n_interp = 2 * n_dims + 1;

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();
    let groups = hole_groups.clone();

    let result = match on_progress {
        Some(cb) => bobyqa_minimize_with_callback(
            &mut |point: &[f64]| {
                set_hole_group_geometry_from_top(&mut work_inst, point, &groups);
                evaluate_norm(&work_inst, &fingerings, &weights, params, calc_params)
            },
            &initial_geometry,
            &lower_bounds,
            &upper_bounds,
            n_interp,
            initial_trust,
            stopping_trust,
            max_eval,
            cb,
        ),
        None => bobyqa_minimize(
            &mut |point: &[f64]| {
                set_hole_group_geometry_from_top(&mut work_inst, point, &groups);
                evaluate_norm(&work_inst, &fingerings, &weights, params, calc_params)
            },
            &initial_geometry,
            &lower_bounds,
            &upper_bounds,
            n_interp,
            initial_trust,
            stopping_trust,
            max_eval,
        ),
    };

    match result {
        Some(opt_result) => {
            set_hole_group_geometry_from_top(instrument, &opt_result.point, &hole_groups);
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
    use wid_types::{parse_instrument_xml, parse_tuning_xml};

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    #[test]
    fn grouped_geometry_has_correct_length() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let groups = default_hole_groups(6);
        let geom = get_hole_group_geometry_from_top(&inst, &groups);
        // 5 position dims + 6 hole diameters = 11
        assert_eq!(geom.len(), 11);
    }

    #[test]
    fn grouped_optimization_reduces_norm() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let params = default_params();

        // Build constraints with proper bounds for the 11-dim grouped problem
        let groups = default_hole_groups(6);
        let constraints = make_grouped_constraints(&groups);

        let result = optimize_hole_group(
            &mut inst,
            &tuning,
            &constraints,
            &params,
            &CalculatorParams::NAF,
        );

        assert!(
            result.final_norm < result.initial_norm,
            "grouped optimization should reduce norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm,
        );
    }

    #[test]
    fn default_groups_match_java() {
        let g6 = default_hole_groups(6);
        assert_eq!(g6, vec![vec![0, 1, 2], vec![3, 4, 5]]);

        let g7 = default_hole_groups(7);
        assert_eq!(g7, vec![vec![0, 1, 2], vec![3, 4, 5], vec![6]]);

        let g4 = default_hole_groups(4);
        assert_eq!(g4, vec![vec![0], vec![1], vec![2], vec![3]]);
    }

    /// Build constraints for 6-hole 2-group NAF (11 dimensions).
    ///
    /// Bounds from NafStudyModel.java defaults for HoleGroupFromTopObjectiveFunction.
    fn make_grouped_constraints(groups: &[Vec<u32>]) -> Constraints {
        use wid_types::{Constraint, ConstraintType};

        let mut constraints = Vec::new();

        // Position constraints (5 dims for 6-hole 2-group):
        // [bore_length, top_ratio, upper_spacing, inter_group_gap, lower_spacing]
        let pos_lower = [0.1905, 0.25, 0.0127, 0.02032, 0.0127];
        let pos_upper = [0.6985, 0.5, 0.03175, 0.06985, 0.03175];
        let pos_types = [
            ConstraintType::DIMENSIONAL,
            ConstraintType::DIMENSIONLESS,
            ConstraintType::DIMENSIONAL,
            ConstraintType::DIMENSIONAL,
            ConstraintType::DIMENSIONAL,
        ];

        for i in 0..5 {
            constraints.push(Constraint {
                display_name: format!("position_{i}"),
                category: "Hole position".to_string(),
                constraint_type: pos_types[i],
                lower_bound: Some(pos_lower[i]),
                upper_bound: Some(pos_upper[i]),
            });
        }

        // Size constraints (6 dims): hole diameters
        for _ in 0..6 {
            constraints.push(Constraint {
                display_name: "diameter".to_string(),
                category: "Hole size".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.0015875),
                upper_bound: Some(0.0127),
            });
        }

        let mut c = Constraints {
            name: "Default".to_string(),
            objective_display_name: "Grouped-hole position & size".to_string(),
            objective_function_name: "HoleGroupFromTopObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
            hole_groups: None,
        };
        c.set_hole_groups(groups.to_vec());
        c
    }
}

#[cfg(test)]
mod golden_tests {
    use super::*;
    use wid_physics::TemperatureType;
    use wid_types::{parse_constraints_xml, parse_instrument_xml, parse_tuning_xml};

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );
    const CONSTRAINTS_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleGroupFromTopObjectiveFunction/6/2-group_1.25_max_spacing.xml"
    );

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    // Golden: NAF-GRP-01/optimize_0.json
    const GOLDEN_INITIAL_NORM: f64 = 1324815.0036351634;
    const GOLDEN_FINAL_NORM: f64 = 1001.8543896629747;
    const GOLDEN_EVALUATIONS: usize = 1767;
    const GOLDEN_INITIAL_GEOMETRY: [f64; 11] = [
        0.3248902169679828,
        0.26393387003800606,
        0.020849751716983264,
        0.04085938293871649,
        0.028659342615868957,
        0.0057100938065062215,
        0.006327228446346466,
        0.006056222214560144,
        0.007836036154750887,
        0.007616195298537355,
        0.007846589456097008,
    ];
    const GOLDEN_FINAL_GEOMETRY: [f64; 11] = [
        0.39518501192871386,
        0.25,
        0.029602107664897924,
        0.047728746168149094,
        0.03174999999999999,
        0.005078961832289208,
        0.00547512624178228,
        0.005204487881536722,
        0.006088884695009846,
        0.0061563121145014185,
        0.006144268700964851,
    ];

    #[test]
    fn grp01_initial_geometry_matches_golden() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let groups = constraints.hole_groups_array().unwrap();
        let geom = get_hole_group_geometry_from_top(&inst, &groups);

        assert_eq!(geom.len(), 11);
        for i in 0..11 {
            assert!(
                (geom[i] - GOLDEN_INITIAL_GEOMETRY[i]).abs() < 1e-10,
                "geometry[{i}]: expected {}, got {}",
                GOLDEN_INITIAL_GEOMETRY[i],
                geom[i]
            );
        }
    }

    #[test]
    fn grp01_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::NAF);
        let rel_err = (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM;
        assert!(
            rel_err < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}, rel_err {rel_err}"
        );
    }

    #[test]
    fn grp01_optimization_matches_golden() {
        let mut inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        let tuning = parse_tuning_xml(TUNING_6HOLE_XML).unwrap();
        let constraints = parse_constraints_xml(CONSTRAINTS_XML).unwrap();
        let params = default_params();

        let result = optimize_hole_group(
            &mut inst,
            &tuning,
            &constraints,
            &params,
            &CalculatorParams::NAF,
        );

        // Final norm should be close to golden
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 1.5,
            "final norm too high: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );

        // Geometry elements within tolerance
        for i in 0..11 {
            assert!(
                (result.final_geometry[i] - GOLDEN_FINAL_GEOMETRY[i]).abs() < 5e-3,
                "final_geometry[{i}]: expected {}, got {}, diff {}",
                GOLDEN_FINAL_GEOMETRY[i],
                result.final_geometry[i],
                (result.final_geometry[i] - GOLDEN_FINAL_GEOMETRY[i]).abs()
            );
        }

        // Evaluation count in reasonable range
        let eval_ratio = result.evaluations as f64 / GOLDEN_EVALUATIONS as f64;
        assert!(
            (0.5..2.0).contains(&eval_ratio),
            "evaluations: {} (golden: {GOLDEN_EVALUATIONS}, ratio {eval_ratio:.2})",
            result.evaluations
        );
    }
}
