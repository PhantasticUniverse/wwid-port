//! Bore section transfer matrix calculation.

use wid_compile::BoreSection;
use wid_math::TransferMatrix;
use wid_physics::PhysicalParameters;

use crate::tube;

/// Compute the transfer matrix for a bore section (conical or cylindrical).
pub fn calc_bore_section_tm(
    section: &BoreSection,
    wave_number: f64,
    params: &PhysicalParameters,
) -> TransferMatrix {
    tube::calc_cone_matrix(
        wave_number,
        section.length,
        section.left_radius,
        section.right_radius,
        params,
    )
}
