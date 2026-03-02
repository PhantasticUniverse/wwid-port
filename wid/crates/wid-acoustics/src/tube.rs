//! Cylindrical and conical tube transfer matrices, radiation impedance.
//!
//! Transfer matrices follow the Lefebvre & Kergomard formulation for lossy
//! conical tubes, with the cylinder as a special case. Radiation impedance
//! uses the Padé approximants of Silva et al. (2008).

use num_complex::Complex64;
use wid_math::TransferMatrix;
use wid_physics::PhysicalParameters;

/// Minimum cone section length (metres). Prevents singularities.
pub const MINIMUM_CONE_LENGTH: f64 = 0.00001;

/// Cylinder transfer matrix with viscothermal losses.
///
/// Uses the complex propagation constant γ = (ε + i(1+ε))·k to model
/// wall losses in a cylindrical tube.
pub fn calc_cylinder_matrix(
    wave_number: f64,
    length: f64,
    radius: f64,
    params: &PhysicalParameters,
) -> TransferMatrix {
    let zc = params.calc_z0(radius);
    let epsilon = params.alpha_constant() / (radius * wave_number.sqrt());
    let gamma_l = Complex64::new(epsilon, 1.0 + epsilon) * (wave_number * length);
    let cosh_l = gamma_l.cosh();
    let sinh_l = gamma_l.sinh();

    TransferMatrix::new(cosh_l, sinh_l * zc, sinh_l / zc, cosh_l)
}

/// Conical tube transfer matrix with viscothermal losses (Lefebvre & Kergomard).
///
/// Handles the cone's expanding wavefront geometry with linearized losses.
/// Falls back to [`calc_cylinder_matrix`] when `source_radius == load_radius`.
pub fn calc_cone_matrix(
    wave_number: f64,
    length: f64,
    source_radius: f64,
    load_radius: f64,
    params: &PhysicalParameters,
) -> TransferMatrix {
    if source_radius == load_radius {
        return calc_cylinder_matrix(wave_number, length, source_radius, params);
    }

    let alpha_0 = params.alpha_constant() / wave_number.sqrt();

    let epsilon = if (load_radius - source_radius).abs() <= 0.00001 * source_radius {
        // Limiting value as radii approach each other
        alpha_0 / load_radius
    } else {
        alpha_0 / (load_radius - source_radius) * (load_radius / source_radius).ln()
    };

    let mean = Complex64::new(1.0 + epsilon, -epsilon);
    let effective_length = length.max(MINIMUM_CONE_LENGTH);
    let k_mean_l = mean * (wave_number * effective_length);

    let radius_diff = load_radius - source_radius;

    // Cotangents of theta_in and theta_out
    let cot_in = Complex64::new(radius_diff / source_radius, 0.0) / k_mean_l;
    let cot_out = Complex64::new(radius_diff / load_radius, 0.0) / k_mean_l;

    let sin_kl = k_mean_l.sin();
    let cos_kl = k_mean_l.cos();

    let ratio_lr = load_radius / source_radius;
    let ratio_sl = source_radius / load_radius;

    let a = cos_kl * ratio_lr - sin_kl * cot_in;
    let b = Complex64::i() * sin_kl * (params.calc_z0(load_radius) * ratio_lr);
    let c = Complex64::i() * (load_radius / (source_radius * params.calc_z0(source_radius)))
        * (sin_kl * (cot_in * cot_out + 1.0) + cos_kl * (cot_out - cot_in));
    let d = cos_kl * ratio_sl + sin_kl * cot_out;

    TransferMatrix::new(a, b, c, d)
}

/// Unflanged open-end radiation impedance (Silva et al. 2008).
///
/// Returns the complex radiation impedance at the open end of a tube
/// with no flange.
pub fn calc_z_load(freq: f64, radius: f64, params: &PhysicalParameters) -> Complex64 {
    let ka = params.calc_wave_number(freq) * radius;
    let ka2 = ka * ka;
    let z0_denom = params.calc_z0(radius) / (1.0 + ka2 * (0.1514 + 0.05221 * ka2));
    Complex64::new(
        ka2 * (0.2499 + 0.05221 * ka2) * z0_denom,
        ka * (0.6133 + 0.0381 * ka2) * z0_denom,
    )
}

/// Flanged open-end radiation impedance (Silva et al. 2008).
pub fn calc_z_flanged(freq: f64, radius: f64, params: &PhysicalParameters) -> Complex64 {
    let ka = params.calc_wave_number(freq) * radius;
    let ka2 = ka * ka;
    let z0_denom = params.calc_z0(radius) / (1.0 + ka2 * (0.358 + 0.1053 * ka2));
    Complex64::new(
        ka2 * (0.5 + 0.1053 * ka2) * z0_denom,
        ka * (0.82159 + 0.059 * ka2) * z0_denom,
    )
}

/// Radiation resistance for a flanged open end (real part of flanged impedance).
///
/// Used by the fipple mouthpiece calculator.
pub fn calc_r_flanged(freq: f64, radius: f64, params: &PhysicalParameters) -> f64 {
    let ka = params.calc_wave_number(freq) * radius;
    let ka2 = ka * ka;
    params.calc_z0(radius) * ka2 * (0.5 + 0.1053 * ka2)
        / (1.0 + ka2 * (0.358 + 0.1053 * ka2))
}
