//! Simple reed mouthpiece transfer matrix calculation.
//!
//! Models the reed mouthpiece as a pressure-node boundary condition with
//! linear frequency-dependent reactance:
//!
//! ```text
//!   X = alpha × 1e-3 × freq + beta
//!
//!   TransferMatrix = | 0+iX   z₀+0i |
//!                    | 1+0i   0+0i  |
//! ```
//!
//! - `alpha` is the reed-specific reactance coefficient (from XML)
//! - `beta` is the mouthpiece beta parameter (default 0.0)
//! - For lip reeds, beta sign is negated
//! - `z₀` is the bore characteristic impedance at the head radius
//!
//! This is far simpler than the fipple or embouchure models — no gain
//! feedback, no window impedance, no radiation resistance.
//!
//! Matches Java `SimpleReedMouthpieceCalculator`.

use num_complex::Complex64;
use wid_compile::{CompiledMouthpiece, MouthpieceType};
use wid_math::TransferMatrix;
use wid_physics::PhysicalParameters;

/// Calculate the reed mouthpiece transfer matrix.
///
/// The reed creates a pressure-node boundary with linear reactance.
/// After applying this TM, the state vector represents the acoustic
/// state at the reed, suitable for impedance calculation.
///
/// # Arguments
/// - `mouthpiece` — compiled mouthpiece (must be `MouthpieceType::SimpleReed`)
/// - `wave_number` — `2π × freq / speed_of_sound`
/// - `params` — physical parameters (for characteristic impedance `z₀`)
pub fn calc_reed_mouthpiece_tm(
    mouthpiece: &CompiledMouthpiece,
    wave_number: f64,
    params: &PhysicalParameters,
) -> TransferMatrix {
    let (alpha, is_lip_reed) = match &mouthpiece.mouthpiece_type {
        MouthpieceType::SimpleReed { alpha, is_lip_reed } => (*alpha, *is_lip_reed),
        _ => return TransferMatrix::identity(),
    };

    // Head radius from compiled bore diameter
    let head_radius = mouthpiece.bore_diameter / 2.0;
    let z0 = params.calc_z0(head_radius);

    // Compute frequency from wave number: freq = k * c / (2π)
    let freq = wave_number * params.speed_of_sound() / (2.0 * std::f64::consts::PI);

    // Beta: for lip reeds, sign is negated (Java: beta = -beta for lip reed)
    let beta = if is_lip_reed { -mouthpiece.beta } else { mouthpiece.beta };

    // Reactance: X = alpha * 1e-3 * freq + beta
    let x = alpha * 1.0e-3 * freq + beta;

    // TransferMatrix: [[0+iX, z0], [1, 0]]
    TransferMatrix::new(
        Complex64::new(0.0, x),     // pp: 0 + iX
        Complex64::new(z0, 0.0),    // pu: z₀
        Complex64::new(1.0, 0.0),   // up: 1
        Complex64::new(0.0, 0.0),   // uu: 0
    )
}
