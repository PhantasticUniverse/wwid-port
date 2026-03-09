// Optimizer entry points inherently take many parameters (instrument, tuning, constraints, etc.).
#![allow(clippy::too_many_arguments)]
//! Optimization infrastructure for WIDesigner instrument tuning.
//!
//! This crate provides objective functions, calibrators, and optimizer dispatch
//! for all four study models (NAF, Whistle, Flute, Reed).
//!
//! # Optimizers
//!
//! - [`brent_min`] — Brent univariate minimizer (for 1D bore optimizers)
//! - [`bobyqa`](::bobyqa_impl) — BOBYQA multivariate (via standalone crate)
//! - [`global_optimize`] / [`multi_start`] — DIRECT-C → BOBYQA two-stage pipeline
//!
//! # Calibrators
//!
//! - [`fipple`] — NAF fipple factor (1D Brent)
//! - [`whistle_calib`] — window height, beta, or joint (BOBYQA)
//! - [`flute_calib`] — airstream length, beta, or joint (BOBYQA)
//! - [`reed_calib`] — alpha + beta joint (2D BOBYQA)
//!
//! # Hole objective functions
//!
//! - [`hole_from_top`] — hole positions from top of bore
//! - [`hole_group_from_top`] — grouped hole positions
//! - [`hole_size`] — hole diameters (NAF-specific trust radius)
//! - [`hole_position`] — absolute hole positions
//! - [`hole_combined`] — combined hole position + size
//!
//! # Bore and geometry
//!
//! - [`bore`] — bore diameter optimization (from top, bottom, or full)
//! - [`single_taper`] — single taper geometry (4 variants)
//! - [`window_height`] — window height optimization
//! - [`airstream_length`] — flute airstream length
//! - [`beta`] — beta factor optimization
//!
//! # Utilities
//!
//! - [`calc_norm`] — weighted L2 norm (matching Java `BaseObjectiveFunction.calcNorm()`)
//! - [`compute_trust_radius`] — BOBYQA trust region from bounds
//! - [`max_evaluations`] — evaluation budget formula

pub mod airstream_length;
pub mod global_optimize;
pub mod multi_start;
pub mod beta;
pub mod brent_min;
pub mod fipple;
pub mod flute_calib;
pub mod hole_combined;
pub mod hole_from_top;
pub mod hole_group_from_top;
pub mod hole_position;
pub mod hole_size;
pub mod reed_calib;
pub mod bore;
pub mod single_taper;
pub mod whistle_calib;
pub mod window_height;

/// Compute the weighted L2 norm (sum of weighted squared errors).
///
/// Only includes fingerings with `weight > 0`. This matches the Java
/// `BaseObjectiveFunction.calcNorm()`.
pub fn calc_norm(error_vector: &[f64], weights: &[i32]) -> f64 {
    let mut sum = 0.0;
    for i in 0..error_vector.len().min(weights.len()) {
        if weights[i] > 0 {
            sum += error_vector[i] * error_vector[i] * weights[i] as f64;
        }
    }
    sum
}

/// Extract optimization weights from fingerings.
///
/// Returns `weight` for each fingering, defaulting to 1 if not specified.
pub fn fingering_weights(fingerings: &[wid_types::Fingering]) -> Vec<i32> {
    fingerings
        .iter()
        .map(|f| f.optimization_weight.unwrap_or(1))
        .collect()
}

/// Result of a fipple factor calibration.
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    pub initial_fipple_factor: f64,
    pub final_fipple_factor: f64,
    pub initial_norm: f64,
    pub final_norm: f64,
}

/// Result of a whistle mouthpiece calibration (window height, beta, or both).
#[derive(Debug, Clone)]
pub struct WhistleCalibrationResult {
    pub initial_window_height: Option<f64>,
    pub final_window_height: Option<f64>,
    pub initial_beta: Option<f64>,
    pub final_beta: Option<f64>,
    pub initial_norm: f64,
    pub final_norm: f64,
}

/// Result of a flute mouthpiece calibration (airstream length, beta, or both).
#[derive(Debug, Clone)]
pub struct FluteCalibrationResult {
    pub initial_airstream_length: Option<f64>,
    pub final_airstream_length: Option<f64>,
    pub initial_beta: Option<f64>,
    pub final_beta: Option<f64>,
    pub initial_norm: f64,
    pub final_norm: f64,
}

/// Result of a reed mouthpiece calibration (alpha + beta jointly).
#[derive(Debug, Clone)]
pub struct ReedCalibrationResult {
    pub initial_alpha: Option<f64>,
    pub final_alpha: Option<f64>,
    pub initial_beta: Option<f64>,
    pub final_beta: Option<f64>,
    pub initial_norm: f64,
    pub final_norm: f64,
}

/// Compute BOBYQA initial and stopping trust region radii from bounds.
///
/// Matches Java `BaseObjectiveFunction.getInitialTrustRegionRadius()`:
/// - initial = max(0.1 * max_bound_range, min_half_bound_range)
/// - stopping = 1e-8 * initial
pub fn compute_trust_radius(lower: &[f64], upper: &[f64]) -> (f64, f64) {
    let mut max_range: f64 = 0.0;
    let mut min_radius: f64 = 1.0e-6;

    for i in 0..lower.len() {
        let diff = upper[i] - lower[i];
        if diff > 1.0e-7 && 0.5 * diff < min_radius {
            min_radius = 0.5 * diff;
        }
        if diff > max_range {
            max_range = diff;
        }
    }

    let initial = if min_radius > 0.1 * max_range {
        min_radius
    } else {
        0.1 * max_range
    };
    let stopping = 1.0e-8 * initial;
    (initial, stopping)
}

/// Compute max evaluations matching Java `HoleObjectiveFunction` pattern.
///
/// Java: `maxEvaluations = 20000 + (getNrDimensions() - 1) * 5000`
pub fn max_evaluations(n_dims: usize) -> usize {
    20000 + n_dims.saturating_sub(1) * 5000
}

/// Result of a hole geometry optimization.
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub initial_norm: f64,
    pub final_norm: f64,
    pub evaluations: usize,
    pub initial_geometry: Vec<f64>,
    pub final_geometry: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_norm_weighted() {
        let errors = [10.0, 20.0, 30.0];
        let weights = [1, 0, 2]; // middle excluded
        // 10^2 * 1 + 30^2 * 2 = 100 + 1800 = 1900
        assert_eq!(calc_norm(&errors, &weights), 1900.0);
    }

    #[test]
    fn calc_norm_all_weight_one() {
        let errors = [3.0, 4.0];
        let weights = [1, 1];
        // 9 + 16 = 25
        assert_eq!(calc_norm(&errors, &weights), 25.0);
    }

    #[test]
    fn calc_norm_empty() {
        assert_eq!(calc_norm(&[], &[]), 0.0);
    }
}
