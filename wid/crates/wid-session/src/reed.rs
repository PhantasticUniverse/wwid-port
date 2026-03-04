//! Reed study model — optimizer registry, calibrator dispatch, and
//! constraint templates.
//!
//! Supports 4 optimizers:
//! - **Calibrator** (no constraints needed): ReedCalibrator (2D joint alpha+beta)
//! - **Hole optimizers** (constraints required): HoleSize, HolePosition, Hole
//!
//! The Reed calibrator uses `CentDeviationEvaluator` (not FminmaxEvaluator),
//! matching Java `ReedCalibratorObjectiveFunction`.

use crate::types::OptimizerInfo;
use wid_types::{Constraint, ConstraintType, Constraints};

/// Reed optimizer keys (matching Java objective function names).
pub const REED_CALIB: &str = "ReedCalibratorObjectiveFunction";
pub const HOLE_SIZE: &str = "HoleSizeObjectiveFunction";
pub const HOLE_POSITION: &str = "HolePositionObjectiveFunction";
pub const HOLE: &str = "HoleObjectiveFunction";

/// Returns the list of available Reed optimizers.
pub fn available_optimizers() -> Vec<OptimizerInfo> {
    vec![
        OptimizerInfo {
            key: REED_CALIB.to_string(),
            display_name: "Reed calibrator".to_string(),
            objective_function_name: REED_CALIB.to_string(),
        },
        OptimizerInfo {
            key: HOLE.to_string(),
            display_name: "Hole position & size".to_string(),
            objective_function_name: HOLE.to_string(),
        },
        OptimizerInfo {
            key: HOLE_POSITION.to_string(),
            display_name: "Hole position only".to_string(),
            objective_function_name: HOLE_POSITION.to_string(),
        },
        OptimizerInfo {
            key: HOLE_SIZE.to_string(),
            display_name: "Hole size only".to_string(),
            objective_function_name: HOLE_SIZE.to_string(),
        },
    ]
}

/// Check if an optimizer key is a valid Reed optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(key, REED_CALIB | HOLE_SIZE | HOLE_POSITION | HOLE)
}

/// Check if the optimizer is a calibrator (doesn't need hole constraints).
pub fn is_calibrator(key: &str) -> bool {
    matches!(key, REED_CALIB)
}

/// Check if the optimizer requires constraints to be selected.
pub fn optimizer_needs_constraints(key: &str) -> bool {
    !is_calibrator(key)
}

/// Create default constraints for a given optimizer and hole count.
pub fn create_default_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let constraints = constraint_template(objective_function_name, number_of_holes);

    Constraints {
        name: "Default".to_string(),
        objective_display_name: display_name.to_string(),
        objective_function_name: objective_function_name.to_string(),
        number_of_holes,
        constraint_list: constraints,
    }
}

/// Create blank constraints — same as default for Reed.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
) -> Constraints {
    create_default_constraints(objective_function_name, number_of_holes)
}

fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        REED_CALIB => "Reed calibrator",
        HOLE => "Hole position and size optimizer",
        HOLE_POSITION => "Hole position optimizer",
        HOLE_SIZE => "Hole size optimizer",
        _ => "Unknown",
    }
}

fn constraint_template(
    objective_function_name: &str,
    n_holes: u32,
) -> Vec<Constraint> {
    match objective_function_name {
        REED_CALIB => reed_calib_constraints(),
        HOLE => hole_constraints(n_holes),
        HOLE_POSITION => hole_position_constraints(n_holes),
        HOLE_SIZE => hole_size_constraints(n_holes),
        _ => Vec::new(),
    }
}

/// Reed calibrator: Alpha + Beta (2 constraints, both DIMENSIONLESS).
fn reed_calib_constraints() -> Vec<Constraint> {
    vec![
        Constraint {
            display_name: "Alpha".to_string(),
            category: "Mouthpiece parameters".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: Some(wid_optimize::reed_calib::DEFAULT_ALPHA_LOWER),
            upper_bound: Some(wid_optimize::reed_calib::DEFAULT_ALPHA_UPPER),
        },
        Constraint {
            display_name: "Beta".to_string(),
            category: "Mouthpiece parameters".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: Some(wid_optimize::reed_calib::DEFAULT_BETA_LOWER),
            upper_bound: Some(wid_optimize::reed_calib::DEFAULT_BETA_UPPER),
        },
    ]
}

/// Hole position constraints — delegate to Whistle (identical structure).
fn hole_position_constraints(n_holes: u32) -> Vec<Constraint> {
    crate::whistle::create_default_constraints(
        crate::whistle::HOLE_POSITION,
        n_holes,
    )
    .constraint_list
}

/// Hole size constraints — delegate to Whistle (identical structure).
fn hole_size_constraints(n_holes: u32) -> Vec<Constraint> {
    crate::whistle::create_default_constraints(
        crate::whistle::HOLE_SIZE,
        n_holes,
    )
    .constraint_list
}

/// Merged hole constraints: position (N+1) + size (N) = 2N+1 total.
fn hole_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = hole_position_constraints(n_holes);
    constraints.extend(hole_size_constraints(n_holes));
    constraints
}
