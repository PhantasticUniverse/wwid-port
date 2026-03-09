//! Reed study model — optimizer registry, calibrator dispatch, and
//! constraint templates.
//!
//! Supports optimizers:
//! - **Calibrator** (no constraints needed): ReedCalibrator (2D joint alpha+beta)
//! - **Hole optimizers** (constraints required): HoleSize, HolePosition, Hole
//! - **Bore optimizers**: BoreDiameterFromBottom, BorePosition, BoreFromBottom, merged variants
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
pub const GLOBAL_HOLE: &str = "GlobalHoleObjectiveFunction";

// Bore optimizers
pub const BORE_DIAMETER_FROM_BOTTOM: &str = "BoreDiameterFromBottomObjectiveFunction";
pub const BORE_POSITION: &str = "BorePositionObjectiveFunction";
pub const BORE_FROM_BOTTOM: &str = "BoreFromBottomObjectiveFunction";
pub const HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "HoleAndBoreDiameterFromBottomObjectiveFunction";
pub const HOLE_AND_BORE_POSITION: &str = "HoleAndBorePositionObjectiveFunction";
pub const HOLE_AND_BORE_FROM_BOTTOM: &str = "HoleAndBoreFromBottomObjectiveFunction";
pub const GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "GlobalHoleAndBoreDiameterFromBottomObjectiveFunction";

// Reed-specific constants matching Java ReedStudyModel
const MIN_BORE_LENGTH: f64 = 0.200;
const MAX_BORE_LENGTH: f64 = 1.000;
const MIN_HOLE_DIAMETER: f64 = 0.0032;
const MAX_HOLE_DIAMETER: f64 = 0.0091;
const MIN_THUMB_HOLE_SPACING: f64 = 0.0002;

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
        OptimizerInfo {
            key: GLOBAL_HOLE.to_string(),
            display_name: "Hole size+spacing (global)".to_string(),
            objective_function_name: GLOBAL_HOLE.to_string(),
        },
        // Bore optimizers
        OptimizerInfo {
            key: BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Bore diameter from bottom".to_string(),
            objective_function_name: BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: BORE_POSITION.to_string(),
            display_name: "Bore position".to_string(),
            objective_function_name: BORE_POSITION.to_string(),
        },
        OptimizerInfo {
            key: BORE_FROM_BOTTOM.to_string(),
            display_name: "Bore from bottom".to_string(),
            objective_function_name: BORE_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Holes + bore diameter from bottom".to_string(),
            objective_function_name: HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_POSITION.to_string(),
            display_name: "Holes + bore position".to_string(),
            objective_function_name: HOLE_AND_BORE_POSITION.to_string(),
        },
        OptimizerInfo {
            key: HOLE_AND_BORE_FROM_BOTTOM.to_string(),
            display_name: "Holes + bore from bottom".to_string(),
            objective_function_name: HOLE_AND_BORE_FROM_BOTTOM.to_string(),
        },
        OptimizerInfo {
            key: GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
            display_name: "Holes + bore dia from bottom (global)".to_string(),
            objective_function_name: GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM.to_string(),
        },
    ]
}

/// Check if an optimizer key is a valid Reed optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        REED_CALIB | HOLE_SIZE | HOLE_POSITION | HOLE | GLOBAL_HOLE
        | BORE_DIAMETER_FROM_BOTTOM | BORE_POSITION | BORE_FROM_BOTTOM
        | HOLE_AND_BORE_DIAMETER_FROM_BOTTOM | HOLE_AND_BORE_POSITION
        | HOLE_AND_BORE_FROM_BOTTOM | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM
    )
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
    inst: Option<&wid_types::InstrumentRaw>,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let mut constraints = constraint_template(objective_function_name, number_of_holes, inst);

    // Apply Reed-specific default bounds
    apply_reed_default_bounds(
        &mut constraints,
        objective_function_name,
        number_of_holes,
        inst,
    );

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
    inst: Option<&wid_types::InstrumentRaw>,
) -> Constraints {
    let display_name = display_name_for(objective_function_name);
    let constraints = constraint_template(objective_function_name, number_of_holes, inst);

    Constraints {
        name: "Blank".to_string(),
        objective_display_name: display_name.to_string(),
        objective_function_name: objective_function_name.to_string(),
        number_of_holes,
        constraint_list: constraints,
        hole_groups: None,
    }
}

fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        REED_CALIB => "Reed calibrator",
        HOLE => "Hole position and size optimizer",
        HOLE_POSITION => "Hole position optimizer",
        HOLE_SIZE => "Hole size optimizer",
        GLOBAL_HOLE => "Hole size+spacing (global) optimizer",
        BORE_DIAMETER_FROM_BOTTOM => "Bore diameter from bottom optimizer",
        BORE_POSITION => "Bore position optimizer",
        BORE_FROM_BOTTOM => "Bore from bottom optimizer",
        HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => "Holes + bore diameter from bottom optimizer",
        HOLE_AND_BORE_POSITION => "Holes + bore position optimizer",
        HOLE_AND_BORE_FROM_BOTTOM => "Holes + bore from bottom optimizer",
        GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => "Holes + bore dia from bottom (global)",
        _ => "Unknown",
    }
}

fn constraint_template(
    objective_function_name: &str,
    n_holes: u32,
    inst: Option<&wid_types::InstrumentRaw>,
) -> Vec<Constraint> {
    // Bore dimension helpers for Reed-specific optimizers.
    // Java uses getTopOfBody()+1 as n_unchanged for all bottom-type bore optimizers.
    let n_from_bottom = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged)).max(1)
        })
    };
    // Standalone BorePosition (bottom_fixed=false): n_bore - n_unchanged
    // (first dim is absolute bottom position, rest fractional)
    let n_position_standalone = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged)).max(1)
        })
    };
    // Merged BorePosition (bottom_fixed=true): n_bore - n_unchanged - 1
    // (all dims fractional, bottom point fixed since holes handle bore length)
    let n_position_merged = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged + 1)).max(1)
        })
    };

    match objective_function_name {
        REED_CALIB => reed_calib_constraints(),
        HOLE => hole_constraints(n_holes),
        HOLE_POSITION => hole_position_constraints(n_holes),
        HOLE_SIZE => hole_size_constraints(n_holes),
        GLOBAL_HOLE => hole_constraints(n_holes),
        // Standalone bore — diameter uses Whistle template, position is Reed-specific
        BORE_DIAMETER_FROM_BOTTOM => {
            crate::whistle::bore_ratio_constraints(n_from_bottom())
        }
        BORE_POSITION => {
            // Standalone: bottom_fixed=false -> first dim is DIMENSIONAL (absolute
            // bottom position), rest are DIMENSIONLESS (fractional positions)
            bore_position_standalone_constraints(n_position_standalone())
        }
        BORE_FROM_BOTTOM => {
            // Standalone merged: position (bottom_fixed=false) + diameter ratios
            let mut c = bore_position_standalone_constraints(n_position_standalone());
            c.extend(crate::whistle::bore_ratio_constraints(n_from_bottom()));
            c
        }
        // Merged bore (with holes — bottom_fixed=true for position component)
        HOLE_AND_BORE_DIAMETER_FROM_BOTTOM | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => {
            let mut c = hole_constraints(n_holes);
            c.extend(crate::whistle::bore_ratio_constraints(n_from_bottom()));
            c
        }
        HOLE_AND_BORE_POSITION => {
            let mut c = hole_constraints(n_holes);
            c.extend(crate::whistle::bore_position_constraints(n_position_merged()));
            c
        }
        HOLE_AND_BORE_FROM_BOTTOM => {
            let mut c = hole_constraints(n_holes);
            c.extend(crate::whistle::bore_position_constraints(n_position_merged()));
            c.extend(crate::whistle::bore_ratio_constraints(n_from_bottom()));
            c
        }
        _ => Vec::new(),
    }
}

/// Apply Reed-specific default bounds.
///
/// Reed differs from Whistle:
/// - Larger bore range (0.200..1.000 vs 0.200..0.600)
/// - Smaller minimum hole diameter (0.0032 vs 0.0040)
/// - Bore flares OUT: diameter ratios 1.0..1.5 (vs Whistle 0.5..1.0)
/// - Bore positions: 0.1..0.9 (fractional)
/// - Thumb hole spacing overrides at positions 1 and 6
fn apply_reed_default_bounds(
    constraints: &mut [Constraint],
    optimizer: &str,
    n_holes: u32,
    inst: Option<&wid_types::InstrumentRaw>,
) {
    let n = n_holes as usize;

    let n_from_bottom = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged)).max(1)
        })
    };
    let n_position_standalone = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged)).max(1)
        })
    };
    let n_position_merged = || {
        inst.map_or(1, |i| {
            let n_unchanged = wid_compile::find_body_top(i) + 1;
            (i.bore_points.len().saturating_sub(n_unchanged + 1)).max(1)
        })
    };

    match optimizer {
        // Calibrator already has bounds set
        REED_CALIB => {}

        // Hole size only
        HOLE_SIZE => {
            for i in 0..n {
                set_bounds(constraints, i, MIN_HOLE_DIAMETER, MAX_HOLE_DIAMETER);
            }
        }

        // Hole position (N+1 dims)
        HOLE_POSITION => {
            apply_reed_hole_position_bounds(constraints, 0, n);
        }

        // Hole position + size (2N+1 dims)
        HOLE | GLOBAL_HOLE => {
            apply_reed_hole_position_bounds(constraints, 0, n);
            for i in 0..n {
                set_bounds(constraints, n + 1 + i, MIN_HOLE_DIAMETER, MAX_HOLE_DIAMETER);
            }
        }

        // Standalone bore diameter from bottom: ratios 1.0..1.5
        BORE_DIAMETER_FROM_BOTTOM => {
            let nd = n_from_bottom();
            for i in 0..nd {
                set_bounds(constraints, i, 1.0, 1.5);
            }
        }

        // Standalone bore position
        BORE_POSITION => {
            let nd = n_position_standalone();
            // First dim: absolute bottom position (DIMENSIONAL)
            set_bounds(constraints, 0, MIN_BORE_LENGTH, MAX_BORE_LENGTH);
            // Rest: fractional positions
            for i in 1..nd {
                set_bounds(constraints, i, 0.1, 0.9);
            }
        }

        // Standalone bore from bottom: position + diameter
        BORE_FROM_BOTTOM => {
            let np = n_position_standalone();
            let nd = n_from_bottom();
            // First dim: absolute bottom position
            set_bounds(constraints, 0, MIN_BORE_LENGTH, MAX_BORE_LENGTH);
            // Check if there are mid-position dims
            if np + nd > 1 {
                // Mid positions (fractional)
                for i in 1..(np + nd) / 2 + 1 {
                    if i < np {
                        set_bounds(constraints, i, 0.1, 0.9);
                    }
                }
                // Diameter ratios
                for i in 0..nd {
                    set_bounds(constraints, np + i, 1.0, 1.5);
                }
            }
        }

        // Holes + bore diameter from bottom
        HOLE_AND_BORE_DIAMETER_FROM_BOTTOM | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => {
            apply_reed_hole_position_bounds(constraints, 0, n);
            for i in 0..n {
                set_bounds(constraints, n + 1 + i, MIN_HOLE_DIAMETER, MAX_HOLE_DIAMETER);
            }
            let bore_start = 2 * n + 1;
            let nd = n_from_bottom();
            // Reed bore flares: ratios 1.0..2.0 (Java: 1.0..2.0 for merged)
            for i in 0..nd {
                set_bounds(constraints, bore_start + i, 1.0, 2.0);
            }
        }

        // Holes + bore position
        HOLE_AND_BORE_POSITION => {
            apply_reed_hole_position_bounds(constraints, 0, n);
            for i in 0..n {
                set_bounds(constraints, n + 1 + i, MIN_HOLE_DIAMETER, MAX_HOLE_DIAMETER);
            }
            let bore_start = 2 * n + 1;
            let nd = n_position_merged();
            for i in 0..nd {
                set_bounds(constraints, bore_start + i, 0.1, 0.9);
            }
        }

        // Holes + bore from bottom (position + diameter)
        HOLE_AND_BORE_FROM_BOTTOM => {
            apply_reed_hole_position_bounds(constraints, 0, n);
            for i in 0..n {
                set_bounds(constraints, n + 1 + i, MIN_HOLE_DIAMETER, MAX_HOLE_DIAMETER);
            }
            let bore_start = 2 * n + 1;
            let np = n_position_merged();
            let nd = n_from_bottom();
            // Positions (fractional)
            for i in 0..np {
                set_bounds(constraints, bore_start + i, 0.1, 0.9);
            }
            // Diameters (ratios)
            for i in 0..nd {
                set_bounds(constraints, bore_start + np + i, 1.0, 1.5);
            }
        }

        _ => {}
    }
}

/// Apply Reed hole position bounds.
/// Same pattern as Whistle but with Reed constants + thumb hole overrides.
fn apply_reed_hole_position_bounds(constraints: &mut [Constraint], offset: usize, n: usize) {
    set_bounds(constraints, offset, MIN_BORE_LENGTH, MAX_BORE_LENGTH);
    for j in 1..n {
        set_bounds(constraints, offset + j, 0.012, 0.040);
    }
    if n > 0 {
        set_bounds(constraints, offset + n, 0.012, 0.200);
    }
    // Java: if (numberOfHoles >= 5) upperBound[(numberOfHoles + 1) / 2] = 0.100
    if n >= 5 {
        if let Some(c) = constraints.get_mut(offset + n.div_ceil(2)) {
            c.upper_bound = Some(0.100);
        }
    }
    // Java thumb hole spacing overrides:
    // if (numberOfHoles == 7 || numberOfHoles == 8) lowerBound[1] = MIN_THUMB_HOLE_SPACING
    // if (numberOfHoles >= 10) lowerBound[6] = MIN_THUMB_HOLE_SPACING
    if n == 7 || n == 8 {
        if let Some(c) = constraints.get_mut(offset + 1) {
            c.lower_bound = Some(MIN_THUMB_HOLE_SPACING);
        }
    }
    if n >= 10 {
        if let Some(c) = constraints.get_mut(offset + 6) {
            c.lower_bound = Some(MIN_THUMB_HOLE_SPACING);
        }
    }
}

fn set_bounds(constraints: &mut [Constraint], idx: usize, lo: f64, hi: f64) {
    if let Some(c) = constraints.get_mut(idx) {
        c.lower_bound = Some(lo);
        c.upper_bound = Some(hi);
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

/// Standalone bore position constraints (bottom_fixed=false).
///
/// First dim is DIMENSIONAL (absolute bottom position in metres),
/// remaining dims are DIMENSIONLESS (fractional positions).
/// Matches Java `BorePositionObjectiveFunction` with `bottomPointUnchanged=false`.
fn bore_position_standalone_constraints(n: usize) -> Vec<Constraint> {
    let mut constraints = Vec::with_capacity(n);
    if n > 0 {
        // First dimension: absolute bottom bore position
        constraints.push(Constraint {
            display_name: "Bottom bore position".to_string(),
            category: "Bore position".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: None,
            upper_bound: None,
        });
    }
    // Remaining dimensions: fractional positions
    for _ in 1..n {
        constraints.push(Constraint {
            display_name: "Bore position".to_string(),
            category: "Bore position".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: None,
            upper_bound: None,
        });
    }
    constraints
}

/// Hole position constraints — delegate to Whistle (identical structure).
fn hole_position_constraints(n_holes: u32) -> Vec<Constraint> {
    crate::whistle::create_blank_constraints(
        crate::whistle::HOLE_POSITION,
        n_holes,
        None,
    )
    .constraint_list
}

/// Hole size constraints — delegate to Whistle (identical structure).
fn hole_size_constraints(n_holes: u32) -> Vec<Constraint> {
    crate::whistle::create_blank_constraints(
        crate::whistle::HOLE_SIZE,
        n_holes,
        None,
    )
    .constraint_list
}

/// Merged hole constraints: position (N+1) + size (N) = 2N+1 total.
fn hole_constraints(n_holes: u32) -> Vec<Constraint> {
    let mut constraints = hole_position_constraints(n_holes);
    constraints.extend(hole_size_constraints(n_holes));
    constraints
}
