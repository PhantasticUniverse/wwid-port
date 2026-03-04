//! Whistle study model: optimizer registry and constraint generation.
//!
//! This module defines the available optimizers for Whistle instruments
//! and provides constraint template generation (default and blank).

use crate::types::OptimizerInfo;
use wid_types::{Constraint, ConstraintType, Constraints, InstrumentRaw};

/// Whistle optimizer keys (matching Java objective function names).
pub const WINDOW_HEIGHT: &str = "WindowHeightObjectiveFunction";
pub const BETA: &str = "BetaObjectiveFunction";
pub const WHISTLE_CALIB: &str = "WhistleCalibrationObjectiveFunction";
pub const HOLE_SIZE: &str = "HoleSizeObjectiveFunction";
pub const HOLE_POSITION: &str = "HolePositionObjectiveFunction";
pub const HOLE: &str = "HoleObjectiveFunction";
pub const GLOBAL_HOLE_POSITION: &str = "GlobalHolePositionObjectiveFunction";
pub const GLOBAL_HOLE: &str = "GlobalHoleObjectiveFunction";

// Bore optimizers
pub const BASIC_TAPER: &str = "BasicTaperObjectiveFunction";
pub const BORE_DIAMETER_FROM_TOP: &str = "BoreDiameterFromTopObjectiveFunction";
pub const BORE_DIAMETER_FROM_BOTTOM: &str = "BoreDiameterFromBottomObjectiveFunction";
pub const BORE_SPACING_FROM_TOP: &str = "BoreSpacingFromTopObjectiveFunction";
pub const HOLE_AND_TAPER: &str = "HoleAndBasicTaperObjectiveFunction";
pub const HOLE_AND_BORE_DIAMETER_FROM_TOP: &str = "HoleAndBoreDiameterFromTopObjectiveFunction";
pub const HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "HoleAndBoreDiameterFromBottomObjectiveFunction";
pub const HOLE_AND_BORE_SPACING: &str = "HoleAndBoreSpacingFromTopObjectiveFunction";
pub const HOLE_AND_HEADJOINT: &str = "HoleAndHeadjointObjectiveFunction";
pub const GLOBAL_HOLE_AND_TAPER: &str = "GlobalHoleAndBasicTaperObjectiveFunction";
pub const GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "GlobalHoleAndBoreDiameterFromBottomObjectiveFunction";

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
        OptimizerInfo {
            key: GLOBAL_HOLE_POSITION.to_string(),
            display_name: "Hole spacing (global)".to_string(),
            objective_function_name: GLOBAL_HOLE_POSITION.to_string(),
        },
        OptimizerInfo {
            key: GLOBAL_HOLE.to_string(),
            display_name: "Hole size+spacing (global)".to_string(),
            objective_function_name: GLOBAL_HOLE.to_string(),
        },
        // Bore optimizers
        OptimizerInfo {
            key: BASIC_TAPER.to_string(),
            display_name: "Basic taper".to_string(),
            objective_function_name: BASIC_TAPER.to_string(),
        },
        OptimizerInfo {
            key: BORE_DIAMETER_FROM_TOP.to_string(),
            display_name: "Bore diameter from top".to_string(),
            objective_function_name: BORE_DIAMETER_FROM_TOP.to_string(),
        },
        OptimizerInfo {
            key: BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Bore diameter from bottom".to_string(),
            objective_function_name: BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: BORE_SPACING_FROM_TOP.to_string(),
            display_name: "Bore spacing from top".to_string(),
            objective_function_name: BORE_SPACING_FROM_TOP.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_TAPER.to_string(),
            display_name: "Holes + basic taper".to_string(),
            objective_function_name: HOLE_AND_TAPER.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_DIAMETER_FROM_TOP.to_string(),
            display_name: "Holes + bore diameter from top".to_string(),
            objective_function_name: HOLE_AND_BORE_DIAMETER_FROM_TOP.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Holes + bore diameter from bottom".to_string(),
            objective_function_name: HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_SPACING.to_string(),
            display_name: "Holes + bore spacing".to_string(),
            objective_function_name: HOLE_AND_BORE_SPACING.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_HEADJOINT.to_string(),
            display_name: "Holes + headjoint".to_string(),
            objective_function_name: HOLE_AND_HEADJOINT.to_string(),
        },
        OptimizerInfo {
            key: GLOBAL_HOLE_AND_TAPER.to_string(),
            display_name: "Holes + basic taper (global)".to_string(),
            objective_function_name: GLOBAL_HOLE_AND_TAPER.to_string(),
        },
        OptimizerInfo {
            key: GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Holes + bore dia from bottom (global)".to_string(),
            objective_function_name: GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
    ]
}

/// Check if an optimizer key is a valid Whistle optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        WINDOW_HEIGHT | BETA | WHISTLE_CALIB | HOLE_SIZE | HOLE_POSITION | HOLE
        | GLOBAL_HOLE_POSITION | GLOBAL_HOLE
        | BASIC_TAPER | BORE_DIAMETER_FROM_TOP | BORE_DIAMETER_FROM_BOTTOM
        | BORE_SPACING_FROM_TOP
        | HOLE_AND_TAPER | HOLE_AND_BORE_DIAMETER_FROM_TOP
        | HOLE_AND_BORE_DIAMETER_FROM_BOTTOM | HOLE_AND_BORE_SPACING
        | HOLE_AND_HEADJOINT
        | GLOBAL_HOLE_AND_TAPER | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM
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
/// When `inst` is provided, bore optimizer constraint counts are derived from
/// the instrument's bore geometry (via `find_head_point`/`find_body_top`).
pub fn create_default_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
    inst: Option<&InstrumentRaw>,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let constraints = constraint_template(objective_function_name, number_of_holes, inst);

    Constraints {
        name: "Default".to_string(),
        objective_display_name: display_name.to_string(),
        objective_function_name: objective_function_name.to_string(),
        number_of_holes,
        constraint_list: constraints,
        hole_groups: None,
    }
}

/// Create blank constraints — same as default for Whistle.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
    inst: Option<&InstrumentRaw>,
) -> Constraints {
    create_default_constraints(objective_function_name, number_of_holes, inst)
}

fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        WINDOW_HEIGHT => "Window Height calibrator",
        BETA => "Beta calibrator",
        WHISTLE_CALIB => "Whistle calibration",
        HOLE => "Hole position and size optimizer",
        HOLE_POSITION => "Hole position optimizer",
        HOLE_SIZE => "Hole size optimizer",
        GLOBAL_HOLE_POSITION => "Hole spacing (global) optimizer",
        GLOBAL_HOLE => "Hole size+spacing (global) optimizer",
        BASIC_TAPER => "Basic taper optimizer",
        BORE_DIAMETER_FROM_TOP => "Bore diameter from top optimizer",
        BORE_DIAMETER_FROM_BOTTOM => "Bore diameter from bottom optimizer",
        BORE_SPACING_FROM_TOP => "Bore spacing from top optimizer",
        HOLE_AND_TAPER => "Holes + basic taper optimizer",
        HOLE_AND_BORE_DIAMETER_FROM_TOP => "Holes + bore diameter from top optimizer",
        HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => "Holes + bore diameter from bottom optimizer",
        HOLE_AND_BORE_SPACING => "Holes + bore spacing optimizer",
        HOLE_AND_HEADJOINT => "Holes + headjoint optimizer",
        GLOBAL_HOLE_AND_TAPER => "Holes + basic taper (global) optimizer",
        GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => "Holes + bore dia from bottom (global)",
        _ => "Unknown",
    }
}

fn constraint_template(
    objective_function_name: &str,
    n_holes: u32,
    inst: Option<&InstrumentRaw>,
) -> Vec<Constraint> {
    // Compute bore dimension counts from instrument geometry when available.
    // FromTop: n_changed = find_head_point() (matches Java getLowestPoint())
    let n_from_top = || inst.map_or(1, |i| wid_compile::find_head_point(i, "Head").max(1));
    // FromBottom: Java uses getTopOfBody()+1 as n_unchanged, so
    //   n_dims = n_bore - (find_body_top + 1)
    let n_from_bottom = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged)).max(1)
        })
    };

    match objective_function_name {
        WINDOW_HEIGHT => window_height_constraints(),
        BETA => beta_constraints(),
        WHISTLE_CALIB => whistle_calib_constraints(),
        HOLE => hole_constraints(n_holes),
        HOLE_POSITION => hole_position_constraints(n_holes),
        HOLE_SIZE => hole_size_constraints(n_holes),
        GLOBAL_HOLE => hole_constraints(n_holes),
        GLOBAL_HOLE_POSITION => hole_position_constraints(n_holes),
        // Standalone bore
        BASIC_TAPER => basic_taper_constraints(),
        BORE_DIAMETER_FROM_TOP => bore_ratio_constraints(n_from_top()),
        BORE_DIAMETER_FROM_BOTTOM => bore_ratio_constraints(n_from_bottom()),
        BORE_SPACING_FROM_TOP => bore_spacing_constraints(n_from_top()),
        // Merged bore
        HOLE_AND_TAPER => {
            let mut c = hole_constraints(n_holes);
            c.extend(basic_taper_constraints());
            c
        }
        HOLE_AND_BORE_DIAMETER_FROM_TOP => {
            let mut c = hole_constraints(n_holes);
            c.extend(bore_ratio_constraints(n_from_top()));
            c
        }
        HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => {
            let mut c = hole_constraints(n_holes);
            c.extend(bore_ratio_constraints(n_from_bottom()));
            c
        }
        HOLE_AND_BORE_SPACING => {
            let mut c = hole_constraints(n_holes);
            c.extend(bore_spacing_constraints(n_from_top()));
            c
        }
        HOLE_AND_HEADJOINT => {
            let mut c = hole_constraints(n_holes);
            c.push(stopper_constraint());
            c.extend(bore_ratio_constraints(n_from_top()));
            c
        }
        // Global bore
        GLOBAL_HOLE_AND_TAPER => {
            let mut c = hole_constraints(n_holes);
            c.extend(basic_taper_constraints());
            c
        }
        GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => {
            let mut c = hole_constraints(n_holes);
            c.extend(bore_ratio_constraints(n_from_bottom()));
            c
        }
        _ => Vec::new(),
    }
}

/// Basic taper constraints: head fraction + foot ratio.
fn basic_taper_constraints() -> Vec<Constraint> {
    vec![
        Constraint {
            display_name: "Head length fraction".to_string(),
            category: "Basic taper".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        },
        Constraint {
            display_name: "Foot diameter ratio".to_string(),
            category: "Basic taper".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        },
    ]
}

/// Bore diameter ratio constraints (DIMENSIONLESS).
pub fn bore_ratio_constraints(n: usize) -> Vec<Constraint> {
    (0..n)
        .map(|_| Constraint {
            display_name: "Bore diameter ratio".to_string(),
            category: "Bore diameter".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        })
        .collect()
}

/// Bore spacing constraints (DIMENSIONAL).
fn bore_spacing_constraints(n: usize) -> Vec<Constraint> {
    (0..n)
        .map(|_| Constraint {
            display_name: "Bore spacing".to_string(),
            category: "Bore spacing".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        })
        .collect()
}

/// Bore position constraints (DIMENSIONLESS — fractional positions).
///
/// Used by `BorePositionObjectiveFunction` (Reed). With `bottom_fixed=true`,
/// all dimensions are fractional positions between adjacent bore points.
pub fn bore_position_constraints(n: usize) -> Vec<Constraint> {
    (0..n)
        .map(|_| Constraint {
            display_name: "Bore position".to_string(),
            category: "Bore position".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        })
        .collect()
}

/// Stopper position constraint (DIMENSIONAL).
pub fn stopper_constraint() -> Constraint {
    Constraint {
        display_name: "Stopper distance".to_string(),
        category: "Stopper position".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: None,
        upper_bound: None,
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
