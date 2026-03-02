//! Fipple mouthpiece transfer matrix calculator.
//!
//! Implements the default fipple mouthpiece model used by NAF instruments.
//! The model combines window impedance (elastic + compliance) with radiation
//! resistance and headspace volume effects.
//!
//! Key physical effects modelled:
//! - Headspace volume (bore sections above mouthpiece, doubled for end correction)
//! - Fipple factor scaling by windway height (cube root law)
//! - Window area → equivalent circular diameter
//! - Radiation resistance at bore opening

use std::f64::consts::PI;

use num_complex::Complex64;
use wid_compile::{BoreSection, CompiledMouthpiece, MouthpieceType};
use wid_math::TransferMatrix;
use wid_physics::{PhysicalParameters, SimplePhysicalParameters};

use crate::tube;

/// Default windway height (metres) used when the instrument XML omits it.
pub const DEFAULT_WINDWAY_HEIGHT: f64 = 0.00078740;

/// Adiabatic index for air, used in the fipple mouthpiece model.
///
/// This is a hardcoded constant from the upstream Java code, slightly
/// different from the CIPM-2007 derived gamma (~1.3993 at 72°F).
pub const AIR_GAMMA: f64 = 1.4018297351222222;

/// Compute the fipple mouthpiece transfer matrix.
///
/// The matrix transforms the acoustic state from the bore (just below
/// the mouthpiece) to the free-field radiation at the window opening.
pub fn calc_fipple_mouthpiece_tm(
    mouthpiece: &CompiledMouthpiece,
    wave_number: f64,
    params: &PhysicalParameters,
) -> TransferMatrix {
    let simple_params = SimplePhysicalParameters::from_physical(params);

    let radius = 0.5 * mouthpiece.bore_diameter;
    let z0 = params.calc_z0(radius);
    let omega = wave_number * params.speed_of_sound();

    let k_delta_l = calc_k_delta_l(mouthpiece, &simple_params, omega, z0);
    let freq = omega / (2.0 * PI);
    let r_rad = tube::calc_r_flanged(freq, radius, params);

    let cos_kl = k_delta_l.cos();
    let sin_kl = k_delta_l.sin();

    let a = Complex64::new(cos_kl, r_rad * sin_kl / z0);
    let b = Complex64::i() * (sin_kl * z0) + Complex64::new(r_rad * cos_kl, 0.0);
    let c = Complex64::i() * (sin_kl / z0);
    let d = Complex64::new(cos_kl, 0.0);

    TransferMatrix::new(a, b, c, d)
}

/// k·Δl: phase shift from the window impedance.
///
/// Combines the elastic admittance (J·Y_E) from the window opening
/// with the compliance admittance (J·Y_C) from the headspace volume.
fn calc_k_delta_l(
    mouthpiece: &CompiledMouthpiece,
    simple_params: &SimplePhysicalParameters,
    omega: f64,
    z0: f64,
) -> f64 {
    let jye = calc_jye(mouthpiece, omega);
    let jyc = calc_jyc(mouthpiece, simple_params, omega);
    (1.0 / (z0 * (jye + jyc))).atan()
}

/// J·Y_E: elastic (window) admittance component.
fn calc_jye(mouthpiece: &CompiledMouthpiece, omega: f64) -> f64 {
    let char_length = get_characteristic_length(mouthpiece);
    char_length / (AIR_GAMMA * omega)
}

/// J·Y_C: compliance (headspace) admittance component.
///
/// The headspace volume is doubled to account for the end correction.
fn calc_jyc(
    mouthpiece: &CompiledMouthpiece,
    simple_params: &SimplePhysicalParameters,
    omega: f64,
) -> f64 {
    let c = simple_params.speed_of_sound();
    let v = 2.0 * calc_headspace_volume(mouthpiece);
    -(omega * v) / (AIR_GAMMA * c * c)
}

/// Total headspace volume (doubled).
///
/// The ×2 factor appears twice in the upstream code: once inside
/// `calcHeadspaceVolume()` and the result is multiplied by 2 again
/// in `calcJYC()`. The net effect is a ×4 factor, but by inspection
/// of the Java code, `calcHeadspaceVolume()` returns `volume * 2.0`
/// and `calcJYC` multiplies by `2.0` again. So the total is ×4.
///
/// Wait — re-reading the Java: `calcHeadspaceVolume()` sums section
/// volumes and multiplies by 2.0. Then `calcJYC()` uses
/// `v = 2.0 * calcHeadspaceVolume()`. So total = raw_volume × 2 × 2 = ×4.
///
/// **UPDATE**: Reading the Java more carefully:
/// ```java
/// protected double calcHeadspaceVolume(Mouthpiece mouthpiece) {
///     volume += getSectionVolume(section);
///     return volume * 2.0;
/// }
/// protected double calcJYC(...) {
///     double v = 2. * calcHeadspaceVolume(mouthpiece);
///     ...
/// }
/// ```
/// So yes, the headspace volume enters as ×4 the geometric volume.
/// However, reading the agent report more carefully, it says calcHeadspaceVolume
/// returns volume * 2.0, and calcJYC uses 2 * calcHeadspaceVolume.
/// Let me match the Java exactly.
fn calc_headspace_volume(mouthpiece: &CompiledMouthpiece) -> f64 {
    let raw_volume: f64 = mouthpiece.headspace.iter().map(section_volume).sum();
    raw_volume * 2.0
}

/// Volume of a conical frustum (bore section).
fn section_volume(section: &BoreSection) -> f64 {
    let r1 = section.left_radius;
    let r2 = section.right_radius;
    PI * section.length * (r1 * r1 + r1 * r2 + r2 * r2) / 3.0
}

/// Characteristic length from the fipple window geometry.
///
/// Converts the rectangular window area to an equivalent circular diameter,
/// scaled by the fipple factor.
fn get_characteristic_length(mouthpiece: &CompiledMouthpiece) -> f64 {
    match &mouthpiece.mouthpiece_type {
        MouthpieceType::Fipple {
            window_length,
            window_width,
            fipple_factor,
            windway_height,
            ..
        } => {
            let scaled_ff = get_scaled_fipple_factor(*fipple_factor, *windway_height);
            let effective_area = window_length * window_width;
            2.0 * (effective_area / PI).sqrt() * scaled_ff
        }
        _ => panic!("calc_fipple_mouthpiece_tm called with non-fipple mouthpiece"),
    }
}

/// Fipple factor with windway height cube-root scaling.
///
/// If windway height is not specified, uses [`DEFAULT_WINDWAY_HEIGHT`].
/// If fipple factor is null, the scaled value is just the ratio.
pub fn get_scaled_fipple_factor(
    fipple_factor: Option<f64>,
    windway_height: Option<f64>,
) -> f64 {
    let wh = windway_height.unwrap_or(DEFAULT_WINDWAY_HEIGHT);
    let ratio = (DEFAULT_WINDWAY_HEIGHT / wh).powf(1.0 / 3.0);

    match fipple_factor {
        Some(ff) => ff * ratio,
        None => ratio,
    }
}
