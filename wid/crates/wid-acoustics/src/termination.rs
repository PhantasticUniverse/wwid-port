//! Open end termination calculators.
//!
//! Supports two termination types:
//! - **ThickFlanged** (NAF): reflection coefficient model with flange geometry
//! - **Unflanged** (Whistle/Flute/Reed): Silva 2008 Padé approximant radiation impedance

use num_complex::Complex64;
use wid_compile::CompiledTermination;
use wid_math::StateVector;
use wid_physics::PhysicalParameters;

use super::tube;

/// Type of open-end termination.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminationType {
    /// Thick-flanged termination (NAF study model).
    ThickFlanged,
    /// Unflanged termination using Silva 2008 radiation impedance
    /// (Whistle, Flute, Reed study models).
    Unflanged,
}

/// State vector at the termination.
///
/// For an open end, computes the impedance using either the thick-flanged
/// reflection model or the unflanged radiation impedance.
/// For a closed end, returns zero flow.
pub fn calc_termination_sv(
    term: &CompiledTermination,
    is_open: bool,
    wave_number: f64,
    params: &PhysicalParameters,
    term_type: TerminationType,
) -> StateVector {
    if !is_open {
        return StateVector::closed_end();
    }

    match term_type {
        TerminationType::ThickFlanged => {
            let z = calc_z_thick_flanged(term, wave_number, params)
                * params.calc_z0(term.bore_diameter / 2.0);
            StateVector::from_impedance(z)
        }
        TerminationType::Unflanged => {
            let freq = wave_number * params.speed_of_sound()
                / (2.0 * std::f64::consts::PI);
            let z = tube::calc_z_load(freq, term.bore_diameter / 2.0, params);
            StateVector::from_impedance(z)
        }
    }
}

/// Normalized impedance from flange geometry and reflection coefficient (thick-flanged model).
fn calc_z_thick_flanged(term: &CompiledTermination, wave_number: f64, _params: &PhysicalParameters) -> Complex64 {
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

#[cfg(test)]
mod tests {
    use super::*;
    use wid_physics::TemperatureType;

    fn test_termination() -> CompiledTermination {
        CompiledTermination {
            bore_diameter: 0.019,
            flange_diameter: 0.038,
            bore_position: 0.3,
        }
    }

    fn test_params() -> PhysicalParameters {
        PhysicalParameters::new(72.0, TemperatureType::F)
    }

    #[test]
    fn unflanged_produces_finite_result() {
        let term = test_termination();
        let params = test_params();
        let wave_number = params.calc_wave_number(440.0);
        let sv = calc_termination_sv(
            &term, true, wave_number, &params,
            TerminationType::Unflanged,
        );
        let z = sv.impedance();
        assert!(z.re.is_finite(), "Re(Z) should be finite: {}", z.re);
        assert!(z.im.is_finite(), "Im(Z) should be finite: {}", z.im);
    }

    #[test]
    fn unflanged_differs_from_thick_flanged() {
        let term = test_termination();
        let params = test_params();
        let wave_number = params.calc_wave_number(440.0);
        let sv_flanged = calc_termination_sv(
            &term, true, wave_number, &params,
            TerminationType::ThickFlanged,
        );
        let sv_unflanged = calc_termination_sv(
            &term, true, wave_number, &params,
            TerminationType::Unflanged,
        );
        let z_f = sv_flanged.impedance();
        let z_u = sv_unflanged.impedance();
        // They should produce different impedances
        assert!(
            (z_f.re - z_u.re).abs() > 1e-10 || (z_f.im - z_u.im).abs() > 1e-10,
            "Flanged and unflanged should differ: flanged={z_f}, unflanged={z_u}"
        );
    }

    #[test]
    fn closed_end_ignores_termination_type() {
        let term = test_termination();
        let params = test_params();
        let wave_number = params.calc_wave_number(440.0);
        let sv = calc_termination_sv(
            &term, false, wave_number, &params,
            TerminationType::Unflanged,
        );
        // Closed end should be the zero-flow state vector regardless of type
        assert_eq!(sv, StateVector::closed_end());
    }
}
