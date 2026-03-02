//! Thick-flanged open end termination calculator.
//!
//! Computes the acoustic state vector at the open end of a tube with
//! a flange of specified diameter, using a reflection coefficient model.

use num_complex::Complex64;
use wid_compile::CompiledTermination;
use wid_math::StateVector;
use wid_physics::PhysicalParameters;

/// State vector at the termination.
///
/// For an open end, computes the reflection-based impedance accounting
/// for the flange geometry. For a closed end, returns zero flow.
pub fn calc_termination_sv(
    term: &CompiledTermination,
    is_open: bool,
    wave_number: f64,
    params: &PhysicalParameters,
) -> StateVector {
    if !is_open {
        return StateVector::closed_end();
    }

    let z = calc_z(term, wave_number, params) * params.calc_z0(term.bore_diameter / 2.0);
    StateVector::from_impedance(z)
}

/// Normalized impedance from flange geometry and reflection coefficient.
fn calc_z(term: &CompiledTermination, wave_number: f64, _params: &PhysicalParameters) -> Complex64 {
    let a = term.bore_diameter / 2.0;
    let b = term.flange_diameter / 2.0;
    let a_b = a / b;
    let ka = wave_number * a;

    let delta_inf = 0.8216;
    let delta_0 = 0.6133;

    // Interpolated end correction between flanged and unflanged limits
    let delta_circ = delta_inf + a_b * (delta_0 - delta_inf)
        + 0.057 * a_b * (1.0 - a_b.powf(5.0));

    // Magnitude of reflection coefficient
    let r0 = (1.0 + 0.2 * ka - 0.084 * ka * ka)
        / (1.0 + 0.2 * ka + (0.5 - 0.084) * ka * ka);

    // Complex reflection coefficient with phase from end correction
    let r = (Complex64::i() * (-2.0 * delta_circ * ka)).exp() * (-r0);

    // Impedance from reflection coefficient: Z = (1+R)/(1-R)
    (r + 1.0) / (Complex64::new(1.0, 0.0) - r)
}
