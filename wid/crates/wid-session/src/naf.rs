//! NAF study model: optimizer registry and constraint generation.
//!
//! This module defines the available optimizers for NAF instruments
//! and provides constraint template generation (default and blank).

use crate::types::OptimizerInfo;
use wid_types::{Constraint, ConstraintType, Constraints};

/// NAF optimizer keys (matching Java objective function names).
pub const FIPPLE_FACTOR: &str = "FippleFactorObjectiveFunction";
pub const NAF_HOLE_SIZE: &str = "NafHoleSizeObjectiveFunction";
pub const HOLE_FROM_TOP: &str = "HoleFromTopObjectiveFunction";
pub const HOLE_GROUP_FROM_TOP: &str = "HoleGroupFromTopObjectiveFunction";

/// Returns the list of available NAF optimizers.
pub fn available_optimizers() -> Vec<OptimizerInfo> {
    vec![
        OptimizerInfo {
            key: FIPPLE_FACTOR.to_string(),
            display_name: "Fipple factor".to_string(),
            objective_function_name: FIPPLE_FACTOR.to_string(),
        },
        OptimizerInfo {
            key: HOLE_GROUP_FROM_TOP.to_string(),
            display_name: "Grouped-hole position & size".to_string(),
            objective_function_name: HOLE_GROUP_FROM_TOP.to_string(),
        },
        OptimizerInfo {
            key: HOLE_FROM_TOP.to_string(),
            display_name: "Hole size & position".to_string(),
            objective_function_name: HOLE_FROM_TOP.to_string(),
        },
        OptimizerInfo {
            key: NAF_HOLE_SIZE.to_string(),
            display_name: "Hole size only".to_string(),
            objective_function_name: NAF_HOLE_SIZE.to_string(),
        },
    ]
}

/// Create default constraints for a given optimizer and hole count.
///
/// "Default" constraints have the correct structure (constraint names,
/// categories, types) but all bounds are set to 0.0 (unset).
/// This matches the Java `StudyModel.getDefaultConstraints()` behavior.
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

/// Create blank constraints — same structure as default, all bounds 0.0.
///
/// In the Java baseline, "blank" and "default" produce the same output
/// for NAF study model constraints.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
) -> Constraints {
    // For NAF, blank and default are identical (both have 0.0 bounds)
    create_default_constraints(objective_function_name, number_of_holes)
}

/// Returns the display name for an objective function.
fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        FIPPLE_FACTOR => "Fipple factor",
        NAF_HOLE_SIZE => "Hole size only",
        HOLE_FROM_TOP => "Hole size & position",
        HOLE_GROUP_FROM_TOP => "Grouped-hole position & size",
        _ => "Unknown",
    }
}

/// Generate the constraint template for a given optimizer and hole count.
///
/// The constraint structure (names, categories, types) matches the Java
/// baseline exactly. Bounds are all None (serializes as absent in XML,
/// defaults to 0.0 in lower_bounds/upper_bounds extraction).
fn constraint_template(
    objective_function_name: &str,
    n_holes: u32,
) -> Vec<Constraint> {
    match objective_function_name {
        FIPPLE_FACTOR => fipple_factor_constraints(),
        HOLE_FROM_TOP => hole_from_top_constraints(n_holes),
        NAF_HOLE_SIZE => hole_size_constraints(n_holes),
        HOLE_GROUP_FROM_TOP => hole_group_from_top_constraints(n_holes),
        _ => Vec::new(),
    }
}

/// Fipple factor constraints: single "Mouthpiece fipple factor" constraint.
fn fipple_factor_constraints() -> Vec<Constraint> {
    vec![Constraint {
        display_name: "Mouthpiece fipple factor".to_string(),
        category: "Mouthpiece fipple".to_string(),
        constraint_type: ConstraintType::DIMENSIONLESS,
        lower_bound: None,
        upper_bound: None,
    }]
}

/// HoleFromTop constraints: position (bore_length + fraction + spacings) + size (diameters).
fn hole_from_top_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();

    // Position constraints
    constraints.push(Constraint {
        display_name: "Bore length".to_string(),
        category: "Hole position".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: None,
        upper_bound: None,
    });

    if n_holes > 0 {
        constraints.push(Constraint {
            display_name: format!(
                "Bore top to Hole {} (top), bore-length fraction",
                n_holes
            ),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        });

        // Inter-hole spacings (top to bottom)
        for i in (1..n_holes).rev() {
            let upper = if i == n_holes - 1 {
                format!("Hole {} (top)", n_holes)
            } else {
                format!("Hole {}", i + 1)
            };
            let lower = if i == 1 {
                "Hole 1 (bottom)".to_string()
            } else {
                format!("Hole {}", i)
            };
            constraints.push(Constraint {
                display_name: format!("{upper} to {lower} distance"),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: None,
                upper_bound: None,
            });
        }
    }

    // Size constraints (hole diameters, top to bottom)
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

/// NafHoleSize constraints: just hole diameters.
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

/// HoleGroupFromTop constraints: group spacings + bore length + hole sizes.
fn hole_group_from_top_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();

    // Position constraints
    constraints.push(Constraint {
        display_name: "Bore length".to_string(),
        category: "Hole position".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: None,
        upper_bound: None,
    });

    if n_holes > 0 {
        constraints.push(Constraint {
            display_name: format!(
                "Bore top to Hole {} (top), bore-length fraction",
                n_holes
            ),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        });

        // Group spacing (for 6-hole NAF: upper group spacing + lower group spacing + inter-group gap)
        constraints.push(Constraint {
            display_name: "Upper group spacing".to_string(),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        });
        constraints.push(Constraint {
            display_name: "Inter-group gap".to_string(),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        });
        constraints.push(Constraint {
            display_name: "Lower group spacing".to_string(),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        });
    }

    // Size constraints
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

/// Check if an optimizer key is a valid NAF optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        FIPPLE_FACTOR | NAF_HOLE_SIZE | HOLE_FROM_TOP | HOLE_GROUP_FROM_TOP
    )
}

/// Check if the optimizer requires constraints.
pub fn optimizer_needs_constraints(_key: &str) -> bool {
    // All NAF optimizers require constraints
    true
}

/// Check if the optimizer is the fipple factor calibrator.
pub fn is_fipple_optimizer(key: &str) -> bool {
    key == FIPPLE_FACTOR
}
