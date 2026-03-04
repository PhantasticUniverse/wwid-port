//! Simple fipple / embouchure hole mouthpiece calculator for Whistle and Flute instruments.
//!
//! Upstream reference: `SimpleFippleMouthpieceCalculator.java` (Whistle),
//! `FluteMouthpieceCalculator.java` (Flute — same algorithm, different parameter extraction).
//!
//! Unlike the default (NAF) fipple model which uses a transfer matrix with
//! headspace compliance and fipple-factor phase shift, the simple fipple model:
//!
//! 1. Computes empirical window impedance Z_window = R + jX from measurements.
//! 2. Computes headspace state via transmission (closed-end through cone sections).
//! 3. Combines bore state in parallel with headspace state.
//! 4. Multiplies by transfer matrix `[[1, Z_window], [0, 1]]`.
//!
//! # Physical model
//!
//! **Window reactance** (empirical, from real whistle measurements):
//! ```text
//! effSize = sqrt(windowLength * windowWidth)
//! Xw = ρ * f / effSize * (4.30 + 2.87 * windowHeight / effSize)
//! ```
//!
//! **Window resistance** (radiation + short cylindrical tube):
//! ```text
//! Rw = Tube.calcR(f, boreRadius, params) + ρ * 0.0184 * sqrt(f) * windowHeight / effSize³
//! ```
//! where `Tube.calcR` is the flanged radiation resistance (Silva 2008 Padé).
//!
//! **Headspace**: Transmission model — start with closed-end state vector, cascade
//! cone transfer matrices through each headspace bore section. Note: cone radii are
//! passed as (rightRadius, leftRadius) — reversed from bore traversal order.
//!
//! # Key difference from default fipple
//!
//! The default fipple model produces a `TransferMatrix` and is applied last in the
//! impedance pipeline. The simple fipple model operates on the bore `StateVector`
//! directly (it needs the bore SV for the parallel combination with headspace),
//! so it takes the bore SV as input and returns the final SV.

use num_complex::Complex64;
use wid_compile::{CompiledMouthpiece, MouthpieceType};
use wid_math::{StateVector, TransferMatrix};
use wid_physics::PhysicalParameters;

use crate::tube;

/// Calculate simple fipple mouthpiece state vector.
///
/// Takes the bore state vector (termination → components, before mouthpiece)
/// and returns the final state vector after mouthpiece processing.
pub fn calc_simple_fipple_sv(
    mouthpiece: &CompiledMouthpiece,
    bore_sv: StateVector,
    wave_number: f64,
    params: &PhysicalParameters,
) -> StateVector {
    let mut sv = bore_sv;

    // Step 1: Compute headspace state and combine in parallel with bore
    if !mouthpiece.headspace.is_empty() {
        let headspace_sv = calc_headspace_transmission(mouthpiece, wave_number, params);
        sv = bore_sv.parallel(&headspace_sv);
    }

    // Step 2: Apply window impedance as transfer matrix [[1, Z_window], [0, 1]]
    let freq = wave_number * params.speed_of_sound() / (2.0 * std::f64::consts::PI);
    let z_window = calc_z_window(mouthpiece, freq, params);

    let tm = TransferMatrix::new(
        Complex64::new(1.0, 0.0),
        z_window,
        Complex64::new(0.0, 0.0),
        Complex64::new(1.0, 0.0),
    );

    tm.multiply_sv(&sv)
}

/// Calculate the impedance of the whistle/flute window at a specified frequency.
///
/// Upstream: `SimpleFippleMouthpieceCalculator.calcZ()` (Whistle),
/// `FluteMouthpieceCalculator.calcZ()` (Flute).
///
/// Both use the same Xw/Rw formulas; only the parameter extraction differs:
/// - Fipple: `eff_size = sqrt(windowLength * windowWidth)`, window_height with fallbacks
/// - EmbouchureHole: `eff_size = sqrt(min(width, airstreamLength) * length)`, height directly
fn calc_z_window(
    mouthpiece: &CompiledMouthpiece,
    freq: f64,
    params: &PhysicalParameters,
) -> Complex64 {
    let (eff_size, window_height) = match &mouthpiece.mouthpiece_type {
        MouthpieceType::Fipple {
            window_length,
            window_width,
            window_height,
            windway_height,
            ..
        } => {
            let eff = (*window_length * *window_width).sqrt();
            // Window height: explicit windowHeight > windwayHeight > default 0.001m
            let wh = window_height.or(*windway_height).unwrap_or(0.001);
            (eff, wh)
        }
        MouthpieceType::EmbouchureHole {
            length,
            width,
            height,
            airstream_length,
            ..
        } => {
            let hole_width = width.min(*airstream_length);
            let eff = (hole_width * *length).sqrt();
            (eff, *height)
        }
        MouthpieceType::SimpleReed { .. } => {
            unreachable!("calc_z_window called with reed mouthpiece")
        }
    };

    // Reactance: empirical model from whistle measurements
    let rho = params.rho();
    let xw = rho * freq / eff_size * (4.30 + 2.87 * window_height / eff_size);

    // Resistance: radiation resistance + short cylindrical tube
    let radius = 0.5 * mouthpiece.bore_diameter;
    let rw = tube::calc_r_flanged(freq, radius, params)
        + rho * 0.0184 * freq.sqrt() * window_height / (eff_size * eff_size * eff_size);

    Complex64::new(rw, xw)
}

/// Calculate headspace state vector using the transmission model.
///
/// Starts with a closed-end state vector and cascades through each headspace
/// bore section. Note: cone radii are passed as (rightRadius, leftRadius) —
/// reversed from the bore traversal direction, matching the Java
/// `calcHeadspace_transmission` which iterates forward through headspace sections
/// but passes (getRightRadius, getLeftRadius) to `calcConeMatrix`.
fn calc_headspace_transmission(
    mouthpiece: &CompiledMouthpiece,
    wave_number: f64,
    params: &PhysicalParameters,
) -> StateVector {
    let mut sv = StateVector::closed_end();

    for section in &mouthpiece.headspace {
        // Java passes (rightRadius, leftRadius) to calcConeMatrix for headspace.
        // Our tube::calc_cone_matrix takes (source_radius, load_radius).
        // The headspace is traversed from the closed end (top) toward the mouthpiece,
        // so source=right (closed end side) and load=left (mouthpiece side).
        let tm = tube::calc_cone_matrix(
            wave_number,
            section.length,
            section.right_radius,
            section.left_radius,
            params,
        );
        sv = tm.multiply_sv(&sv);
    }

    sv
}
