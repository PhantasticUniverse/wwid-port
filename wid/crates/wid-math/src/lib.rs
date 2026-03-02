//! Core math types for acoustic modelling.
//!
//! Provides [`TransferMatrix`] (2x2 complex matrix) and [`StateVector`]
//! (pressure + volume-flow pair) used throughout the impedance pipeline.

use num_complex::Complex64;

/// 2x2 complex transfer matrix for acoustic transmission.
///
/// Components follow the acoustic convention:
/// - `pp`: pressure → pressure
/// - `pu`: volume-flow → pressure
/// - `up`: pressure → volume-flow
/// - `uu`: volume-flow → volume-flow
///
/// The matrix acts on a [`StateVector`] `[P, U]ᵀ` to propagate the
/// acoustic state through a bore section, tonehole, or other element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransferMatrix {
    pub pp: Complex64,
    pub pu: Complex64,
    pub up: Complex64,
    pub uu: Complex64,
}

impl TransferMatrix {
    /// Create a new transfer matrix from its four complex components.
    pub fn new(pp: Complex64, pu: Complex64, up: Complex64, uu: Complex64) -> Self {
        Self { pp, pu, up, uu }
    }

    /// The 2x2 identity matrix (no-op transformation).
    pub fn identity() -> Self {
        Self {
            pp: Complex64::new(1.0, 0.0),
            pu: Complex64::new(0.0, 0.0),
            up: Complex64::new(0.0, 0.0),
            uu: Complex64::new(1.0, 0.0),
        }
    }

    /// Matrix-matrix product: `self * rhs`.
    pub fn multiply(&self, rhs: &TransferMatrix) -> TransferMatrix {
        TransferMatrix {
            pp: self.pp * rhs.pp + self.pu * rhs.up,
            pu: self.pp * rhs.pu + self.pu * rhs.uu,
            up: self.up * rhs.pp + self.uu * rhs.up,
            uu: self.up * rhs.pu + self.uu * rhs.uu,
        }
    }

    /// Matrix-vector product: `self * sv`.
    pub fn multiply_sv(&self, sv: &StateVector) -> StateVector {
        StateVector {
            p: self.pp * sv.p + self.pu * sv.u,
            u: self.up * sv.p + self.uu * sv.u,
        }
    }

    /// Determinant: `pp*uu - pu*up`.
    pub fn determinant(&self) -> Complex64 {
        self.pp * self.uu - self.pu * self.up
    }
}

/// Acoustic state at a point in the bore: pressure `p` and volume-flow `u`.
///
/// Impedance is defined as `Z = P / U`. The state vector can also be
/// constructed from an impedance value using [`StateVector::from_impedance`],
/// which applies the Dickens (2007) normalization for numerical robustness.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StateVector {
    pub p: Complex64,
    pub u: Complex64,
}

impl StateVector {
    /// Create a state vector from explicit pressure and volume-flow.
    pub fn new(p: Complex64, u: Complex64) -> Self {
        Self { p, u }
    }

    /// State vector representing an open end (pressure = 0).
    pub fn open_end() -> Self {
        Self {
            p: Complex64::new(0.0, 0.0),
            u: Complex64::new(1.0, 0.0),
        }
    }

    /// State vector representing a closed end (volume-flow = 0).
    pub fn closed_end() -> Self {
        Self {
            p: Complex64::new(1.0, 0.0),
            u: Complex64::new(0.0, 0.0),
        }
    }

    /// Construct a state vector from an impedance value.
    ///
    /// Uses the Dickens (2007) normalization: divides both P and U by (1+Z)
    /// so that both components stay between 0 and 1, while their ratio
    /// still equals Z. Handles infinite impedance (closed end) as a special case.
    pub fn from_impedance(z: Complex64) -> Self {
        if z.re == f64::INFINITY {
            return Self {
                p: Complex64::new(1.0, 0.0),
                u: Complex64::new(0.0, 0.0),
            };
        }
        if z.re == f64::NEG_INFINITY {
            return Self {
                p: Complex64::new(-1.0, 0.0),
                u: Complex64::new(0.0, 0.0),
            };
        }
        let z_plus_1 = z + Complex64::new(1.0, 0.0);
        Self {
            p: z / z_plus_1,
            u: Complex64::new(1.0, 0.0) / z_plus_1,
        }
    }

    /// Impedance Z = P / U.
    pub fn impedance(&self) -> Complex64 {
        self.p / self.u
    }

    /// Admittance Y = U / P.
    pub fn admittance(&self) -> Complex64 {
        self.u / self.p
    }

    /// Pressure reflection coefficient relative to characteristic impedance Z0.
    ///
    /// `R = (P - U*Z0) / (P + U*Z0)`
    pub fn reflectance(&self, z0: f64) -> Complex64 {
        let uz0 = self.u * z0;
        (self.p - uz0) / (self.p + uz0)
    }

    /// Series combination: resulting impedance = Z_self + Z_other.
    ///
    /// Operates on the state vector components directly rather than
    /// computing and adding impedances, for numerical stability.
    pub fn series(&self, other: &StateVector) -> StateVector {
        StateVector {
            p: self.p * other.u + other.p * self.u,
            u: self.u * other.u,
        }
    }

    /// Parallel combination: 1/Z_result = 1/Z_self + 1/Z_other.
    ///
    /// Operates on the state vector components directly rather than
    /// computing reciprocal impedances, for numerical stability.
    pub fn parallel(&self, other: &StateVector) -> StateVector {
        StateVector {
            p: self.p * other.p,
            u: self.p * other.u + other.p * self.u,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    const EPSILON: f64 = 1e-12;

    fn c(re: f64, im: f64) -> Complex64 {
        Complex64::new(re, im)
    }

    // ── TransferMatrix tests ────────────────────────────────────────

    #[test]
    fn identity_is_diagonal_ones() {
        let id = TransferMatrix::identity();
        assert_eq!(id.pp, c(1.0, 0.0));
        assert_eq!(id.pu, c(0.0, 0.0));
        assert_eq!(id.up, c(0.0, 0.0));
        assert_eq!(id.uu, c(1.0, 0.0));
    }

    #[test]
    fn multiply_identity_is_noop() {
        let m = TransferMatrix::new(c(1.0, 2.0), c(3.0, 4.0), c(5.0, 6.0), c(7.0, 8.0));
        let id = TransferMatrix::identity();

        let result = m.multiply(&id);
        assert_eq!(result, m);

        let result2 = id.multiply(&m);
        assert_eq!(result2, m);
    }

    #[test]
    fn multiply_two_matrices() {
        // A = [[1+i, 2], [3, 4-i]]
        // B = [[0, 1+2i], [1, 0]]
        let a = TransferMatrix::new(c(1.0, 1.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, -1.0));
        let b = TransferMatrix::new(c(0.0, 0.0), c(1.0, 2.0), c(1.0, 0.0), c(0.0, 0.0));

        let ab = a.multiply(&b);

        // pp = a.pp*b.pp + a.pu*b.up = (1+i)*0 + 2*1 = 2
        assert_abs_diff_eq!(ab.pp.re, 2.0, epsilon = EPSILON);
        assert_abs_diff_eq!(ab.pp.im, 0.0, epsilon = EPSILON);

        // pu = a.pp*b.pu + a.pu*b.uu = (1+i)*(1+2i) + 2*0 = 1+2i+i+2i² = 1+3i-2 = -1+3i
        assert_abs_diff_eq!(ab.pu.re, -1.0, epsilon = EPSILON);
        assert_abs_diff_eq!(ab.pu.im, 3.0, epsilon = EPSILON);

        // up = a.up*b.pp + a.uu*b.up = 3*0 + (4-i)*1 = 4-i
        assert_abs_diff_eq!(ab.up.re, 4.0, epsilon = EPSILON);
        assert_abs_diff_eq!(ab.up.im, -1.0, epsilon = EPSILON);

        // uu = a.up*b.pu + a.uu*b.uu = 3*(1+2i) + (4-i)*0 = 3+6i
        assert_abs_diff_eq!(ab.uu.re, 3.0, epsilon = EPSILON);
        assert_abs_diff_eq!(ab.uu.im, 6.0, epsilon = EPSILON);
    }

    #[test]
    fn multiply_sv_identity_is_noop() {
        let id = TransferMatrix::identity();
        let sv = StateVector::new(c(3.0, 4.0), c(5.0, -1.0));
        let result = id.multiply_sv(&sv);
        assert_eq!(result, sv);
    }

    #[test]
    fn multiply_sv_general() {
        let m = TransferMatrix::new(c(2.0, 0.0), c(0.0, 1.0), c(1.0, 0.0), c(0.0, 0.0));
        let sv = StateVector::new(c(3.0, 0.0), c(0.0, 2.0));

        let result = m.multiply_sv(&sv);

        // p = pp*P + pu*U = 2*3 + i*(2i) = 6 + 2i² = 6 - 2 = 4
        assert_abs_diff_eq!(result.p.re, 4.0, epsilon = EPSILON);
        assert_abs_diff_eq!(result.p.im, 0.0, epsilon = EPSILON);

        // u = up*P + uu*U = 1*3 + 0*(2i) = 3
        assert_abs_diff_eq!(result.u.re, 3.0, epsilon = EPSILON);
        assert_abs_diff_eq!(result.u.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn determinant_identity_is_one() {
        let id = TransferMatrix::identity();
        let det = id.determinant();
        assert_abs_diff_eq!(det.re, 1.0, epsilon = EPSILON);
        assert_abs_diff_eq!(det.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn determinant_general() {
        // [[1+i, 2], [3, 4-i]] → det = (1+i)(4-i) - 2*3 = 4-i+4i-i² - 6 = 4+3i+1 - 6 = -1+3i
        let m = TransferMatrix::new(c(1.0, 1.0), c(2.0, 0.0), c(3.0, 0.0), c(4.0, -1.0));
        let det = m.determinant();
        assert_abs_diff_eq!(det.re, -1.0, epsilon = EPSILON);
        assert_abs_diff_eq!(det.im, 3.0, epsilon = EPSILON);
    }

    // ── StateVector tests ───────────────────────────────────────────

    #[test]
    fn open_end_has_zero_pressure() {
        let sv = StateVector::open_end();
        assert_eq!(sv.p, c(0.0, 0.0));
        assert_eq!(sv.u, c(1.0, 0.0));
    }

    #[test]
    fn closed_end_has_zero_flow() {
        let sv = StateVector::closed_end();
        assert_eq!(sv.p, c(1.0, 0.0));
        assert_eq!(sv.u, c(0.0, 0.0));
    }

    #[test]
    fn impedance_is_p_over_u() {
        let sv = StateVector::new(c(6.0, 3.0), c(2.0, 1.0));
        let z = sv.impedance();
        // (6+3i)/(2+i) = (6+3i)(2-i)/5 = (12-6i+6i-3i²)/5 = (12+3)/5 = 3
        assert_abs_diff_eq!(z.re, 3.0, epsilon = EPSILON);
        assert_abs_diff_eq!(z.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn admittance_is_u_over_p() {
        let sv = StateVector::new(c(6.0, 3.0), c(2.0, 1.0));
        let y = sv.admittance();
        // 1/3
        assert_abs_diff_eq!(y.re, 1.0 / 3.0, epsilon = EPSILON);
        assert_abs_diff_eq!(y.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn from_impedance_real() {
        let z = c(3.0, 0.0);
        let sv = StateVector::from_impedance(z);
        // P/U should equal Z
        let recovered = sv.impedance();
        assert_abs_diff_eq!(recovered.re, 3.0, epsilon = EPSILON);
        assert_abs_diff_eq!(recovered.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn from_impedance_complex() {
        let z = c(2.0, 5.0);
        let sv = StateVector::from_impedance(z);
        let recovered = sv.impedance();
        assert_abs_diff_eq!(recovered.re, 2.0, epsilon = EPSILON);
        assert_abs_diff_eq!(recovered.im, 5.0, epsilon = EPSILON);
    }

    #[test]
    fn from_impedance_positive_infinity_is_closed_end() {
        let sv = StateVector::from_impedance(c(f64::INFINITY, 0.0));
        assert_eq!(sv.p, c(1.0, 0.0));
        assert_eq!(sv.u, c(0.0, 0.0));
    }

    #[test]
    fn from_impedance_negative_infinity() {
        let sv = StateVector::from_impedance(c(f64::NEG_INFINITY, 0.0));
        assert_eq!(sv.p, c(-1.0, 0.0));
        assert_eq!(sv.u, c(0.0, 0.0));
    }

    #[test]
    fn reflectance_matched_is_zero() {
        // If Z = Z0 (matched), reflectance = 0
        let sv = StateVector::new(c(100.0, 0.0), c(1.0, 0.0)); // Z = 100
        let r = sv.reflectance(100.0);
        assert_abs_diff_eq!(r.re, 0.0, epsilon = EPSILON);
        assert_abs_diff_eq!(r.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn reflectance_open_end_is_negative_one() {
        // Open end: P=0, U=1 → R = (0-Z0)/(0+Z0) = -1
        let sv = StateVector::open_end();
        let r = sv.reflectance(100.0);
        assert_abs_diff_eq!(r.re, -1.0, epsilon = EPSILON);
        assert_abs_diff_eq!(r.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn reflectance_closed_end_is_positive_one() {
        // Closed end: P=1, U=0 → R = 1/1 = 1
        let sv = StateVector::closed_end();
        let r = sv.reflectance(100.0);
        assert_abs_diff_eq!(r.re, 1.0, epsilon = EPSILON);
        assert_abs_diff_eq!(r.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn series_adds_impedances() {
        let z1 = c(3.0, 1.0);
        let z2 = c(2.0, -1.0);
        let sv1 = StateVector::from_impedance(z1);
        let sv2 = StateVector::from_impedance(z2);
        let combined = sv1.series(&sv2);
        let z_combined = combined.impedance();
        // Should be z1 + z2 = 5 + 0i
        assert_abs_diff_eq!(z_combined.re, 5.0, epsilon = EPSILON);
        assert_abs_diff_eq!(z_combined.im, 0.0, epsilon = EPSILON);
    }

    #[test]
    fn parallel_adds_admittances() {
        let z1 = c(6.0, 0.0);
        let z2 = c(3.0, 0.0);
        let sv1 = StateVector::from_impedance(z1);
        let sv2 = StateVector::from_impedance(z2);
        let combined = sv1.parallel(&sv2);
        let z_combined = combined.impedance();
        // 1/Z = 1/6 + 1/3 = 1/2, so Z = 2
        assert_abs_diff_eq!(z_combined.re, 2.0, epsilon = EPSILON);
        assert_abs_diff_eq!(z_combined.im, 0.0, epsilon = EPSILON);
    }

    // ── Associativity / composition tests ───────────────────────────

    #[test]
    fn matrix_multiply_is_associative() {
        let a = TransferMatrix::new(c(1.0, 1.0), c(2.0, 0.0), c(0.0, 3.0), c(1.0, -1.0));
        let b = TransferMatrix::new(c(0.0, 1.0), c(1.0, 0.0), c(2.0, 0.0), c(0.0, 2.0));
        let d = TransferMatrix::new(c(3.0, 0.0), c(0.0, -1.0), c(1.0, 1.0), c(2.0, 0.0));

        let ab_d = a.multiply(&b).multiply(&d);
        let a_bd = a.multiply(&b.multiply(&d));

        assert_abs_diff_eq!(ab_d.pp.re, a_bd.pp.re, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.pp.im, a_bd.pp.im, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.pu.re, a_bd.pu.re, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.pu.im, a_bd.pu.im, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.up.re, a_bd.up.re, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.up.im, a_bd.up.im, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.uu.re, a_bd.uu.re, epsilon = EPSILON);
        assert_abs_diff_eq!(ab_d.uu.im, a_bd.uu.im, epsilon = EPSILON);
    }

    #[test]
    fn matrix_sv_multiply_matches_chained() {
        // (A * B) * sv == A * (B * sv)
        let a = TransferMatrix::new(c(1.0, 0.0), c(0.0, 1.0), c(2.0, 0.0), c(1.0, 0.0));
        let b = TransferMatrix::new(c(0.0, 0.0), c(1.0, 0.0), c(1.0, 0.0), c(0.0, 0.0));
        let sv = StateVector::new(c(3.0, 1.0), c(-1.0, 2.0));

        let result1 = a.multiply(&b).multiply_sv(&sv);
        let result2 = a.multiply_sv(&b.multiply_sv(&sv));

        assert_abs_diff_eq!(result1.p.re, result2.p.re, epsilon = EPSILON);
        assert_abs_diff_eq!(result1.p.im, result2.p.im, epsilon = EPSILON);
        assert_abs_diff_eq!(result1.u.re, result2.u.re, epsilon = EPSILON);
        assert_abs_diff_eq!(result1.u.im, result2.u.im, epsilon = EPSILON);
    }
}
