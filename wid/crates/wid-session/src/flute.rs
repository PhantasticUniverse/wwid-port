//! Flute study model — optimizer registry, calibrator dispatch, and
//! constraint templates.
//!
//! Mirrors `whistle.rs` but uses airstream length instead of window height.
//! AirstreamLength and Beta sub-calibrators are used internally by
//! FluteCalibration but not listed in the GUI optimizer menu (matching Java).

use crate::types::OptimizerInfo;
use wid_types::{Constraint, ConstraintType, Constraints};

/// Flute optimizer keys (matching Java objective function names).
pub const AIRSTREAM_LENGTH: &str = "AirstreamLengthObjectiveFunction";
pub const BETA: &str = "BetaObjectiveFunction";
pub const FLUTE_CALIB: &str = "FluteCalibrationObjectiveFunction";
pub const HOLE_SIZE: &str = "HoleSizeObjectiveFunction";
pub const HOLE_POSITION: &str = "HolePositionObjectiveFunction";
pub const HOLE: &str = "HoleObjectiveFunction";
pub const GLOBAL_HOLE_POSITION: &str = "GlobalHolePositionObjectiveFunction";
pub const GLOBAL_HOLE: &str = "GlobalHoleObjectiveFunction";

// Bore optimizers (Flute-specific + inherited from Whistle)
pub const STOPPER_POSITION: &str = "StopperPositionObjectiveFunction";
pub const HEADJOINT: &str = "HeadjointObjectiveFunction";
pub const BASIC_TAPER: &str = "BasicTaperObjectiveFunction";
pub const BORE_DIAMETER_FROM_BOTTOM: &str = "BoreDiameterFromBottomObjectiveFunction";
pub const BORE_SPACING_FROM_TOP: &str = "BoreSpacingFromTopObjectiveFunction";
pub const HOLE_AND_TAPER: &str = "HoleAndBasicTaperObjectiveFunction";
pub const HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "HoleAndBoreDiameterFromBottomObjectiveFunction";
pub const HOLE_AND_BORE_SPACING: &str = "HoleAndBoreSpacingFromTopObjectiveFunction";
pub const HOLE_AND_HEADJOINT: &str = "HoleAndHeadjointObjectiveFunction";
pub const GLOBAL_HOLE_AND_TAPER: &str = "GlobalHoleAndBasicTaperObjectiveFunction";
pub const GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM: &str = "GlobalHoleAndBoreDiameterFromBottomObjectiveFunction";

/// Returns the list of available Flute optimizers.
pub fn available_optimizers() -> Vec<OptimizerInfo> {
    vec![
        OptimizerInfo {
            key: FLUTE_CALIB.to_string(),
            display_name: "Flute calibration".to_string(),
            objective_function_name: FLUTE_CALIB.to_string(),
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
            key: STOPPER_POSITION.to_string(),
            display_name: "Stopper position".to_string(),
            objective_function_name: STOPPER_POSITION.to_string(),
        },
        OptimizerInfo {
            key: HEADJOINT.to_string(),
            display_name: "Headjoint".to_string(),
            objective_function_name: HEADJOINT.to_string(),
        },
        OptimizerInfo {
            key: BASIC_TAPER.to_string(),
            display_name: "Basic taper".to_string(),
            objective_function_name: BASIC_TAPER.to_string(),
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

/// Check if an optimizer key is a valid Flute optimizer.
pub fn is_valid_optimizer(key: &str) -> bool {
    matches!(
        key,
        AIRSTREAM_LENGTH | BETA | FLUTE_CALIB | HOLE_SIZE | HOLE_POSITION | HOLE
        | GLOBAL_HOLE_POSITION | GLOBAL_HOLE
        | STOPPER_POSITION | HEADJOINT | BASIC_TAPER
        | BORE_DIAMETER_FROM_BOTTOM | BORE_SPACING_FROM_TOP
        | HOLE_AND_TAPER | HOLE_AND_BORE_DIAMETER_FROM_BOTTOM
        | HOLE_AND_BORE_SPACING | HOLE_AND_HEADJOINT
        | GLOBAL_HOLE_AND_TAPER | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM
    )
}

/// Check if the optimizer is a calibrator (doesn't need hole constraints).
pub fn is_calibrator(key: &str) -> bool {
    matches!(key, AIRSTREAM_LENGTH | BETA | FLUTE_CALIB)
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

/// Create blank constraints — same as default for Flute.
pub fn create_blank_constraints(
    objective_function_name: &str,
    number_of_holes: u32,
    inst: Option<&wid_types::InstrumentRaw>,
) -> Constraints {
    create_default_constraints(objective_function_name, number_of_holes, inst)
}

fn display_name_for(objective_function_name: &str) -> &'static str {
    match objective_function_name {
        AIRSTREAM_LENGTH => "Airstream length calibrator",
        BETA => "Beta calibrator",
        FLUTE_CALIB => "Flute calibration",
        HOLE => "Hole position and size optimizer",
        HOLE_POSITION => "Hole position optimizer",
        HOLE_SIZE => "Hole size optimizer",
        GLOBAL_HOLE_POSITION => "Hole spacing (global) optimizer",
        GLOBAL_HOLE => "Hole size+spacing (global) optimizer",
        STOPPER_POSITION => "Stopper position optimizer",
        HEADJOINT => "Headjoint optimizer",
        BASIC_TAPER => "Basic taper optimizer",
        BORE_DIAMETER_FROM_BOTTOM => "Bore diameter from bottom optimizer",
        BORE_SPACING_FROM_TOP => "Bore spacing from top optimizer",
        HOLE_AND_TAPER => "Holes + basic taper optimizer",
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
    inst: Option<&wid_types::InstrumentRaw>,
) -> Vec<Constraint> {
    match objective_function_name {
        AIRSTREAM_LENGTH => airstream_length_constraints(),
        BETA => beta_constraints(),
        FLUTE_CALIB => flute_calib_constraints(),
        HOLE => hole_constraints(n_holes),
        HOLE_POSITION => hole_position_constraints(n_holes),
        HOLE_SIZE => hole_size_constraints(n_holes),
        GLOBAL_HOLE => hole_constraints(n_holes),
        GLOBAL_HOLE_POSITION => hole_position_constraints(n_holes),
        // Bore optimizers — delegate to Whistle constraint templates
        STOPPER_POSITION => vec![crate::whistle::stopper_constraint()],
        HEADJOINT | BASIC_TAPER | BORE_DIAMETER_FROM_BOTTOM
        | BORE_SPACING_FROM_TOP
        | HOLE_AND_TAPER | HOLE_AND_BORE_DIAMETER_FROM_BOTTOM
        | HOLE_AND_BORE_SPACING | HOLE_AND_HEADJOINT
        | GLOBAL_HOLE_AND_TAPER | GLOBAL_HOLE_AND_BORE_DIAMETER_FROM_BOTTOM => {
            crate::whistle::create_default_constraints(objective_function_name, n_holes, inst)
                .constraint_list
        }
        _ => Vec::new(),
    }
}

/// AirstreamLength: single constraint with default bounds.
fn airstream_length_constraints() -> Vec<Constraint> {
    vec![Constraint {
        display_name: "Airstream length".to_string(),
        category: "Mouthpiece calibration".to_string(),
        constraint_type: ConstraintType::DIMENSIONAL,
        lower_bound: Some(wid_optimize::airstream_length::DEFAULT_AL_LOWER),
        upper_bound: Some(wid_optimize::airstream_length::DEFAULT_AL_UPPER),
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

/// FluteCalibration: airstream length + beta (2 constraints).
fn flute_calib_constraints() -> Vec<Constraint> {
    vec![
        Constraint {
            display_name: "Airstream length".to_string(),
            category: "Mouthpiece calibration".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: Some(wid_optimize::airstream_length::DEFAULT_AL_LOWER),
            upper_bound: Some(wid_optimize::airstream_length::DEFAULT_AL_UPPER),
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
/// Same ordering as Whistle — reused hole optimizer infrastructure.
fn hole_position_constraints(n_holes: u32) -> Vec<Constraint> {
    // Reuse Whistle's hole_position_constraints — identical structure
    crate::whistle::create_default_constraints(
        crate::whistle::HOLE_POSITION,
        n_holes,
        None,
    )
    .constraint_list
}

/// Hole size constraints: diameters only (N total).
fn hole_size_constraints(n_holes: u32) -> Vec<Constraint> {
    crate::whistle::create_default_constraints(
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
