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
pub const TAPER_NO_GROUPING: &str = "SingleTaperNoHoleGroupingFromTopObjectiveFunction";
pub const TAPER_NO_GROUPING_HEMI: &str =
    "SingleTaperNoHoleGroupingFromTopHemiHeadObjectiveFunction";
pub const TAPER_HOLE_GROUP: &str = "SingleTaperHoleGroupFromTopObjectiveFunction";
pub const TAPER_HOLE_GROUP_HEMI: &str =
    "SingleTaperHoleGroupFromTopHemiHeadObjectiveFunction";

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
        OptimizerInfo {
            key: TAPER_HOLE_GROUP.to_string(),
            display_name: "Taper, grouped-hole".to_string(),
            objective_function_name: TAPER_HOLE_GROUP.to_string(),
        },
        OptimizerInfo {
            key: TAPER_HOLE_GROUP_HEMI.to_string(),
            display_name: "Taper, grouped-hole, hemispherical".to_string(),
            objective_function_name: TAPER_HOLE_GROUP_HEMI.to_string(),
        },
        OptimizerInfo {
            key: TAPER_NO_GROUPING.to_string(),
            display_name: "Taper, no hole grouping".to_string(),
            objective_function_name: TAPER_NO_GROUPING.to_string(),
        },
        OptimizerInfo {
            key: TAPER_NO_GROUPING_HEMI.to_string(),
            display_name: "Taper, no hole grouping, hemispherical".to_string(),
            objective_function_name: TAPER_NO_GROUPING_HEMI.to_string(),
        },
    ]
}

/// Create default constraints for a given optimizer and hole count.
///
/// "Default" constraints have pre-populated bounds matching Java defaults.
/// NAF has hardcoded bounds for 0, 6, and 7 holes; other hole counts
/// fall back to blank bounds.
pub fn create_default_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
    _inst: Option<&wid_types::InstrumentRaw>,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let mut constraints = constraint_template(objective_function_name, number_of_holes);
    apply_default_bounds(&mut constraints, objective_function_name, number_of_holes);

    Constraints {
        name: "Default".to_string(),
        objective_display_name: display_name.to_string(),
        objective_function_name: objective_function_name.to_string(),
        number_of_holes,
        constraint_list: constraints,
        hole_groups: None,
    }
}

/// Create blank constraints — same structure, all bounds None.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
    _inst: Option<&wid_types::InstrumentRaw>,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let constraints = constraint_template(objective_function_name, number_of_holes);

    Constraints {
        name: "Blank".to_string(),
        objective_display_name: display_name.to_string(),
        objective_function_name: objective_function_name.to_string(),
        number_of_holes,
        constraint_list: constraints,
        hole_groups: None,
    }
}

/// Returns the display name for an objective function.
fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        FIPPLE_FACTOR => "Fipple factor",
        NAF_HOLE_SIZE => "Hole size only",
        HOLE_FROM_TOP => "Hole size & position",
        HOLE_GROUP_FROM_TOP => "Grouped-hole position & size",
        TAPER_NO_GROUPING => "Taper, no hole grouping",
        TAPER_NO_GROUPING_HEMI => "Taper, no hole grouping, hemispherical",
        TAPER_HOLE_GROUP => "Taper, grouped-hole",
        TAPER_HOLE_GROUP_HEMI => "Taper, grouped-hole, hemispherical",
        _ => "Unknown",
    }
}

/// Generate the constraint template for a given optimizer and hole count.
///
/// The constraint structure (names, categories, types) matches the Java
/// baseline exactly. Bounds are all None (populated separately for defaults).
fn constraint_template(
    objective_function_name: &str,
    n_holes: u32,
) -> Vec<Constraint> {
    match objective_function_name {
        FIPPLE_FACTOR => fipple_factor_constraints(),
        HOLE_FROM_TOP => hole_from_top_constraints(n_holes),
        NAF_HOLE_SIZE => hole_size_constraints(n_holes),
        HOLE_GROUP_FROM_TOP => hole_group_from_top_constraints(n_holes),
        TAPER_NO_GROUPING | TAPER_NO_GROUPING_HEMI => {
            taper_no_grouping_constraints(n_holes)
        }
        TAPER_HOLE_GROUP | TAPER_HOLE_GROUP_HEMI => {
            taper_hole_group_constraints(n_holes)
        }
        _ => Vec::new(),
    }
}

/// Apply default bounds from Java NafStudyModel hardcoded arrays.
///
/// NAF has specific defaults for 0, 6, and 7 holes only.
/// For other hole counts, Java falls back to blank constraints.
fn apply_default_bounds(constraints: &mut [Constraint], optimizer: &str, n_holes: u32) {
    let bounds: Option<(Vec<f64>, Vec<f64>)> = match optimizer {
        FIPPLE_FACTOR => Some((vec![0.2], vec![1.5])),

        NAF_HOLE_SIZE => match n_holes {
            0 => Some((vec![], vec![])),
            6 => Some((
                vec![0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.003175],
                vec![0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.0127],
            )),
            7 => Some((
                vec![0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.002032, 0.002032],
                vec![0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.00635, 0.00635],
            )),
            _ => None,
        },

        HOLE_FROM_TOP => match n_holes {
            0 => Some((vec![0.1905], vec![0.6985])),
            6 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.02032, 0.02032,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.003175],
                vec![0.6985, 0.50, 0.03175, 0.03175, 0.0762, 0.03175, 0.03175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.0127],
            )),
            7 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.02032, 0.02032, 0.0,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.002032, 0.002032],
                vec![0.6985, 0.50, 0.03175, 0.03175, 0.0762, 0.03175, 0.03175, 0.003175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.00635, 0.00635],
            )),
            _ => None,
        },

        HOLE_GROUP_FROM_TOP => match n_holes {
            0 => Some((vec![0.1905], vec![0.6985])),
            6 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.003175],
                vec![0.6985, 0.5, 0.03175, 0.0762, 0.03175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.0127],
            )),
            7 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.0,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.002032, 0.002032],
                vec![0.6985, 0.5, 0.03175, 0.0762, 0.03175, 0.003175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.00635, 0.00635],
            )),
            _ => None,
        },

        TAPER_NO_GROUPING | TAPER_NO_GROUPING_HEMI => match n_holes {
            0 => Some((
                vec![0.1905, 0.8, 0.0, 0.0],
                vec![0.6985, 1.2, 1.0, 1.0],
            )),
            6 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.02032, 0.02032,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.003175,
                     0.8, 0.0, 0.0],
                vec![0.6985, 0.50, 0.03175, 0.03175, 0.0762, 0.03175, 0.03175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.0127,
                     1.2, 1.0, 1.0],
            )),
            7 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.02032, 0.02032, 0.0,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.002032, 0.002032,
                     0.8, 0.0, 0.0],
                vec![0.6985, 0.50, 0.03175, 0.03175, 0.0762, 0.03175, 0.03175, 0.003175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.00635, 0.00635,
                     1.2, 1.0, 1.0],
            )),
            _ => None,
        },

        TAPER_HOLE_GROUP | TAPER_HOLE_GROUP_HEMI => match n_holes {
            0 => Some((
                vec![0.1905, 0.8, 0.0, 0.0],
                vec![0.6985, 1.2, 1.0, 1.0],
            )),
            6 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.003175,
                     0.8, 0.0, 0.0],
                vec![0.6985, 0.5, 0.03175, 0.0762, 0.03175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.0127,
                     1.2, 1.0, 1.0],
            )),
            7 => Some((
                vec![0.1905, 0.25, 0.02032, 0.02032, 0.02032, 0.0,
                     0.002032, 0.003175, 0.003175, 0.003175, 0.003175, 0.002032, 0.002032,
                     0.8, 0.0, 0.0],
                vec![0.6985, 0.5, 0.03175, 0.0762, 0.03175, 0.003175,
                     0.0127, 0.0127, 0.0127, 0.0127, 0.0127, 0.00635, 0.00635,
                     1.2, 1.0, 1.0],
            )),
            _ => None,
        },

        _ => None,
    };

    if let Some((lower, upper)) = bounds {
        for (c, (lo, hi)) in constraints.iter_mut().zip(lower.into_iter().zip(upper)) {
            c.lower_bound = Some(lo);
            c.upper_bound = Some(hi);
        }
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
    append_hole_size_constraints(&mut constraints, n_holes);

    constraints
}

/// NafHoleSize constraints: just hole diameters.
fn hole_size_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    append_hole_size_constraints(&mut constraints, n_holes);
    constraints
}

/// HoleGroupFromTop constraints: group spacings + bore length + hole sizes.
fn hole_group_from_top_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();

    // Position constraints
    append_group_position_constraints(&mut constraints, n_holes);

    // Size constraints
    append_hole_size_constraints(&mut constraints, n_holes);

    constraints
}

/// SingleTaperNoHoleGroupingFromTop constraints: hole position + size + taper.
fn taper_no_grouping_constraints(n_holes: u32) -> Vec<Constraint> {
    // Same position + size as HoleFromTop, plus 3 taper dims
    let mut constraints = hole_from_top_constraints(n_holes);
    append_taper_constraints(&mut constraints);
    constraints
}

/// SingleTaperHoleGroupFromTop constraints: grouped position + size + taper.
fn taper_hole_group_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = Vec::new();
    append_group_position_constraints(&mut constraints, n_holes);
    append_hole_size_constraints(&mut constraints, n_holes);
    append_taper_constraints(&mut constraints);
    constraints
}

// ── Shared constraint builders ──────────────────────────────────

/// Append hole diameter constraints (top to bottom).
fn append_hole_size_constraints(constraints: &mut Vec<Constraint>, n_holes: u32) {
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
}

/// Append grouped position constraints (bore_end + top_ratio + group spacings).
fn append_group_position_constraints(constraints: &mut Vec<Constraint>, n_holes: u32) {
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

        // Group spacings (for 6-hole: upper spacing, inter-group gap, lower spacing)
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
}

/// Append single taper constraints (3 dimensionless bounds).
fn append_taper_constraints(constraints: &mut Vec<Constraint>) {
    constraints.push(Constraint {
        display_name: "Taper ratio".to_string(),
        category: "Single bore taper".to_string(),
        constraint_type: ConstraintType::DIMENSIONLESS,
        lower_bound: None,
        upper_bound: None,
    });
    constraints.push(Constraint {
        display_name: "Taper start".to_string(),
        category: "Single bore taper".to_string(),
        constraint_type: ConstraintType::DIMENSIONLESS,
        lower_bound: None,
        upper_bound: None,
    });
    constraints.push(Constraint {
        display_name: "Taper length".to_string(),
        category: "Single bore taper".to_string(),
        constraint_type: ConstraintType::DIMENSIONLESS,
        lower_bound: None,
        upper_bound: None,
    });
}

/// Check if an optimizer key is a valid NAF optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        FIPPLE_FACTOR
            | NAF_HOLE_SIZE
            | HOLE_FROM_TOP
            | HOLE_GROUP_FROM_TOP
            | TAPER_NO_GROUPING
            | TAPER_NO_GROUPING_HEMI
            | TAPER_HOLE_GROUP
            | TAPER_HOLE_GROUP_HEMI
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
