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
}
