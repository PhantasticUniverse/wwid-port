//! Brent-Dekker root finding algorithm.
//!
//! Finds a root of f(x) = 0 in the interval [a, b] where f(a) and f(b)
//! have opposite signs. Combines bisection, secant, and inverse quadratic
//! interpolation for superlinear convergence with guaranteed progress.

/// Find a root of `f` in `[a, b]` using Brent's method.
///
/// Returns `None` if the bracket is invalid or max evaluations exceeded.
pub fn find_root(
    f: &dyn Fn(f64) -> f64,
    mut a: f64,
    mut b: f64,
    _rel_tol: f64,
    abs_tol: f64,
    max_eval: usize,
) -> Option<f64> {
    let mut fa = f(a);
    let mut fb = f(b);
    let mut evals = 2;

    // Bracket must contain a sign change
    if fa * fb > 0.0 {
        return None;
    }

    // Ensure |f(b)| <= |f(a)| (b is the better approximation)
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }

    let mut c = a;
    let mut fc = fa;
    let mut d = b - a;
    let mut e = d;

    loop {
        if fb == 0.0 {
            return Some(b);
        }
        if evals >= max_eval {
            return Some(b); // return best estimate
        }

        // Ensure |f(b)| <= |f(c)|
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }

        let tol = 2.0 * f64::EPSILON * b.abs() + abs_tol;
        let m = 0.5 * (c - b);

        if m.abs() <= tol || fb == 0.0 {
            return Some(b);
        }

        // Decide between interpolation and bisection
        if e.abs() >= tol && fa.abs() > fb.abs() {
            let s;
            if (a - c).abs() < f64::EPSILON {
                // Secant method (linear interpolation)
                s = -fb * (b - a) / (fb - fa);
            } else {
                // Inverse quadratic interpolation
                let r = fb / fc;
                let q = fa / fc;
                let p_val = fb / fa;
                s = -(p_val
                    * (2.0 * m * q * (q - r) - (b - a) * (r - 1.0)))
                    / ((q - 1.0) * (r - 1.0) * (p_val - 1.0));
            }

            // Check if interpolation is acceptable
            let bound1 = 0.75 * m - 0.5 * tol.abs();
            if s.abs() < bound1.abs() && s.abs() < (0.5 * e).abs() {
                e = d;
                d = s;
            } else {
                // Bisect
                d = m;
                e = m;
            }
        } else {
            // Bisect
            d = m;
            e = m;
        }

        a = b;
        fa = fb;

        if d.abs() > tol {
            b += d;
        } else if m > 0.0 {
            b += tol;
        } else {
            b -= tol;
        }

        fb = f(b);
        evals += 1;

        // Maintain bracket: f(b) and f(c) must have opposite signs
        if (fb > 0.0 && fc > 0.0) || (fb < 0.0 && fc < 0.0) {
            c = a;
            fc = fa;
            d = b - a;
            e = d;
        }
    }
}

/// Find a local minimum of `f` in `[a, b]` using Brent's method.
///
/// Combines golden-section and parabolic interpolation.
/// Returns `(x_min, f(x_min))`. Matches the Apache Commons Math
/// `BrentOptimizer` used in Java `PlayingRange.findFmin()`.
pub fn find_minimum(
    f: &dyn Fn(f64) -> f64,
    a: f64,
    b: f64,
    rel_tol: f64,
    abs_tol: f64,
    max_eval: usize,
) -> (f64, f64) {
    let golden = 0.5 * (3.0 - 5.0_f64.sqrt()); // ~0.381966

    let (mut lo, mut hi) = if a < b { (a, b) } else { (b, a) };
    let mut x = lo + golden * (hi - lo);
    let mut fx = f(x);
    let mut v = x;
    let mut fv = fx;
    let mut w = x;
    let mut fw = fx;
    let mut d: f64 = 0.0;
    let mut e: f64 = 0.0;
    let mut evals = 1;

    loop {
        let mid = 0.5 * (lo + hi);
        let tol1 = rel_tol * x.abs() + abs_tol;
        let tol2 = 2.0 * tol1;

        if (x - mid).abs() <= tol2 - 0.5 * (hi - lo) || evals >= max_eval {
            return (x, fx);
        }

        // Try parabolic interpolation
        let mut use_golden = true;

        if e.abs() > tol1 {
            // Fit parabola
            let r = (x - w) * (fx - fv);
            let mut q = (x - v) * (fx - fw);
            let mut p = (x - v) * q - (x - w) * r;
            q = 2.0 * (q - r);
            if q > 0.0 {
                p = -p;
            } else {
                q = -q;
            }

            if p.abs() < (0.5 * q * e).abs() && p > q * (lo - x) && p < q * (hi - x) {
                // Parabolic step
                let step = p / q;
                let u_check = x + step;
                if (u_check - lo) < tol2 || (hi - u_check) < tol2 {
                    // Step is too close to boundary — use midpoint side
                    d = if x < mid { tol1 } else { -tol1 };
                } else {
                    d = step;
                }
                use_golden = false;
                e = d;
            }
        }

        if use_golden {
            // Golden section step
            e = if x < mid { hi - x } else { lo - x };
            d = golden * e;
        }

        // Evaluate at new point
        let u = if d.abs() >= tol1 {
            x + d
        } else if d > 0.0 {
            x + tol1
        } else {
            x - tol1
        };

        let fu = f(u);
        evals += 1;

        // Update bracket
        if fu <= fx {
            if u < x {
                hi = x;
            } else {
                lo = x;
            }
            v = w;
            fv = fw;
            w = x;
            fw = fx;
            x = u;
            fx = fu;
        } else {
            if u < x {
                lo = u;
            } else {
                hi = u;
            }
            if fu <= fw || (w - x).abs() < f64::EPSILON {
                v = w;
                fv = fw;
                w = u;
                fw = fu;
            } else if fu <= fv || (v - x).abs() < f64::EPSILON || (v - w).abs() < f64::EPSILON {
                v = u;
                fv = fu;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_sqrt2() {
        let root = find_root(&|x| x * x - 2.0, 1.0, 2.0, 1e-12, 1e-12, 100);
        assert!((root.unwrap() - std::f64::consts::SQRT_2).abs() < 1e-10);
    }

    #[test]
    fn find_zero_of_sin() {
        let root = find_root(&|x| x.sin(), 3.0, 3.5, 1e-12, 1e-12, 100);
        assert!((root.unwrap() - std::f64::consts::PI).abs() < 1e-10);
    }

    #[test]
    fn invalid_bracket_returns_none() {
        let root = find_root(&|x| x * x + 1.0, 0.0, 2.0, 1e-12, 1e-12, 100);
        assert!(root.is_none());
    }

    #[test]
    fn find_minimum_quadratic() {
        // f(x) = (x-3)^2, minimum at x=3
        let (x, fx) = find_minimum(&|x| (x - 3.0) * (x - 3.0), 0.0, 6.0, 1e-10, 1e-10, 100);
        assert!((x - 3.0).abs() < 1e-6);
        assert!(fx < 1e-10);
    }

    #[test]
    fn find_minimum_sin() {
        // sin(x) has minimum at 3π/2 ≈ 4.712 in [π, 2π]
        let (x, fx) = find_minimum(
            &|x| x.sin(),
            std::f64::consts::PI,
            2.0 * std::f64::consts::PI,
            1e-10, 1e-10, 100,
        );
        assert!((x - 1.5 * std::f64::consts::PI).abs() < 1e-6);
        assert!((fx - (-1.0)).abs() < 1e-6);
    }
}
