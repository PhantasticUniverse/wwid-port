//! Tonehole transfer matrix (Lefebvre & Scavone 2012).
//!
//! Computes the T-network transfer matrix for an open or closed tonehole,
//! including series impedance Za (anti-symmetric inertance) and shunt
//! admittance Ys (radiation or closed-end compliance).

use num_complex::Complex64;
use wid_compile::CompiledHole;
use wid_math::TransferMatrix;
use wid_physics::PhysicalParameters;

/// Hole size multiplier for NAF instruments.
pub const NAF_HOLE_SIZE_MULT: f64 = 0.9605;

/// Default hole size multiplier.
pub const DEFAULT_HOLE_SIZE_MULT: f64 = 1.0;

/// Finger adjustment values (in metres).
pub const NO_FINGER_ADJ: f64 = 0.0;
pub const DEFAULT_FINGER_ADJ: f64 = 0.010;

/// Compute the tonehole transfer matrix.
///
/// Uses the Lefebvre-Scavone (2012) model with T-network formulation.
/// The `is_open` flag determines whether the hole radiates (open) or
/// acts as a closed stub (closed by finger).
pub fn calc_hole_tm(
    hole: &CompiledHole,
    is_open: bool,
    wave_number: f64,
    params: &PhysicalParameters,
    hole_size_mult: f64,
    finger_adjustment: f64,
) -> TransferMatrix {
    let radius = hole_size_mult * hole.diameter / 2.0;
    let bore_radius = hole.bore_diameter / 2.0;
    let delta = radius / bore_radius;
    let delta2 = delta * delta;

    let z0h = params.calc_z0(radius);

    // Eq. 8: Mouth-end correction length
    let tm = 0.125 * radius * delta * (1.0 + 0.207 * delta * delta2);

    // Total effective length
    let te = hole.height + tm;

    // Eq. 31: Series impedance intrinsic inertance
    let mut ti = radius
        * (0.822
            + delta
                * (-0.095
                    + delta * (-1.566 + delta * (2.138 + delta * (-1.640 + delta * 0.502)))));

    let ta;
    let ys;

    if is_open {
        let kb = wave_number * radius;
        let ka = wave_number * bore_radius;

        // Eq. 33: Open-end acoustic mass correction
        ta = (-0.35 + 0.06 * (2.7 * hole.height / radius).tanh()) * radius * delta2;

        // Eq. 31 × 32: Frequency-dependent inertance correction
        ti *= 1.0
            + (1.0 - 4.56 * delta + 6.55 * delta2)
                * ka
                * (0.17 + ka * (0.92 + ka * (0.16 - 0.29 * ka)));

        // Radiation resistance (normalized), Eq. 3
        let rr = 0.25 * kb * kb;

        // Eq. 11: Radiation length correction
        let tr = radius * (0.822 - 0.47 * (radius / (bore_radius + hole.height)).powf(0.8));

        // Total acoustic mass of tonehole (Eq. 3 and 7)
        let kttotal = wave_number * ti + (wave_number * (te + tr)).tan();
        ys = Complex64::new(1.0, 0.0)
            / (Complex64::i() * kttotal + Complex64::new(rr, 0.0))
            / z0h;
    } else if hole.inner_curvature_radius.is_some() && hole.inner_curvature_radius.unwrap() < 0.0 {
        // Fully plugged: no admittance
        ta = 0.0;
        ys = Complex64::new(0.0, 0.0);
    } else {
        // Closed by finger (no key for NAF)
        // Eq. 34 (revised): Finger-closed acoustic mass correction
        ta = (-0.20 - 0.10 * (2.4 * hole.height / radius).tanh()) * radius * delta2;

        let tf = if finger_adjustment > 0.0 {
            radius * radius / finger_adjustment
        } else {
            0.0
        };

        // Eq. 16: Effective length with finger intrusion
        let tan_kt = (wave_number * (te - tf)).tan();
        ys = Complex64::new(0.0, tan_kt / (z0h * (1.0 - wave_number * ti * tan_kt)));
    }

    // Series impedance (Eq. 4, 6)
    let za = Complex64::i() * (z0h * delta2 * wave_number * ta);

    // T-network transfer matrix (Eq. 2)
    let za_ys = za * ys;
    let a = za_ys / 2.0 + 1.0;
    let b = za * (za_ys / 4.0 + 1.0);
    let c = ys;

    TransferMatrix::new(a, b, c, a)
}
