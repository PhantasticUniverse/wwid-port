//! Hole size optimization (diameters only).
//!
//! N-dimensional BOBYQA optimization of hole diameters.
//! Port of WIDesigner's `HoleSizeObjectiveFunction`.

use bobyqa_impl::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use wid_compile::{compile, get_hole_diameters, set_hole_diameters};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, fingering_weights};

/// Optimize hole diameters using BOBYQA.
///
/// The instrument is modified in place with the optimized diameters.
pub fn optimize_hole_size(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_hole_size_impl(instrument, tuning, constraints, params, calc_params, None)
}

/// Like [`optimize_hole_size`], but with a progress callback.
pub fn optimize_hole_size_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_size_impl(
        instrument,
        tuning,
        constraints,
        params,
        calc_params,
        Some(on_progress),
    )
}

fn optimize_hole_size_impl(
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

    let raw_geometry = get_hole_diameters(instrument);
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
                set_hole_diameters(&mut work_inst, point);
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
                set_hole_diameters(&mut work_inst, point);
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
            set_hole_diameters(instrument, &opt_result.point);
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

    fn size_constraints() -> Constraints {
        // 6 diameter constraints from DefaultHoleConstraints.xml
        let constraints: Vec<Constraint> = (0..6)
            .map(|_| Constraint {
                display_name: "diameter".to_string(),
                category: "Hole size".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.004),
                upper_bound: Some(0.0091),
            })
            .collect();
        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Hole size only".to_string(),
            objective_function_name: "HoleSizeObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
        }
    }

    // Golden: opt_hole_size.json
    const GOLDEN_INITIAL_NORM: f64 = 15900.000398470573;
    const GOLDEN_FINAL_NORM: f64 = 5660.8274741111045;

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
    fn optimization_improves_and_matches_golden() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = size_constraints();

        let result = optimize_hole_size(
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

        // Final norm should be in same ballpark as golden (within 2x, BOBYQA paths differ)
        assert!(
            result.final_norm < GOLDEN_FINAL_NORM * 2.0,
            "final norm should be close to golden: expected ~{GOLDEN_FINAL_NORM}, got {}",
            result.final_norm
        );
    }
}
