//! Whistle study model: optimizer registry and constraint generation.
//!
//! This module defines the available optimizers for Whistle instruments
//! and provides constraint template generation (default and blank).

use crate::types::OptimizerInfo;
use wid_types::{Constraint, ConstraintType, Constraints};

/// Whistle optimizer keys (matching Java objective function names).
pub const WINDOW_HEIGHT: &str = "WindowHeightObjectiveFunction";
pub const BETA: &str = "BetaObjectiveFunction";
pub const WHISTLE_CALIB: &str = "WhistleCalibrationObjectiveFunction";
pub const HOLE_SIZE: &str = "HoleSizeObjectiveFunction";
pub const HOLE_POSITION: &str = "HolePositionObjectiveFunction";
pub const HOLE: &str = "HoleObjectiveFunction";

/// Returns the list of available Whistle optimizers.
pub fn available_optimizers() -> Vec<OptimizerInfo> {
    vec![
        OptimizerInfo {
            key: WINDOW_HEIGHT.to_string(),
            display_name: "Window height calibrator".to_string(),
            objective_function_name: WINDOW_HEIGHT.to_string(),
        },
        OptimizerInfo {
            key: BETA.to_string(),
            display_name: "Beta calibrator".to_string(),
            objective_function_name: BETA.to_string(),
        },
        OptimizerInfo {
            key: WHISTLE_CALIB.to_string(),
            display_name: "Whistle calibration".to_string(),
            objective_function_name: WHISTLE_CALIB.to_string(),
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

/// Check if an optimizer key is a valid Whistle optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        WINDOW_HEIGHT | BETA | WHISTLE_CALIB | HOLE_SIZE | HOLE_POSITION | HOLE
    )
}

/// Check if the optimizer is a calibrator (doesn't need hole constraints).
pub fn is_calibrator(key: &str) -> bool {
    matches!(key, WINDOW_HEIGHT | BETA | WHISTLE_CALIB)
}

/// Check if the optimizer requires constraints to be selected.
///
/// Calibrators use built-in default bounds; hole optimizers need explicit constraints.
pub fn optimizer_needs_constraints(key: &str) -> bool {
    !is_calibrator(key)
}

/// Create default constraints for a given optimizer and hole count.
///
/// Calibrator constraints have pre-filled default bounds matching Java defaults.
/// Hole optimizer constraints have blank bounds (all None).
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

/// Create blank constraints — same as default for Whistle.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
) -> Constraints {
    create_default_constraints(objective_function_name, number_of_holes)
}

fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        WINDOW_HEIGHT => "Window Height calibrator",
        BETA => "Beta calibrator",
        WHISTLE_CALIB => "Whistle calibration",
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
        WINDOW_HEIGHT => window_height_constraints(),
        BETA => beta_constraints(),
        WHISTLE_CALIB => whistle_calib_constraints(),
        HOLE => hole_constraints(n_holes),
        HOLE_POSITION => hole_position_constraints(n_holes),
        HOLE_SIZE => hole_size_constraints(n_holes),
        _ => Vec::new(),
    }
}

/// WindowHeight: single constraint with default bounds.
fn window_height_constraints() -> Vec<Constraint> {
    vec![Constraint {
        display_name: "Window height".to_string(),
        category: "Mouthpiece window".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: Some(wid_optimize::window_height::DEFAULT_WH_LOWER),
        upper_bound: Some(wid_optimize::window_height::DEFAULT_WH_UPPER),
    }]
}

/// Beta: single constraint with default bounds.
fn beta_constraints() -> Vec<Constraint> {
    vec![Constraint {
        display_name: "Beta".to_string(),
        category: "Mouthpiece beta".to_string(),
        constraint_type: ConstraintType::DIMENSIONLESS,
        lower_bound: Some(wid_optimize::beta::DEFAULT_BETA_LOWER),
        upper_bound: Some(wid_optimize::beta::DEFAULT_BETA_UPPER),
    }]
}

/// WhistleCalibration: window height + beta (2 constraints).
fn whistle_calib_constraints() -> Vec<Constraint> {
    vec![
        Constraint {
            display_name: "Window height".to_string(),
            category: "Mouthpiece calibration".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: Some(wid_optimize::window_height::DEFAULT_WH_LOWER),
            upper_bound: Some(wid_optimize::window_height::DEFAULT_WH_UPPER),
        },
        Constraint {
            display_name: "Beta".to_string(),
            category: "Mouthpiece calibration".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: Some(wid_optimize::beta::DEFAULT_BETA_LOWER),
            upper_bound: Some(wid_optimize::beta::DEFAULT_BETA_UPPER),
        },
    ]
}

/// Hole position constraints: bore length + inter-hole spacings (N+1 total).
///
/// Ordering matches Java HolePositionObjectiveFunction geometry vector:
/// `[bore_end, spacing_top_to_next, ..., spacing_bottom_to_bore_end]`
///
/// Hole numbering: Hole N = top (closest to mouthpiece), Hole 1 = bottom (closest to bell).
fn hole_position_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();

    constraints.push(Constraint {
        display_name: "Bore length".to_string(),
        category: "Hole position".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: None,
        upper_bound: None,
    });

    // Spacings: geometry[1..=n_holes] maps to constraints top-to-bottom then bore-end
    for j in 1..=n_holes {
        if j < n_holes {
            // Inter-hole spacing
            let upper_num = n_holes - j + 1;
            let lower_num = n_holes - j;
            let upper = if upper_num == n_holes {
                format!("Hole {} (top)", n_holes)
            } else {
                format!("Hole {}", upper_num)
            };
            let lower = if lower_num == 1 {
                "Hole 1 (bottom)".to_string()
            } else {
                format!("Hole {}", lower_num)
            };
            constraints.push(Constraint {
                display_name: format!("{upper} to {lower} distance"),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: None,
                upper_bound: None,
            });
        } else {
            // Bottom hole to bore end
            constraints.push(Constraint {
                display_name: "Hole 1 (bottom) to bore end distance".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: None,
                upper_bound: None,
            });
        }
    }

    constraints
}

/// Hole size constraints: diameters only (N total).
///
/// Ordered top-to-bottom matching geometry vector.
fn hole_size_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    for i in (1..=n_holes).rev() {
        let name = if i == n_holes {
            format!("Hole {} (top) diameter", n_holes)
        } else if i == 1 {
            "Hole 1 (bottom) diameter".to_string()
        } else {
            format!("Hole {} diameter", i)
        };
        constraints.push(Constraint {
            display_name: name,
            category: "Hole size".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        });
    }
    constraints
}

/// Merged hole constraints: position (N+1) + size (N) = 2N+1 total.
fn hole_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = hole_position_constraints(n_holes);
    constraints.extend(hole_size_constraints(n_holes));
    constraints
}
