//! Bore geometry optimization (diameters, spacings, positions, taper).
//!
//! This module provides standalone, merged, and global bore optimizers that
//! adjust bore profile parameters while keeping hole geometry fixed (standalone)
//! or jointly optimizing holes and bore (merged/global).
//!
//! # Architecture
//!
//! Three tiers of bore optimizers, matching Java WIDesigner's class hierarchy:
//!
//! 1. **Standalone** — optimize bore only, holes unchanged. Small dimension count.
//! 2. **Merged** — jointly optimize holes + bore in one BOBYQA run. 10-25 dims.
//! 3. **Global** — DIRECT-C global search → BOBYQA local refinement for merged objectives.
//!
//! Shared infrastructure:
//! - `evaluate_norm()` — compile + evaluate + weighted RMS error
//! - `run_bobyqa()` / `run_1d_or_nd()` — BOBYQA with optional Brent dispatch for 1D
//! - `set_merged_bore_geometry()` — apply hole + bore geometry with stale point cleanup
//!
//! # Solver dispatch
//!
//! `run_1d_or_nd` matches Java's `BaseObjectiveFunction.getOptimizer()`:
//! - 1D → Brent (rel_tol=1e-4, abs_tol=1e-4, matching Java's `BrentOptimizer(1e-4, 1e-4)`)
//! - Multi-dim → BOBYQA (2n+1 interpolation points)
//!
//! # Standalone optimizers
//!
//! | Function | Dims | Solver | Java class |
//! |----------|------|--------|------------|
//! | `optimize_bore_diameter_from_top` | N | auto | `BoreDiameterFromTopObjectiveFunction` |
//! | `optimize_bore_diameter_from_bottom` | M | auto | `BoreDiameterFromBottomObjectiveFunction` |
//! | `optimize_bore_spacing_from_top` | N | auto | `BoreSpacingFromTopObjectiveFunction` |
//! | `optimize_bore_position` | M | auto | `BorePositionObjectiveFunction` |
//! | `optimize_basic_taper` | 2 | BOBYQA | `BasicTaperObjectiveFunction` |
//! | `optimize_stopper_position` | 1 | Brent | `StopperPositionObjectiveFunction` |
//!
//! N = `find_head_point(inst)`, M = `n_bore - find_body_top(inst) - 1` (standalone),
//! M = `n_bore - n_unchanged` (BoreDiameter).
//!
//! # Merged optimizers
//!
//! | Function | Components | BoreLengthAdjust | max_eval | stopping |
//! |----------|------------|-----------------|----------|----------|
//! | `bore_from_bottom` | BorePos + BoreDiaFromBottom | — | 40K | 0.8e-6 |
//! | `headjoint` | StopperPos + BoreDiaFromTop | — | 40K | default |
//! | `hole_and_taper` | HolePos + HoleSize + BasicTaper | MOVE_BOTTOM | 20K | default |
//! | `hole_and_bore_diameter_from_top` | HolePos + HoleSize + BoreDiaFromTop | PRESERVE_TAPER | 50K | default |
//! | `hole_and_bore_diameter_from_bottom` | HolePos + HoleSize + BoreDiaFromBottom | MOVE_BOTTOM | 50K | default |
//! | `hole_and_bore_spacing` | HolePos + HoleSize + BoreSpacing | PRESERVE_TAPER | 50K | 0.9e-6 |
//! | `hole_and_bore_position` | HolePos + HoleSize + BorePos | PRESERVE_BELL | 50K | 0.9e-6 |
//! | `hole_and_bore_from_bottom` | HolePos + HoleSize + BorePos + BoreDia | PRESERVE_BELL | 60K | 0.9e-6 |
//! | `hole_and_headjoint` | HolePos + HoleSize + Stopper + BoreDia | PRESERVE_TAPER | 50K | default |
//!
//! Merged geometry vector layout: `[hole_position_dims..., hole_size_dims..., bore_dims...]`
//!
//! # Stale bore point cleanup
//!
//! When BOBYQA reuses a work instrument between evaluations, bore points from
//! a previous evaluation may persist above the new bore end. Before applying
//! new bore geometry, `set_merged_bore_geometry` removes these stale points:
//! ```text
//! raw.bore_points.retain(|bp| bp.position <= new_bore_end + epsilon)
//! ```
//!
//! # Global optimizers
//!
//! Two-stage DIRECT-C → BOBYQA for merged hole + bore objectives.
//! Dispatched from `wid-session` for Global* optimizer keys.

use bobyqa_impl::{BobyqaProgress, bobyqa_minimize, bobyqa_minimize_with_callback};
use wid_compile::{
    BoreLengthAdjust, compile,
    get_bore_diameter_from_bottom, get_bore_diameter_from_top,
    get_bore_position, get_bore_spacing_from_top,
    get_basic_taper, get_stopper_position,
    get_hole_diameters, get_hole_geometry_position,
    set_bore_diameter_from_bottom, set_bore_diameter_from_top,
    set_bore_position, set_bore_spacing_from_top,
    set_basic_taper, set_stopper_position,
    set_hole_diameters, set_hole_positions_adjusted,
    clamp_bore_spacing_upper_bounds,
};
use wid_eval::{CalculatorParams, calculate_error_vector};
use wid_physics::PhysicalParameters;
use wid_types::{Constraints, InstrumentRaw, Tuning};

use crate::{OptimizationResult, calc_norm, fingering_weights};
use crate::brent_min::brent_minimize;
use crate::global_optimize::optimize_global_with_progress;

// ── Shared evaluation helper ────────────────────────────────────

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

// ── Shared optimization runners ─────────────────────────────────

/// Run BOBYQA with optional progress callback and custom stopping trust radius.
fn run_bobyqa(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial: &[f64],
    lower: &[f64],
    upper: &[f64],
    max_eval: usize,
    stopping_trust_override: Option<f64>,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> Option<bobyqa_impl::BobyqaResult> {
    let n_dims = lower.len();
    let n_interp = 2 * n_dims + 1;
    let (initial_trust, stopping_trust) = crate::compute_trust_radius(lower, upper);
    let stopping = stopping_trust_override.unwrap_or(stopping_trust);

    match on_progress {
        Some(cb) => bobyqa_minimize_with_callback(
            f, initial, lower, upper, n_interp, initial_trust, stopping, max_eval, cb,
        ),
        None => bobyqa_minimize(
            f, initial, lower, upper, n_interp, initial_trust, stopping, max_eval,
        ),
    }
}

/// Auto-select Brent (1D) or BOBYQA (multi-dim), matching Java's
/// `BaseObjectiveFunction.getOptimizer()`.
///
/// Java uses `BrentOptimizer` when `nrDimensions == 1` and `BOBYQAOptimizer`
/// otherwise. This function wraps the same `FnMut(&[f64]) -> f64` objective
/// for both paths, so callers don't need separate closures.
fn run_1d_or_nd(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial: &[f64],
    lower: &[f64],
    upper: &[f64],
    max_eval: usize,
    stopping_trust_override: Option<f64>,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> Option<bobyqa_impl::BobyqaResult> {
    if lower.len() == 1 {
        // 1D → Brent optimizer (matches Java BrentOptimizer dispatch).
        // Always use Brent for 1D regardless of progress callback,
        // since BOBYQA doesn't reliably handle 1D optimization.
        let start = initial[0].clamp(lower[0], upper[0]);
        let mut buf = [0.0f64; 1];
        let brent_result = brent_minimize(
            &mut |x| { buf[0] = x; f(&buf) },
            lower[0], upper[0], start, 1e-4, 1e-4, max_eval,
        );
        brent_result.map(|(x_min, f_min)| bobyqa_impl::BobyqaResult {
            point: vec![x_min],
            value: f_min,
            evaluations: 0,
        })
    } else {
        // Multi-dim → BOBYQA
        run_bobyqa(f, initial, lower, upper, max_eval, stopping_trust_override, on_progress)
    }
}

/// Build OptimizationResult from BOBYQA result.
fn make_result(
    bobyqa_result: Option<bobyqa_impl::BobyqaResult>,
    initial_norm: f64,
    initial_geometry: Vec<f64>,
) -> OptimizationResult {
    match bobyqa_result {
        Some(r) => OptimizationResult {
            initial_norm,
            final_norm: r.value,
            evaluations: r.evaluations,
            initial_geometry,
            final_geometry: r.point,
        },
        None => OptimizationResult {
            initial_norm,
            final_norm: initial_norm,
            evaluations: 0,
            initial_geometry: initial_geometry.clone(),
            final_geometry: initial_geometry,
        },
    }
}

// ══════════════════════════════════════════════════════════════════
// STANDALONE BORE OPTIMIZERS
// ══════════════════════════════════════════════════════════════════

// ── Bore diameter from top ──────────────────────────────────────

/// Optimize bore diameter ratios from the headjoint downward.
pub fn optimize_bore_diameter_from_top(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_bore_diameter_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

/// Like [`optimize_bore_diameter_from_top`], but with a progress callback.
pub fn optimize_bore_diameter_from_top_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_bore_diameter_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_bore_diameter_from_top_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_bore_diameter_from_top(instrument, n_changed);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let max_eval = crate::max_evaluations(lower.len());
    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_1d_or_nd(
        &mut |point: &[f64]| {
            set_bore_diameter_from_top(&mut work, point, n_changed);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper, max_eval, None, on_progress,
    );

    if let Some(ref r) = result {
        set_bore_diameter_from_top(instrument, &r.point, n_changed);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Bore diameter from bottom ───────────────────────────────────

/// Optimize bore diameter ratios from the bell upward.
pub fn optimize_bore_diameter_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
) -> OptimizationResult {
    optimize_bore_diameter_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params, n_unchanged, None,
    )
}

pub fn optimize_bore_diameter_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_bore_diameter_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params, n_unchanged, Some(on_progress),
    )
}

fn optimize_bore_diameter_from_bottom_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_bore_diameter_from_bottom(instrument, n_unchanged);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let max_eval = crate::max_evaluations(lower.len());
    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_1d_or_nd(
        &mut |point: &[f64]| {
            set_bore_diameter_from_bottom(&mut work, point, n_unchanged);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper, max_eval, None, on_progress,
    );

    if let Some(ref r) = result {
        set_bore_diameter_from_bottom(instrument, &r.point, n_unchanged);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Bore spacing from top ───────────────────────────────────────

/// Optimize bore point spacings from the top.
pub fn optimize_bore_spacing_from_top(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_bore_spacing_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

pub fn optimize_bore_spacing_from_top_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_bore_spacing_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_bore_spacing_from_top_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let mut upper = constraints.upper_bounds();

    // Clamp upper bounds to prevent bore point reordering.
    clamp_bore_spacing_upper_bounds(instrument, n_changed, &mut upper);

    let raw_geom = get_bore_spacing_from_top(instrument, n_changed);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let max_eval = crate::max_evaluations(lower.len());
    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_1d_or_nd(
        &mut |point: &[f64]| {
            set_bore_spacing_from_top(&mut work, point, n_changed);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper, max_eval, None, on_progress,
    );

    if let Some(ref r) = result {
        set_bore_spacing_from_top(instrument, &r.point, n_changed);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Bore position ───────────────────────────────────────────────

/// Optimize bore point positions (fractional).
pub fn optimize_bore_position(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> OptimizationResult {
    optimize_bore_position_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, None,
    )
}

pub fn optimize_bore_position_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_bore_position_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, Some(on_progress),
    )
}

fn optimize_bore_position_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_bore_position(instrument, n_unchanged, bottom_fixed);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let max_eval = crate::max_evaluations(lower.len());
    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_1d_or_nd(
        &mut |point: &[f64]| {
            set_bore_position(&mut work, point, n_unchanged, bottom_fixed);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper, max_eval, None, on_progress,
    );

    if let Some(ref r) = result {
        set_bore_position(instrument, &r.point, n_unchanged, bottom_fixed);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Basic taper (2D) ────────────────────────────────────────────

/// Optimize basic 2D taper profile using BOBYQA.
pub fn optimize_basic_taper(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_basic_taper_impl(instrument, tuning, constraints, params, calc_params, None)
}

pub fn optimize_basic_taper_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_basic_taper_impl(
        instrument, tuning, constraints, params, calc_params, Some(on_progress),
    )
}

fn optimize_basic_taper_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_basic_taper(instrument);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let max_eval = crate::max_evaluations(lower.len());
    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            let taper = [point[0], point[1]];
            set_basic_taper(&mut work, &taper);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper, max_eval, None, on_progress,
    );

    if let Some(ref r) = result {
        let taper = [r.point[0], r.point[1]];
        set_basic_taper(instrument, &taper);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Stopper position (1D Brent) ─────────────────────────────────

/// Optimize flute stopper position using Brent minimizer.
pub fn optimize_stopper_position(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    preserve_taper: bool,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let current = get_stopper_position(instrument);
    let start = current.clamp(lower[0], upper[0]);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);
    let initial_geometry = vec![start];

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = brent_minimize(
        &mut |x| {
            set_stopper_position(&mut work, x, preserve_taper);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        lower[0],
        upper[0],
        start,
        1e-6,
        1e-14,
        1000,
    );

    match result {
        Some((x_opt, f_opt)) => {
            set_stopper_position(instrument, x_opt, preserve_taper);
            OptimizationResult {
                initial_norm,
                final_norm: f_opt,
                evaluations: 0, // Brent doesn't report this
                initial_geometry,
                final_geometry: vec![x_opt],
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

// ══════════════════════════════════════════════════════════════════
// MERGED BORE OPTIMIZERS
// ══════════════════════════════════════════════════════════════════

// ── Merged geometry helpers ─────────────────────────────────────

/// Get merged hole + bore geometry vector.
///
/// Layout: `[hole_position_dims..., hole_size_dims..., bore_dims...]`
fn get_merged_bore_geometry(
    raw: &InstrumentRaw,
    bore_getter: &dyn Fn(&InstrumentRaw) -> Vec<f64>,
) -> Vec<f64> {
    let mut geom = get_hole_geometry_position(raw);
    geom.extend(get_hole_diameters(raw));
    geom.extend(bore_getter(raw));
    geom
}

/// Set merged hole + bore geometry, applying components in order:
/// 1. Hole positions (with BoreLengthAdjust)
/// 2. Hole diameters
/// 3. Bore geometry
///
/// Includes stale bore point cleanup (retain pattern).
fn set_merged_bore_geometry(
    raw: &mut InstrumentRaw,
    geom: &[f64],
    n_holes: usize,
    adjust: BoreLengthAdjust,
    bore_setter: &dyn Fn(&mut InstrumentRaw, &[f64]),
) {
    let n_pos = n_holes + 1;
    let n_size = n_holes;
    let hole_pos = &geom[..n_pos];
    let hole_size = &geom[n_pos..n_pos + n_size];
    let bore_dims = &geom[n_pos + n_size..];

    // Save foot diameter for MOVE_BOTTOM semantics
    let m = raw.length_type.to_metres();
    let foot_diameter = raw
        .bore_points
        .iter()
        .max_by(|a, b| a.bore_position.partial_cmp(&b.bore_position).unwrap())
        .map(|bp| bp.bore_diameter * m)
        .unwrap_or(0.019050);

    // 1. Set hole positions with bore length adjustment
    set_hole_positions_adjusted(raw, hole_pos, adjust);

    let new_bore_end = hole_pos[0];

    // Remove stale bore points beyond new bore end
    raw.bore_points
        .retain(|bp| bp.bore_position * m <= new_bore_end + 1e-9);

    // For MOVE_BOTTOM: restore foot diameter
    if adjust == BoreLengthAdjust::MoveBottom {
        if let Some(last_bp) = raw
            .bore_points
            .iter_mut()
            .max_by(|a, b| a.bore_position.partial_cmp(&b.bore_position).unwrap())
        {
            last_bp.bore_diameter = foot_diameter / m;
        }
    }

    // 2. Set hole diameters
    set_hole_diameters(raw, hole_size);

    // 3. Set bore geometry
    bore_setter(raw, bore_dims);
}

/// Clamp geometry values to bounds.
fn clamp_geometry(raw_geom: &[f64], lower: &[f64], upper: &[f64]) -> Vec<f64> {
    raw_geom
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            if i < lower.len() {
                v.clamp(lower[i], upper[i])
            } else {
                v
            }
        })
        .collect()
}

// ── BoreFromBottom (BorePosition + BoreDiaFromBottom) ────────────

/// Optimize bore position + diameter from bottom. 40K evals, 0.8e-6 stopping.
pub fn optimize_bore_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> OptimizationResult {
    optimize_bore_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, None,
    )
}

pub fn optimize_bore_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_bore_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, Some(on_progress),
    )
}

fn optimize_bore_from_bottom_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    // Merged geometry: [bore_position_dims..., bore_dia_from_bottom_dims...]
    let pos_geom = get_bore_position(instrument, n_unchanged, bottom_fixed);
    let dia_geom = get_bore_diameter_from_bottom(instrument, n_unchanged);
    let n_pos = pos_geom.len();

    let mut raw_geom = pos_geom;
    raw_geom.extend(dia_geom);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_bore_position(&mut work, &point[..n_pos], n_unchanged, bottom_fixed);
            set_bore_diameter_from_bottom(&mut work, &point[n_pos..], n_unchanged);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        40_000,
        Some(0.8e-6),
        on_progress,
    );

    if let Some(ref r) = result {
        set_bore_position(instrument, &r.point[..n_pos], n_unchanged, bottom_fixed);
        set_bore_diameter_from_bottom(instrument, &r.point[n_pos..], n_unchanged);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── Headjoint (StopperPosition + BoreDiaFromTop) ────────────────

/// Optimize stopper position + bore diameter from top. 40K evals.
pub fn optimize_headjoint(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_headjoint_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

pub fn optimize_headjoint_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_headjoint_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_headjoint_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    // Merged geometry: [stopper_distance, bore_dia_ratios...]
    let stopper = get_stopper_position(instrument);
    let dia_geom = get_bore_diameter_from_top(instrument, n_changed);

    let mut raw_geom = vec![stopper];
    raw_geom.extend(dia_geom);
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_stopper_position(&mut work, point[0], true);
            set_bore_diameter_from_top(&mut work, &point[1..], n_changed);
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        40_000,
        None,
        on_progress,
    );

    if let Some(ref r) = result {
        set_stopper_position(instrument, r.point[0], true);
        set_bore_diameter_from_top(instrument, &r.point[1..], n_changed);
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndTaper (Holes + BasicTaper) ───────────────────────────

/// Optimize holes + basic taper. 20K evals, MOVE_BOTTOM.
pub fn optimize_hole_and_taper(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_hole_and_taper_impl(instrument, tuning, constraints, params, calc_params, None)
}

pub fn optimize_hole_and_taper_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_taper_impl(
        instrument, tuning, constraints, params, calc_params, Some(on_progress),
    )
}

fn optimize_hole_and_taper_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        let t = get_basic_taper(raw);
        vec![t[0], t[1]]
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| {
                    let taper = [bore_dims[0], bore_dims[1]];
                    set_basic_taper(raw, &taper);
                },
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        20_000,
        None,
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::MoveBottom,
            &|raw, bore_dims| {
                let taper = [bore_dims[0], bore_dims[1]];
                set_basic_taper(raw, &taper);
            },
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndBoreDiameterFromTop ──────────────────────────────────

/// Optimize holes + bore diameter from top. 50K evals, PRESERVE_TAPER.
pub fn optimize_hole_and_bore_diameter_from_top(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_hole_and_bore_diameter_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

pub fn optimize_hole_and_bore_diameter_from_top_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_bore_diameter_from_top_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_hole_and_bore_diameter_from_top_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        get_bore_diameter_from_top(raw, n_changed)
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveTaper,
                &|raw, bore_dims| set_bore_diameter_from_top(raw, bore_dims, n_changed),
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        None,
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::PreserveTaper,
            &|raw, bore_dims| set_bore_diameter_from_top(raw, bore_dims, n_changed),
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndBoreDiameterFromBottom ────────────────────────────────

/// Optimize holes + bore diameter from bottom. 50K evals, MOVE_BOTTOM.
pub fn optimize_hole_and_bore_diameter_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
) -> OptimizationResult {
    optimize_hole_and_bore_diameter_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params, n_unchanged, None,
    )
}

pub fn optimize_hole_and_bore_diameter_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_bore_diameter_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params, n_unchanged, Some(on_progress),
    )
}

fn optimize_hole_and_bore_diameter_from_bottom_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        get_bore_diameter_from_bottom(raw, n_unchanged)
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| set_bore_diameter_from_bottom(raw, bore_dims, n_unchanged),
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        None,
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::MoveBottom,
            &|raw, bore_dims| set_bore_diameter_from_bottom(raw, bore_dims, n_unchanged),
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndBoreSpacing ──────────────────────────────────────────

/// Optimize holes + bore spacing from top. 50K evals, 0.9e-6 stopping, PRESERVE_TAPER.
pub fn optimize_hole_and_bore_spacing(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_hole_and_bore_spacing_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

pub fn optimize_hole_and_bore_spacing_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_bore_spacing_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_hole_and_bore_spacing_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let mut upper = constraints.upper_bounds();

    // Clamp bore spacing upper bounds (only the bore portion at end of vector)
    let n_hole_dims = 2 * n_holes + 1;
    if upper.len() > n_hole_dims {
        clamp_bore_spacing_upper_bounds(
            instrument, n_changed, &mut upper[n_hole_dims..],
        );
    }

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        get_bore_spacing_from_top(raw, n_changed)
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveTaper,
                &|raw, bore_dims| set_bore_spacing_from_top(raw, bore_dims, n_changed),
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        Some(0.9e-6),
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::PreserveTaper,
            &|raw, bore_dims| set_bore_spacing_from_top(raw, bore_dims, n_changed),
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndBorePosition ─────────────────────────────────────────

/// Optimize holes + bore position. 50K evals, 0.9e-6 stopping, PRESERVE_BELL.
pub fn optimize_hole_and_bore_position(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> OptimizationResult {
    optimize_hole_and_bore_position_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, None,
    )
}

pub fn optimize_hole_and_bore_position_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_bore_position_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, Some(on_progress),
    )
}

fn optimize_hole_and_bore_position_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        get_bore_position(raw, n_unchanged, bottom_fixed)
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveBell,
                &|raw, bore_dims| set_bore_position(raw, bore_dims, n_unchanged, bottom_fixed),
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        Some(0.9e-6),
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::PreserveBell,
            &|raw, bore_dims| set_bore_position(raw, bore_dims, n_unchanged, bottom_fixed),
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndBoreFromBottom (4 components) ────────────────────────

/// Optimize holes + bore position + bore diameter from bottom.
/// 60K evals, 0.9e-6 stopping, PRESERVE_BELL.
pub fn optimize_hole_and_bore_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> OptimizationResult {
    optimize_hole_and_bore_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, None,
    )
}

pub fn optimize_hole_and_bore_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_bore_from_bottom_impl(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, Some(on_progress),
    )
}

fn optimize_hole_and_bore_from_bottom_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    // Merged: [hole_pos..., hole_size..., bore_pos..., bore_dia...]
    let bore_pos_geom = get_bore_position(instrument, n_unchanged, bottom_fixed);
    let n_bore_pos = bore_pos_geom.len();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        let mut g = get_bore_position(raw, n_unchanged, bottom_fixed);
        g.extend(get_bore_diameter_from_bottom(raw, n_unchanged));
        g
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveBell,
                &|raw, bore_dims| {
                    set_bore_position(raw, &bore_dims[..n_bore_pos], n_unchanged, bottom_fixed);
                    set_bore_diameter_from_bottom(raw, &bore_dims[n_bore_pos..], n_unchanged);
                },
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        60_000,
        Some(0.9e-6),
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::PreserveBell,
            &|raw, bore_dims| {
                set_bore_position(raw, &bore_dims[..n_bore_pos], n_unchanged, bottom_fixed);
                set_bore_diameter_from_bottom(raw, &bore_dims[n_bore_pos..], n_unchanged);
            },
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ── HoleAndHeadjoint (4 components) ─────────────────────────────

/// Optimize holes + stopper position + bore diameter from top.
/// 50K evals, PRESERVE_TAPER.
pub fn optimize_hole_and_headjoint(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
) -> OptimizationResult {
    optimize_hole_and_headjoint_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, None,
    )
}

pub fn optimize_hole_and_headjoint_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    optimize_hole_and_headjoint_impl(
        instrument, tuning, constraints, params, calc_params, n_changed, Some(on_progress),
    )
}

fn optimize_hole_and_headjoint_impl(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_changed: usize,
    on_progress: Option<&mut dyn FnMut(BobyqaProgress) -> bool>,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    // Merged: [hole_pos..., hole_size..., stopper_distance, bore_dia_ratios...]
    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        let mut g = vec![get_stopper_position(raw)];
        g.extend(get_bore_diameter_from_top(raw, n_changed));
        g
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = run_bobyqa(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveTaper,
                &|raw, bore_dims| {
                    set_stopper_position(raw, bore_dims[0], true);
                    set_bore_diameter_from_top(raw, &bore_dims[1..], n_changed);
                },
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        None,
        on_progress,
    );

    if let Some(ref r) = result {
        set_merged_bore_geometry(
            instrument, &r.point, n_holes,
            BoreLengthAdjust::PreserveTaper,
            &|raw, bore_dims| {
                set_stopper_position(raw, bore_dims[0], true);
                set_bore_diameter_from_top(raw, &bore_dims[1..], n_changed);
            },
        );
    }
    make_result(result, initial_norm, initial_geometry)
}

// ══════════════════════════════════════════════════════════════════
// GLOBAL BORE OPTIMIZERS
// ══════════════════════════════════════════════════════════════════

// ── GlobalHoleAndTaper ──────────────────────────────────────────

/// Global optimization of holes + basic taper (DIRECT-C → BOBYQA).
pub fn optimize_global_hole_and_taper(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
) -> OptimizationResult {
    optimize_global_hole_and_taper_with_progress(
        instrument, tuning, constraints, params, calc_params, &mut |_| true,
    )
}

pub fn optimize_global_hole_and_taper_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        let t = get_basic_taper(raw);
        vec![t[0], t[1]]
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = optimize_global_with_progress(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| {
                    let taper = [bore_dims[0], bore_dims[1]];
                    set_basic_taper(raw, &taper);
                },
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        20_000,
        on_progress,
    );

    match result {
        Some(r) => {
            set_merged_bore_geometry(
                instrument, &r.point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| {
                    let taper = [bore_dims[0], bore_dims[1]];
                    set_basic_taper(raw, &taper);
                },
            );
            OptimizationResult {
                initial_norm,
                final_norm: r.value,
                evaluations: r.direct_evaluations + r.bobyqa_evaluations,
                initial_geometry,
                final_geometry: r.point,
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

// ── GlobalHoleAndBoreDiameterFromBottom ──────────────────────────

/// Global optimization of holes + bore diameter from bottom.
pub fn optimize_global_hole_and_bore_diameter_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
) -> OptimizationResult {
    optimize_global_hole_and_bore_diameter_from_bottom_with_progress(
        instrument, tuning, constraints, params, calc_params, n_unchanged, &mut |_| true,
    )
}

pub fn optimize_global_hole_and_bore_diameter_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        get_bore_diameter_from_bottom(raw, n_unchanged)
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = optimize_global_with_progress(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| set_bore_diameter_from_bottom(raw, bore_dims, n_unchanged),
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        50_000,
        on_progress,
    );

    match result {
        Some(r) => {
            set_merged_bore_geometry(
                instrument, &r.point, n_holes,
                BoreLengthAdjust::MoveBottom,
                &|raw, bore_dims| set_bore_diameter_from_bottom(raw, bore_dims, n_unchanged),
            );
            OptimizationResult {
                initial_norm,
                final_norm: r.value,
                evaluations: r.direct_evaluations + r.bobyqa_evaluations,
                initial_geometry,
                final_geometry: r.point,
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

// ── GlobalHoleAndBoreFromBottom ──────────────────────────────────

/// Global optimization of holes + bore position + bore diameter from bottom.
pub fn optimize_global_hole_and_bore_from_bottom(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
) -> OptimizationResult {
    optimize_global_hole_and_bore_from_bottom_with_progress(
        instrument, tuning, constraints, params, calc_params,
        n_unchanged, bottom_fixed, &mut |_| true,
    )
}

pub fn optimize_global_hole_and_bore_from_bottom_with_progress(
    instrument: &mut InstrumentRaw,
    tuning: &Tuning,
    constraints: &Constraints,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    n_unchanged: usize,
    bottom_fixed: bool,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> OptimizationResult {
    let n_holes = instrument.holes.len();
    let weights = fingering_weights(&tuning.fingerings);
    let lower = constraints.lower_bounds();
    let upper = constraints.upper_bounds();

    let bore_pos_geom = get_bore_position(instrument, n_unchanged, bottom_fixed);
    let n_bore_pos = bore_pos_geom.len();

    let raw_geom = get_merged_bore_geometry(instrument, &|raw| {
        let mut g = get_bore_position(raw, n_unchanged, bottom_fixed);
        g.extend(get_bore_diameter_from_bottom(raw, n_unchanged));
        g
    });
    let initial_geometry = clamp_geometry(&raw_geom, &lower, &upper);
    let initial_norm = evaluate_norm(instrument, &tuning.fingerings, &weights, params, calc_params);

    let mut work = instrument.clone();
    let fingerings = tuning.fingerings.clone();

    let result = optimize_global_with_progress(
        &mut |point: &[f64]| {
            set_merged_bore_geometry(
                &mut work, point, n_holes,
                BoreLengthAdjust::PreserveBell,
                &|raw, bore_dims| {
                    set_bore_position(raw, &bore_dims[..n_bore_pos], n_unchanged, bottom_fixed);
                    set_bore_diameter_from_bottom(raw, &bore_dims[n_bore_pos..], n_unchanged);
                },
            );
            evaluate_norm(&work, &fingerings, &weights, params, calc_params)
        },
        &initial_geometry, &lower, &upper,
        60_000,
        on_progress,
    );

    match result {
        Some(r) => {
            set_merged_bore_geometry(
                instrument, &r.point, n_holes,
                BoreLengthAdjust::PreserveBell,
                &|raw, bore_dims| {
                    set_bore_position(raw, &bore_dims[..n_bore_pos], n_unchanged, bottom_fixed);
                    set_bore_diameter_from_bottom(raw, &bore_dims[n_bore_pos..], n_unchanged);
                },
            );
            OptimizationResult {
                initial_norm,
                final_norm: r.value,
                evaluations: r.direct_evaluations + r.bobyqa_evaluations,
                initial_geometry,
                final_geometry: r.point,
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

// ══════════════════════════════════════════════════════════════════
// TESTS
// ══════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use wid_compile::{find_body_top, find_head_point};
    use wid_physics::TemperatureType;
    use wid_types::{Constraint, ConstraintType, parse_instrument_xml, parse_tuning_xml};

    const PVC_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/instruments/SamplePVC-Whistle.xml");
    const PVC_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/WhistleStudy/tunings/SamplePVC-tuning.xml");
    const FLUTE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/instruments/SamplePVC-Flute.xml");
    const FLUTE_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/FluteStudy/tunings/D4-Equal.xml");

    fn default_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    fn bore_dia_from_top_constraints(n: usize) -> Constraints {
        let constraints: Vec<Constraint> = (0..n)
            .map(|_| Constraint {
                display_name: "bore ratio".to_string(),
                category: "Bore diameter".to_string(),
                constraint_type: ConstraintType::DIMENSIONLESS,
                lower_bound: Some(0.5),
                upper_bound: Some(2.0),
            })
            .collect();
        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Bore diameter from top".to_string(),
            objective_function_name: "BoreDiameterFromTopObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
            hole_groups: None,
        }
    }

    fn bore_dia_from_bottom_constraints(n: usize) -> Constraints {
        let constraints: Vec<Constraint> = (0..n)
            .map(|_| Constraint {
                display_name: "bore ratio".to_string(),
                category: "Bore diameter".to_string(),
                constraint_type: ConstraintType::DIMENSIONLESS,
                lower_bound: Some(0.5),
                upper_bound: Some(2.0),
            })
            .collect();
        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Bore diameter from bottom".to_string(),
            objective_function_name: "BoreDiameterFromBottomObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: constraints,
            hole_groups: None,
        }
    }

    fn basic_taper_constraints() -> Constraints {
        Constraints {
            name: "Default".to_string(),
            objective_display_name: "Basic taper".to_string(),
            objective_function_name: "BasicTaperObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: vec![
                Constraint {
                    display_name: "head fraction".to_string(),
                    category: "Basic taper".to_string(),
                    constraint_type: ConstraintType::DIMENSIONLESS,
                    lower_bound: Some(0.01),
                    upper_bound: Some(0.99),
                },
                Constraint {
                    display_name: "foot ratio".to_string(),
                    category: "Basic taper".to_string(),
                    constraint_type: ConstraintType::DIMENSIONLESS,
                    lower_bound: Some(0.5),
                    upper_bound: Some(2.0),
                },
            ],
            hole_groups: None,
        }
    }

    // ── Standalone bore optimizer tests ──────────────────────────

    #[test]
    fn bore_diameter_from_top_does_not_worsen() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let n_changed = find_head_point(&inst, "Head");
        let constraints = bore_dia_from_top_constraints(n_changed);

        let result = optimize_bore_diameter_from_top(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE, n_changed,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    #[test]
    fn bore_diameter_from_bottom_does_not_worsen() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        // Java uses getTopOfBody()+1 for n_unchanged
        let n_unchanged = find_body_top(&inst) + 1;
        let n_dims = inst.bore_points.len() - n_unchanged;
        let constraints = bore_dia_from_bottom_constraints(n_dims);

        let result = optimize_bore_diameter_from_bottom(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE, n_unchanged,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    #[test]
    fn basic_taper_does_not_worsen() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = basic_taper_constraints();

        let result = optimize_basic_taper(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    #[test]
    fn stopper_position_does_not_worsen() {
        let mut inst = parse_instrument_xml(FLUTE_XML).unwrap();
        let tuning = parse_tuning_xml(FLUTE_TUNING_XML).unwrap();
        let params = default_params();
        let constraints = Constraints {
            name: "Default".to_string(),
            objective_display_name: "Stopper position".to_string(),
            objective_function_name: "StopperPositionObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: vec![Constraint {
                display_name: "stopper distance".to_string(),
                category: "Stopper".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.005),
                upper_bound: Some(0.05),
            }],
            hole_groups: None,
        };

        let result = optimize_stopper_position(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::FLUTE, true,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    // ── Merged bore optimizer tests ─────────────────────────────

    fn merged_hole_constraints(n_holes: usize) -> Vec<Constraint> {
        let mut constraints = Vec::new();
        // Position: bore_end + N spacings
        constraints.push(Constraint {
            display_name: "bore end".to_string(),
            category: "Hole position".to_string(),
            constraint_type: ConstraintType::DIMENSIONAL,
            lower_bound: Some(0.2),
            upper_bound: Some(0.7),
        });
        for _ in 0..n_holes {
            constraints.push(Constraint {
                display_name: "spacing".to_string(),
                category: "Hole position".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.012),
                upper_bound: Some(0.04),
            });
        }
        // Size: N diameters
        for _ in 0..n_holes {
            constraints.push(Constraint {
                display_name: "diameter".to_string(),
                category: "Hole size".to_string(),
                constraint_type: ConstraintType::DIMENSIONAL,
                lower_bound: Some(0.004),
                upper_bound: Some(0.0091),
            });
        }
        constraints
    }

    #[test]
    fn hole_and_bore_diameter_from_bottom_does_not_worsen() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let n_unchanged = find_body_top(&inst) + 1;
        let n_bore_dims = inst.bore_points.len() - n_unchanged;

        let mut constraint_list = merged_hole_constraints(6);
        for _ in 0..n_bore_dims {
            constraint_list.push(Constraint {
                display_name: "bore ratio".to_string(),
                category: "Bore diameter".to_string(),
                constraint_type: ConstraintType::DIMENSIONLESS,
                lower_bound: Some(0.5),
                upper_bound: Some(2.0),
            });
        }

        let constraints = Constraints {
            name: "Default".to_string(),
            objective_display_name: "Holes + bore diameter from bottom".to_string(),
            objective_function_name: "HoleAndBoreDiameterFromBottomObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list,
            hole_groups: None,
        };

        let result = optimize_hole_and_bore_diameter_from_bottom(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE, n_unchanged,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    #[test]
    fn hole_and_taper_does_not_worsen() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();

        let mut constraint_list = merged_hole_constraints(6);
        // Basic taper: head_fraction + foot_ratio
        constraint_list.push(Constraint {
            display_name: "head fraction".to_string(),
            category: "Basic taper".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: Some(0.01),
            upper_bound: Some(0.99),
        });
        constraint_list.push(Constraint {
            display_name: "foot ratio".to_string(),
            category: "Basic taper".to_string(),
            constraint_type: ConstraintType::DIMENSIONLESS,
            lower_bound: Some(0.5),
            upper_bound: Some(2.0),
        });

        let constraints = Constraints {
            name: "Default".to_string(),
            objective_display_name: "Holes + basic taper".to_string(),
            objective_function_name: "HoleAndBasicTaperObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list,
            hole_groups: None,
        };

        let result = optimize_hole_and_taper(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE,
        );

        assert!(
            result.final_norm <= result.initial_norm * 1.01,
            "should not worsen: initial={}, final={}",
            result.initial_norm, result.final_norm,
        );
    }

    // ── Golden parity tests (WH-BORE-01, WH-BORE-02, RD-BORE-01) ──

    // Didgeridoo instrument + tuning for RD-BORE-01
    const DIDGE_XML: &str =
        include_str!("../../../../oracle/v2.6.0/ReedStudy/instruments/Didgeridoo-2stage-D2-D3.xml");
    const DIDGE_TUNING_XML: &str =
        include_str!("../../../../oracle/v2.6.0/ReedStudy/tunings/Didgeridoo-D2-D3-tuning.xml");

    // Golden norms from BoreOptDriver output
    // WH-BORE-01: BoreDiameterFromBottom on SamplePVC-Whistle (1D Brent)
    const WH_BORE_01_INITIAL_NORM: f64 = 15900.000398470573;
    const WH_BORE_01_FINAL_NORM: f64 = 6883.450271500735;
    const WH_BORE_01_INITIAL_GEOMETRY: [f64; 1] = [1.0];
    const WH_BORE_01_FINAL_GEOMETRY: [f64; 1] = [0.7843269596368897];

    // WH-BORE-02: BoreDiameterFromTop on SamplePVC-Whistle (1D Brent)
    const WH_BORE_02_INITIAL_NORM: f64 = 24135.597275207845;
    const WH_BORE_02_FINAL_NORM: f64 = 24135.597275207845;
    const WH_BORE_02_INITIAL_GEOMETRY: [f64; 1] = [0.8403361344537815];
    const WH_BORE_02_FINAL_GEOMETRY: [f64; 1] = [0.999];

    // RD-BORE-01: BorePosition on Didgeridoo-2stage (3D BOBYQA)
    const RD_BORE_01_INITIAL_NORM: f64 = 234.7815305042882;
    const RD_BORE_01_FINAL_NORM: f64 = 199.003946410246;
    const RD_BORE_01_INITIAL_GEOMETRY: [f64; 3] =
        [1.554138879519837, 0.5009095173215202, 1.5033467813416743e-4];
    const RD_BORE_01_FINAL_GEOMETRY: [f64; 3] =
        [1.5501908768558668, 0.500918651829363, 1.471520119881214e-4];

    // ── WH-BORE-01: BoreDiameterFromBottom evaluation parity ──

    #[test]
    fn wh_bore_01_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        let rel_err = (norm - WH_BORE_01_INITIAL_NORM).abs() / WH_BORE_01_INITIAL_NORM;
        assert!(
            rel_err < 1e-6,
            "WH-BORE-01 initial norm: expected {WH_BORE_01_INITIAL_NORM}, got {norm} (rel_err={rel_err})"
        );
    }

    #[test]
    fn wh_bore_01_geometry_extraction_matches_golden() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let n_unchanged = find_body_top(&inst) + 1;
        let geom = get_bore_diameter_from_bottom(&inst, n_unchanged);
        assert_eq!(geom.len(), WH_BORE_01_INITIAL_GEOMETRY.len(),
            "dimension count mismatch");
        for (i, (got, expected)) in geom.iter().zip(&WH_BORE_01_INITIAL_GEOMETRY).enumerate() {
            let err = (got - expected).abs();
            assert!(err < 1e-10, "WH-BORE-01 geom[{i}]: expected {expected}, got {got}");
        }
    }

    #[test]
    fn wh_bore_01_final_geometry_norm_matches_golden() {
        // Apply golden-optimal geometry → evaluate → norm should match golden finalNorm
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_unchanged = find_body_top(&inst) + 1;

        set_bore_diameter_from_bottom(&mut inst, &WH_BORE_01_FINAL_GEOMETRY, n_unchanged);
        let norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        let rel_err = (norm - WH_BORE_01_FINAL_NORM).abs() / WH_BORE_01_FINAL_NORM;
        assert!(
            rel_err < 1e-6,
            "WH-BORE-01 final norm: expected {WH_BORE_01_FINAL_NORM}, got {norm} (rel_err={rel_err})"
        );
    }

    // ── WH-BORE-02: BoreDiameterFromTop evaluation parity ──

    #[test]
    fn wh_bore_02_geometry_extraction_matches_golden() {
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let n_changed = find_head_point(&inst, "Head");
        let geom = get_bore_diameter_from_top(&inst, n_changed);
        assert_eq!(geom.len(), WH_BORE_02_INITIAL_GEOMETRY.len(),
            "dimension count mismatch");
        for (i, (got, expected)) in geom.iter().zip(&WH_BORE_02_INITIAL_GEOMETRY).enumerate() {
            let err = (got - expected).abs();
            assert!(err < 1e-10, "WH-BORE-02 geom[{i}]: expected {expected}, got {got}");
        }
    }

    #[test]
    fn wh_bore_02_final_geometry_norm_matches_golden() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_changed = find_head_point(&inst, "Head");

        set_bore_diameter_from_top(&mut inst, &WH_BORE_02_FINAL_GEOMETRY, n_changed);
        let norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        // WH-BORE-02: the optimizer didn't improve (constrained at boundary),
        // so finalNorm == initialNorm. Check that applying the final geometry
        // still reproduces the golden norm.
        let rel_err = (norm - WH_BORE_02_FINAL_NORM).abs() / WH_BORE_02_FINAL_NORM;
        assert!(
            rel_err < 1e-6,
            "WH-BORE-02 final norm: expected {WH_BORE_02_FINAL_NORM}, got {norm} (rel_err={rel_err})"
        );
    }

    // ── RD-BORE-01: BorePosition evaluation parity ──

    #[test]
    fn rd_bore_01_initial_norm_matches_golden() {
        let inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let tuning = parse_tuning_xml(DIDGE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);

        let norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::REED,
        );

        let rel_err = (norm - RD_BORE_01_INITIAL_NORM).abs() / RD_BORE_01_INITIAL_NORM;
        assert!(
            rel_err < 1e-5,
            "RD-BORE-01 initial norm: expected {RD_BORE_01_INITIAL_NORM}, got {norm} (rel_err={rel_err})"
        );
    }

    #[test]
    fn rd_bore_01_geometry_extraction_matches_golden() {
        let inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let n_unchanged = find_body_top(&inst) + 1;
        let geom = get_bore_position(&inst, n_unchanged, false);
        assert_eq!(geom.len(), RD_BORE_01_INITIAL_GEOMETRY.len(),
            "dimension count mismatch: expected {}, got {}",
            RD_BORE_01_INITIAL_GEOMETRY.len(), geom.len());
        for (i, (got, expected)) in geom.iter().zip(&RD_BORE_01_INITIAL_GEOMETRY).enumerate() {
            let rel_err = if expected.abs() > 1e-10 {
                (got - expected).abs() / expected.abs()
            } else {
                (got - expected).abs()
            };
            assert!(rel_err < 1e-6, "RD-BORE-01 geom[{i}]: expected {expected}, got {got}");
        }
    }

    #[test]
    fn rd_bore_01_final_geometry_norm_matches_golden() {
        let mut inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let tuning = parse_tuning_xml(DIDGE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_unchanged = find_body_top(&inst) + 1;

        set_bore_position(&mut inst, &RD_BORE_01_FINAL_GEOMETRY, n_unchanged, false);
        let norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::REED,
        );

        // Allow 1% tolerance for BOBYQA trajectory differences
        let rel_err = (norm - RD_BORE_01_FINAL_NORM).abs() / RD_BORE_01_FINAL_NORM;
        assert!(
            rel_err < 0.01,
            "RD-BORE-01 final norm: expected {RD_BORE_01_FINAL_NORM}, got {norm} (rel_err={rel_err})"
        );
    }

    // ── Mutation tests ──────────────────────────────────────────────
    // Verify that perturbing the optimal geometry worsens the norm,
    // confirming the objective function is sensitive to bore parameters.

    #[test]
    fn wh_bore_01_mutation_worsens_norm() {
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_unchanged = find_body_top(&inst) + 1;

        // Apply optimal geometry
        set_bore_diameter_from_bottom(&mut inst, &WH_BORE_01_FINAL_GEOMETRY, n_unchanged);
        let optimal_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        // Perturb: increase ratio by 10%
        let perturbed: Vec<f64> = WH_BORE_01_FINAL_GEOMETRY.iter()
            .map(|v| v * 1.10)
            .collect();
        set_bore_diameter_from_bottom(&mut inst, &perturbed, n_unchanged);
        let perturbed_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        assert!(
            perturbed_norm > optimal_norm * 1.01,
            "mutation should worsen norm: optimal={optimal_norm}, perturbed={perturbed_norm}"
        );
    }

    #[test]
    fn rd_bore_01_mutation_worsens_norm() {
        let mut inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let tuning = parse_tuning_xml(DIDGE_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_unchanged = find_body_top(&inst) + 1;

        // Apply optimal geometry
        set_bore_position(&mut inst, &RD_BORE_01_FINAL_GEOMETRY, n_unchanged, false);
        let optimal_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::REED,
        );

        // Perturb: shift bottom position by +5%
        let mut perturbed = RD_BORE_01_FINAL_GEOMETRY.to_vec();
        perturbed[0] *= 1.05;
        set_bore_position(&mut inst, &perturbed, n_unchanged, false);
        let perturbed_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::REED,
        );

        assert!(
            perturbed_norm > optimal_norm * 1.001,
            "mutation should worsen norm: optimal={optimal_norm}, perturbed={perturbed_norm}"
        );
    }

    #[test]
    fn wh_bore_02_mutation_changes_norm() {
        // Even though BoreDiameterFromTop hit bounds, perturbing the geometry
        // away from the optimal within the feasible region should change the norm.
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let weights = fingering_weights(&tuning.fingerings);
        let n_changed = find_head_point(&inst, "Head");

        // Apply optimal geometry (which is at the lower bound)
        set_bore_diameter_from_top(&mut inst, &WH_BORE_02_FINAL_GEOMETRY, n_changed);
        let optimal_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        // Perturb: change ratio to 0.7 (well within [0.5, 1.0] range)
        set_bore_diameter_from_top(&mut inst, &[0.7], n_changed);
        let perturbed_norm = evaluate_norm(
            &inst, &tuning.fingerings, &weights, &params, &CalculatorParams::WHISTLE,
        );

        // Norm should be different (worse or better doesn't matter here — it's
        // a boundary-constrained result, so perturbation might improve or worsen)
        assert!(
            (perturbed_norm - optimal_norm).abs() > 1.0,
            "mutation should change norm: optimal={optimal_norm}, perturbed={perturbed_norm}"
        );
    }

    // ── Constraint dimension count tests ─────────────────────────

    #[test]
    fn bore_diameter_from_bottom_dimension_count() {
        // SamplePVC-Whistle: 3 bore points, getTopOfBody()=1, n_unchanged=2, n_dims=1
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let n_unchanged = find_body_top(&inst) + 1;
        let n_dims = inst.bore_points.len() - n_unchanged;
        assert_eq!(n_dims, 1,
            "SamplePVC: expected 1 bore dim from bottom, got {n_dims}");
    }

    #[test]
    fn bore_diameter_from_top_dimension_count() {
        // SamplePVC-Whistle: 3 bore points, getLowestPoint("Head")=1, n_dims=1
        let inst = parse_instrument_xml(PVC_XML).unwrap();
        let n_changed = find_head_point(&inst, "Head");
        assert_eq!(n_changed, 1,
            "SamplePVC: expected 1 bore dim from top, got {n_changed}");
    }

    #[test]
    fn bore_position_dimension_count_didgeridoo() {
        // Didgeridoo-2stage: 5 bore points, getTopOfBody()=1, n_unchanged=2,
        // bottomPointUnchanged=false → n_dims = 5-2-0 = 3
        let inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let n_unchanged = find_body_top(&inst) + 1;
        let n_dims = inst.bore_points.len() - n_unchanged;
        assert_eq!(n_dims, 3,
            "Didgeridoo-2stage: expected 3 bore position dims, got {n_dims}");
    }

    #[test]
    fn bore_position_geometry_matches_constraint_count() {
        // The geometry vector from get_bore_position should match n_dims
        let inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let n_unchanged = find_body_top(&inst) + 1;
        let geom = get_bore_position(&inst, n_unchanged, false);
        assert_eq!(geom.len(), 3,
            "Didgeridoo: get_bore_position should return 3 dims, got {}", geom.len());
    }

    // ── Optimization convergence tests ──────────────────────────

    #[test]
    fn wh_bore_01_optimization_converges() {
        // Full optimization with golden bounds [0.5, 1.0] — now uses Brent for 1D,
        // matching Java's dispatch. Brent can handle initial point at upper bound.
        let mut inst = parse_instrument_xml(PVC_XML).unwrap();
        let tuning = parse_tuning_xml(PVC_TUNING_XML).unwrap();
        let params = default_params();
        let n_unchanged = find_body_top(&inst) + 1;
        let n_dims = inst.bore_points.len() - n_unchanged;

        let constraints = Constraints {
            name: "Default".to_string(),
            objective_display_name: "Bore diameter from bottom".to_string(),
            objective_function_name: "BoreDiameterFromBottomObjectiveFunction".to_string(),
            number_of_holes: 6,
            constraint_list: (0..n_dims).map(|_| Constraint {
                display_name: "bore ratio".to_string(),
                category: "Bore diameter".to_string(),
                constraint_type: ConstraintType::DIMENSIONLESS,
                lower_bound: Some(0.5),
                upper_bound: Some(1.0),
            }).collect(),
            hole_groups: None,
        };

        let result = optimize_bore_diameter_from_bottom(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::WHISTLE, n_unchanged,
        );

        // With Brent for 1D, should converge to golden ~6883 from initial ~15900
        assert!(result.final_norm < result.initial_norm * 0.95,
            "bore_dia_from_bottom should significantly improve: {} -> {}",
            result.initial_norm, result.final_norm);
    }

    #[test]
    fn rd_bore_01_optimization_converges() {
        // BorePosition on Didgeridoo with oracle constraints
        let mut inst = parse_instrument_xml(DIDGE_XML).unwrap();
        let tuning = parse_tuning_xml(DIDGE_TUNING_XML).unwrap();
        let params = default_params();
        let n_unchanged = find_body_top(&inst) + 1;

        // Use oracle constraint bounds from DidgeridooConstraints-2stage.xml
        let constraints = Constraints {
            name: "Didgeridoo 2-stage".to_string(),
            objective_display_name: "Bore Position optimizer".to_string(),
            objective_function_name: "BorePositionObjectiveFunction".to_string(),
            number_of_holes: 0,
            constraint_list: vec![
                Constraint {
                    display_name: "Bottom bore position".to_string(),
                    category: "Bore point positions".to_string(),
                    constraint_type: ConstraintType::DIMENSIONAL,
                    lower_bound: Some(0.508),
                    upper_bound: Some(2.54),
                },
                Constraint {
                    display_name: "Relative position pt 3".to_string(),
                    category: "Bore point positions".to_string(),
                    constraint_type: ConstraintType::DIMENSIONLESS,
                    lower_bound: Some(0.01),
                    upper_bound: Some(0.99),
                },
                Constraint {
                    display_name: "Relative position pt 4".to_string(),
                    category: "Bore point positions".to_string(),
                    constraint_type: ConstraintType::DIMENSIONLESS,
                    lower_bound: Some(0.0),
                    upper_bound: Some(0.001),
                },
            ],
            hole_groups: None,
        };

        let result = optimize_bore_position(
            &mut inst, &tuning, &constraints, &params,
            &CalculatorParams::REED, n_unchanged, false,
        );

        assert!(result.final_norm < result.initial_norm,
            "bore_position should improve: {} -> {}",
            result.initial_norm, result.final_norm);
    }
}
