//! Hole position optimization (bore length + spacings).
//!
//! (N+1)-dimensional BOBYQA optimization of bore end position and
//! inter-hole spacings. Uses PRESERVE_TAPER bore adjustment.
//!
//! Port of WIDesigner's `HolePositionObjectiveFunction`.

use bobyqa_impl::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use wid_compile::{compile, get_hole_geometry_position, set_hole_geometry_position};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, fingering_weights};

/// Optimize hole positions using BOBYQA.
///
/// The instrument is modified in place with the optimized geometry.
pub fn optimize_hole_position(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_hole_position_impl(instrument, tuning, constraints, params, calc_params, None)
}

/// Like [`optimize_hole_position`], but with a progress callback.
pub fn optimize_hole_position_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_position_impl(
        instrument,
        tuning,
        constraints,
        params,
        calc_params,
        Some(on_progress),
    )
}

fn optimize_hole_position_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower_bounds = constraints.lower_bounds();
    let upper_bounds = constraints.upper_bounds();
    let n_dims = lower_bounds.len();

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

    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let initial_trust = 10.0;
    let stopping_trust = 1e-8;
    let max_eval = 20000 + n_dims.saturating_sub(1) * 5000;
    let n_interp = 2 * n_dims + 1;

    let mut work_inst = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = match on_progress {
        Some(cb) => bobyqa_minimize_with_callback(
            &mut |point: &[f64]| {
                set_hole_geometry_position(&mut work_inst, point);
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
                set_hole_geometry_position(&mut work_inst, point);
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
            set_hole_geometry_position(instrument, &opt_result.point);
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
    use wid_types::{Constraint, ConstraintType, parse_instrument_xml, parse_tuning_xml};

    const PVC_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/SamplePVC-Whistle.xml");
    const PVC_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/tunings/SamplePVC-tuning.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    fn position_constraints() -> Constraints {
        // 7 position constraints from DefaultHoleConstraints.xml
        let lower_bounds = [0.2, 0.012, 0.012, 0.012, 0.012, 0.012, 0.012];
        let upper_bounds = [0.7, 0.04, 0.04, 0.1, 0.04, 0.04, 0.2];
        let constraints: Vec<Constraint> = lower_bounds
            .iter()
            .zip(upper_bounds.iter())
            .map(|(&lb, &ub)| Constraint {
                display_name: "position".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(lb),
                upper_bound: Some(ub),
            })
            .collect();
        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Hole position optimizer".to_string(),
            objective_function_name: "HolePositionObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
        }
    }

    // Golden: opt_hole_position.json
    const GOLDEN_INITIAL_NORM: f64 = 15900.000398470573;
    const GOLDEN_FINAL_NORM: f64 = 4603.929777899983;

    #[test]
    fn initial_norm_matches() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let norm = evaluate_norm(&inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE);
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn initial_geometry_is_correct_length() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let geometry = get_hole_geometry_position(&inst);
        assert_eq!(geometry.len(), 7, "expected 7-element geometry for 6-hole (N+1)");
    }

    #[test]
    fn optimization_improves_and_matches_golden() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = position_constraints();

        let result = optimize_hole_position(
            &mut inst,
            &tuning,
            &constraints,
            &params,
            &CalculatorParams::WHISTLE,
        );

        // Should improve significantly
        assert!(
            result.final_norm < result.initial_norm * 0.5,
            "optimization should significantly reduce norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm
        );

        // Final norm should be in same ballpark as golden (within 2x)
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 2.0,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }
}

#[cfg(test)]
mod flute_tests {
    use super::*;
    use wid_physics::TemperatureType;
    use wid_types::{Constraint, ConstraintType, parse_instrument_xml, parse_tuning_xml};

    const FLUTE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/instruments/SamplePVC-Flute.xml");
    const FLUTE_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/tunings/D4-Equal.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    fn flute_position_constraints() -> Constraints {
        // First 7 constraints from LargeHoleSize_Spacing_6holes.xml (position only)
        let lower_bounds = [0.4, 0.015, 0.015, 0.015, 0.015, 0.015, 0.015];
        let upper_bounds = [0.7, 0.035, 0.035, 0.1, 0.035, 0.0375, 0.2];
        let constraints: Vec<Constraint> = lower_bounds
            .iter()
            .zip(upper_bounds.iter())
            .map(|(&lb, &ub)| Constraint {
                display_name: "position".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(lb),
                upper_bound: Some(ub),
            })
            .collect();
        Constraints {
            name: "Large Hole Size+Spacing".to_string(),
            objective_display_name: "Hole position optimizer".to_string(),
            objective_function_name: "HolePositionObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
        }
    }

    // Golden: FL-OPT/opt_hole_position.json
    const GOLDEN_INITIAL_NORM: f64 = 2649.612447927295;
    const GOLDEN_FINAL_NORM: f64 = 1594.0436923451473;

    #[test]
    fn flute_initial_norm_matches() {
        let inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = crate::fingering_weights(&tuning.fingerings);
        let norm = evaluate_norm(
            &inst,
            &tuning.fingerings,
            &weights,
            &params,
            &CalculatorParams::FLUTE,
        );
        assert!(
            (norm - GOLDEN_INITIAL_NORM).abs() / GOLDEN_INITIAL_NORM < 0.01,
            "flute initial norm: expected {GOLDEN_INITIAL_NORM}, got {norm}"
        );
    }

    #[test]
    fn flute_optimization_matches_golden() {
        let mut inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = flute_position_constraints();

        let result = optimize_hole_position(
            &mut inst,
            &tuning,
            &constraints,
            &params,
            &CalculatorParams::FLUTE,
        );

        // Should improve
        assert!(
            result.final_norm < result.initial_norm,
            "optimization should reduce norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm
        );

        // Final norm within 2x of golden
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 2.0,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }

    #[test]
    fn fife_optimization_reduces_norm() {
        let fife_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/instruments/fife.xml"
        );
        let fife_tuning_xml = include_str!(
            "../../../../oracle/v2.6.0/FluteStudy/tunings/fife-tuning.xml"
        );
        let mut inst = parse_instrument_xml(fife_xml).unwrap();
        let tuning = parse_tuning_xml(fife_tuning_xml).unwrap();
        let params = default_params();
        let constraints = flute_position_constraints();

        let result = optimize_hole_position(
            &mut inst,
            &tuning,
            &constraints,
            &params,
            &CalculatorParams::FLUTE,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "fife optimization should not worsen norm: initial={}, final={}",
            result.initial_norm,
            result.final_norm
        );
    }
}
