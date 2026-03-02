//! Brent univariate minimizer.
//!
//! Implements Brent's method for finding the minimum of a unimodal function
//! within a bracket [lower, upper]. Combines golden section search with
//! parabolic interpolation for superlinear convergence.
//!
//! Port of Apache Commons Math 3 `BrentOptimizer.doOptimize()`.

/// Golden section ratio: (3 - sqrt(5)) / 2 ≈ 0.381966.
const GOLDEN_SECTION: f64 = 0.381_966_011_250_105_1;

/// Find the minimum of a univariate function on [lower, upper].
///
/// Returns `Some((x_min, f_min))` on success, `None` if max evaluations exceeded
/// without finding a valid point.
///
/// # Arguments
/// * `f` — objective function
/// * `lower` — lower bound of search interval
/// * `upper` — upper bound of search interval
/// * `start` — initial guess (must be in [lower, upper])
/// * `rel_tol` — relative convergence tolerance (e.g. 1e-6)
/// * `abs_tol` — absolute convergence tolerance (e.g. 1e-14)
/// * `max_eval` — maximum function evaluations
pub fn brent_minimize(
    f: &mut dyn FnMut(f64) -> f64,
    lower: f64,
    upper: f64,
    start: f64,
    rel_tol: f64,
    abs_tol: f64,
    max_eval: usize,
) -> Option<(f64, f64)> {
    let mut a = lower;
    let mut b = upper;

    let mut x = start; // best point so far
    let mut v = x;     // second-best point
    let mut w = x;     // third-best point

    let mut d: f64 = 0.0; // current step size
    let mut e: f64 = 0.0; // step size from two iterations ago

    let mut fx = f(x);
    let mut fv = fx;
    let mut fw = fx;
    let mut evals = 1_usize;

    loop {
        let m = 0.5 * (a + b);
        let tol1 = rel_tol * x.abs() + abs_tol;
        let tol2 = 2.0 * tol1;

        // Convergence check: is the interval small enough?
        if (x - m).abs() <= tol2 - 0.5 * (b - a) {
            return Some((x, fx));
        }

        if evals >= max_eval {
            return Some((x, fx));
        }

        let mut p: f64;
        let mut q: f64;
        let mut u: f64;

        if e.abs() > tol1 {
            // Try parabolic interpolation
            let r = (x - w) * (fx - fv);
            q = (x - v) * (fx - fw);
            p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);

            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }

            let old_e = e; // save step from two iterations ago
            e = d;         // e ← previous step (will be "two ago" next iteration)

            // Accept parabolic step if:
            // 1. It's within the bracket
            // 2. It's less than half the step from two iterations ago
            if p > q * (a - x) && p < q * (b - x) && p.abs() < (0.5 * q * old_e).abs() {
                // Parabolic interpolation step
                d = p / q;
                u = x + d;

                // Don't evaluate too close to bracket endpoints
                if (u - a) < tol2 || (b - u) < tol2 {
                    d = if x < m { tol1 } else { -tol1 };
                }
            } else {
                // Golden section step
                e = if x < m { b - x } else { a - x };
                d = GOLDEN_SECTION * e;
            }
        } else {
            // Golden section step
            e = if x < m { b - x } else { a - x };
            d = GOLDEN_SECTION * e;
        }

        // Evaluate at new point (at least tol1 away from x)
        u = if d.abs() >= tol1 {
            x + d
        } else if d > 0.0 {
            x + tol1
        } else {
            x - tol1
        };

        let fu = f(u);
        evals += 1;

        // Update bracket and best/second-best/third-best points
        if fu <= fx {
            if u < x {
                b = x;
            } else {
                a = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                a = u;
            } else {
                b = u;
            }
            if fu <= fw || w == x {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || v == x || v == w {
                v = u;
                fv = fu;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn minimize_quadratic() {
        // f(x) = (x - 3)^2, minimum at x = 3
        let (x, fx) = brent_minimize(
            &mut |x| (x - 3.0) * (x - 3.0),
            0.0,
            10.0,
            5.0,
            1e-6,
            1e-14,
            1000,
        )
        .unwrap();
        assert_abs_diff_eq!(x, 3.0, epsilon = 1e-6);
        assert_abs_diff_eq!(fx, 0.0, epsilon = 1e-10);
    }

    #[test]
    fn minimize_cosine() {
        // f(x) = -cos(x), minimum at x = 0 on [-1, 1]
        let (x, fx) = brent_minimize(
            &mut |x| -x.cos(),
            -1.0,
            1.0,
            0.5,
            1e-6,
            1e-14,
            1000,
        )
        .unwrap();
        assert_abs_diff_eq!(x, 0.0, epsilon = 1e-6);
        assert_abs_diff_eq!(fx, -1.0, epsilon = 1e-10);
    }

    #[test]
    fn minimize_with_matching_tolerances() {
        // Use the same tolerances as WIDesigner: rel=1e-6, abs=1e-14
        let (x, _fx) = brent_minimize(
            &mut |x| (x - 0.2662) * (x - 0.2662),
            0.2,
            1.5,
            0.75,
            1e-6,
            1e-14,
            1000,
        )
        .unwrap();
        assert_abs_diff_eq!(x, 0.2662, epsilon = 1e-6);
    }

    #[test]
    fn minimize_start_at_boundary() {
        let (x, _) = brent_minimize(
            &mut |x| (x - 5.0) * (x - 5.0),
            0.0,
            10.0,
            0.0,
            1e-6,
            1e-14,
            1000,
        )
        .unwrap();
        assert_abs_diff_eq!(x, 5.0, epsilon = 1e-5);
    }
}
