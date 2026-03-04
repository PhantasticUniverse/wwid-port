//! Optimization infrastructure for WIDesigner instrument tuning.
//!
//! This crate provides:
//! - Brent univariate minimizer (`brent_min`)
//! - BOBYQA multivariate minimizer (`bobyqa`)
//! - Fipple factor calibration (`fipple`)
//! - Hole geometry optimization (`hole_from_top`)
//! - Weighted norm calculation (`calc_norm`)

pub mod airstream_length;
pub mod beta;
pub mod brent_min;
pub mod fipple;
pub mod flute_calib;
pub mod hole_combined;
pub mod hole_from_top;
pub mod hole_position;
pub mod hole_size;
pub mod reed_calib;
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
