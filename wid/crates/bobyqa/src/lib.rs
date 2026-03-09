// Faithful port of Powell's Fortran BOBYQA — index-based loops match the original.
#![allow(
    clippy::needless_range_loop,
    clippy::too_many_arguments,
    clippy::manual_memcpy,
    clippy::assign_op_pattern,
    clippy::manual_saturating_arithmetic,
    clippy::implicit_saturating_sub
)]
//! Pure Rust implementation of Powell's BOBYQA algorithm.
//!
//! **BOBYQA** (Bound Optimization BY Quadratic Approximation) is a
//! derivative-free optimizer for minimizing a function of several variables
//! subject to simple bound constraints on the variables. It was developed
//! by M.J.D. Powell and described in:
//!
//! > M.J.D. Powell, "The BOBYQA algorithm for bound constrained optimization
//! > without derivatives," Cambridge NA Report NA2009/06, University of
//! > Cambridge, 2009.
//!
//! The algorithm builds and maintains a quadratic interpolation model of the
//! objective function, using a set of interpolation points that moves with the
//! trust region. At each iteration it solves a trust-region subproblem on the
//! model to produce a trial step, evaluates the true objective, and updates the
//! model accordingly.
//!
//! # When to use BOBYQA
//!
//! BOBYQA is appropriate when:
//! - The objective function has **no available derivatives** (black-box).
//! - Variables are subject to **simple lower/upper bound constraints**.
//! - The function is **smooth** (at least approximately).
//! - The number of variables is **moderate** (up to ~50–100).
//!
//! It is *not* a global optimizer — it finds a local minimum from the given
//! starting point.
//!
//! # Convergence sensitivity
//!
//! BOBYQA's optimization trajectory is **chaotically sensitive** to sub-ULP
//! evaluation differences. The quadratic model's Hessian is estimated via
//! finite differences at spacing `rhobeg`. When `rhobeg` is small, even
//! 1e-9 relative error in function values can produce ~1e-6 relative error
//! in Hessian estimates (~1000× amplification), steering the first
//! trust-region step differently and causing divergent trajectories.
//!
//! In practice this means that two implementations producing function values
//! matching to 10+ significant digits may converge to *different local
//! minima* on multimodal landscapes. This is inherent to derivative-free
//! quadratic modelling, not a bug. For problems requiring global
//! convergence, combine BOBYQA with multi-start or DIRECT-C global search.
//!
//! # Implementation notes
//!
//! This is a faithful port of the `BOBYQAOptimizer` class from
//! [Apache Commons Math 3.6.1](https://commons.apache.org/proper/commons-math/),
//! which was itself a translation of Powell's original Fortran code. The port
//! preserves the algorithm's numerical behavior: on many test functions it
//! produces bit-identical results to the Java implementation, and on all tested
//! functions it converges to the same optima within comparable evaluation counts.
//!
//! The Fortran code used `GOTO` statements for control flow; the Java version
//! translated these into a `switch`-based state machine. This Rust port uses
//! `loop { match state { … } }` with the same state labels (20, 60, 90, …).
//!
//! # Evaluation budget
//!
//! The `max_eval` parameter is a hard budget: the algorithm checks after
//! each function evaluation and stops immediately when the limit is reached.
//! The actual count is reported in [`BobyqaResult::evaluations`].
//!
//! # Known limitations
//!
//! - **No rescue procedure**: The Fortran original has a `RESCUE` subroutine
//!   that resets the interpolation set when it becomes ill-conditioned.
//!   Neither the Apache Commons Math Java version nor this Rust port
//!   implements rescue. On ill-conditioned problems, the algorithm may
//!   terminate early instead of recovering.
//! - **N >= 2 required**: BOBYQA requires at least 2 variables. For
//!   1-dimensional problems, use Brent's method or another 1D optimizer.
//! - **Local optimizer only**: BOBYQA finds a local minimum from the given
//!   starting point. For global optimization, combine with multi-start or
//!   DIRECT-C.
//!
//! # Zero dependencies
//!
//! This crate has no runtime dependencies. All linear algebra operations are
//! performed inline on `Vec<f64>`. This makes it suitable for `no_std`
//! environments (with `alloc`) and WebAssembly targets.
//!
//! # Example
//!
//! ```
//! use bobyqa::bobyqa_minimize;
//!
//! // Minimize the Rosenbrock function in 2D, bounded to [-5, 5]
//! let result = bobyqa_minimize(
//!     &mut |x| {
//!         let t = x[0] * x[0] - x[1];
//!         100.0 * t * t + (x[0] - 1.0) * (x[0] - 1.0)
//!     },
//!     &[-1.0, -1.0],             // initial point
//!     &[-5.0, -5.0],             // lower bounds
//!     &[5.0, 5.0],               // upper bounds
//!     5,                          // interpolation points (2n+1)
//!     1.0,                        // initial trust region radius
//!     1e-8,                       // stopping trust region radius
//!     5000,                       // max evaluations
//! )
//! .expect("optimization should converge");
//!
//! assert!((result.point[0] - 1.0).abs() < 1e-6);
//! assert!((result.point[1] - 1.0).abs() < 1e-6);
//! assert!(result.value < 1e-12);
//! ```
//!
//! # References
//!
//! - Powell, M.J.D. (2009). "The BOBYQA algorithm for bound constrained
//!   optimization without derivatives." Cambridge NA Report NA2009/06.
//! - Apache Commons Math 3.6.1, `org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer`
//! - Powell, M.J.D. (2006). "The NEWUOA software for unconstrained optimization
//!   without derivatives." (BOBYQA extends NEWUOA with bound handling.)

// These initializations are intentional: the Fortran/Java state machine guarantees
// that variables are assigned before use, but Rust's borrow checker cannot prove
// this across `match` arms. The default values are never read.
#![allow(unused_assignments)]

/// Progress information passed to the callback in [`bobyqa_minimize_with_callback`].
#[derive(Debug, Clone)]
pub struct BobyqaProgress {
    /// Number of objective function evaluations completed so far.
    pub evaluations: usize,
    /// Best (lowest) objective function value found so far.
    pub best_value: f64,
}

/// Result returned by [`bobyqa_minimize`] on successful convergence.
#[derive(Debug, Clone)]
pub struct BobyqaResult {
    /// The point (variable values) at the minimum found.
    pub point: Vec<f64>,
    /// The objective function value at `point`.
    pub value: f64,
    /// The total number of objective function evaluations used.
    pub evaluations: usize,
}

/// Find the minimum of a bounded multivariate function without derivatives.
///
/// This is the main entry point for BOBYQA optimization. It minimizes an
/// objective function `f` starting from `initial_point`, subject to the
/// box constraints `lower_bounds[i] <= x[i] <= upper_bounds[i]`.
///
/// # Arguments
///
/// * `f` — The objective function to minimize. Takes a slice of `n` variables
///   and returns a scalar value. Will be called at most `max_eval` times.
///
/// * `initial_point` — Starting point for the optimization. Must satisfy
///   the bound constraints. Length `n` defines the problem dimension (n >= 2).
///
/// * `lower_bounds` — Lower bound for each variable (length `n`).
///
/// * `upper_bounds` — Upper bound for each variable (length `n`).
///   Must satisfy `upper_bounds[i] - lower_bounds[i] >= 2 * stopping_trust`
///   for all `i`.
///
/// * `n_interp` — Number of interpolation points for the quadratic model.
///   Must satisfy `n + 2 <= n_interp <= (n+1)(n+2)/2`. The recommended
///   value is `2 * n + 1`. Fewer points means faster iterations but a less
///   accurate model; more points improves the model but costs more per step.
///
/// * `initial_trust` — Initial trust region radius. Controls the size of
///   the first steps. Should be roughly the scale of the expected distance
///   from the starting point to the optimum. Automatically reduced if the
///   bounds are too narrow.
///
/// * `stopping_trust` — Stopping trust region radius. The algorithm
///   terminates when the trust region radius falls below this value.
///   Typical values: `1e-6` to `1e-10`.
///
/// * `max_eval` — Maximum number of function evaluations before stopping.
///
/// # Returns
///
/// `Some(BobyqaResult)` with the best point found, or `None` if `n < 2`.
///
/// # Panics
///
/// May panic if bounds are inconsistent (lower > upper) or if `n_interp`
/// is outside the valid range.
pub fn bobyqa_minimize(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial_point: &[f64],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    n_interp: usize,
    initial_trust: f64,
    stopping_trust: f64,
    max_eval: usize,
) -> Option<BobyqaResult> {
    let n = initial_point.len();
    if n < 2 {
        return None;
    }

    let mut state = BobyqaState::new(
        n,
        n_interp,
        initial_point,
        lower_bounds,
        upper_bounds,
        initial_trust,
        stopping_trust,
        max_eval,
    );

    let value = state.bobyqa(f, lower_bounds, upper_bounds);
    Some(BobyqaResult {
        point: state.current_best.clone(),
        value,
        evaluations: state.evals,
    })
}

/// Like [`bobyqa_minimize`], but with a progress callback for monitoring and cancellation.
///
/// The `on_progress` callback is called after every objective function evaluation.
/// It receives a [`BobyqaProgress`] struct and should return `true` to continue
/// or `false` to cancel the optimization (returns the best result found so far).
pub fn bobyqa_minimize_with_callback(
    f: &mut dyn FnMut(&[f64]) -> f64,
    initial_point: &[f64],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    n_interp: usize,
    initial_trust: f64,
    stopping_trust: f64,
    max_eval: usize,
    on_progress: &mut dyn FnMut(BobyqaProgress) -> bool,
) -> Option<BobyqaResult> {
    let n = initial_point.len();
    if n < 2 {
        return None;
    }

    // Track the best value seen so far for progress reporting.
    // Use Cell for interior mutability so the closure can share these
    // with the post-optimization code without borrow conflicts.
    use std::cell::Cell;
    let best_value = Cell::new(f64::MAX);
    let cancelled = Cell::new(false);
    let eval_count = Cell::new(0_usize);

    // Wrap the objective to call progress callback after each evaluation
    let mut wrapped_f = |x: &[f64]| -> f64 {
        if cancelled.get() {
            return best_value.get();
        }
        let val = f(x);
        eval_count.set(eval_count.get() + 1);
        if val < best_value.get() {
            best_value.set(val);
        }
        let should_continue = on_progress(BobyqaProgress {
            evaluations: eval_count.get(),
            best_value: best_value.get(),
        });
        if !should_continue {
            cancelled.set(true);
        }
        val
    };

    let mut state = BobyqaState::new(
        n,
        n_interp,
        initial_point,
        lower_bounds,
        upper_bounds,
        initial_trust,
        stopping_trust,
        max_eval,
    );

    let value = state.bobyqa(&mut wrapped_f, lower_bounds, upper_bounds);
    Some(BobyqaResult {
        point: state.current_best.clone(),
        value,
        evaluations: state.evals,
    })
}

// ── Internal state ──────────────────────────────────────────────────

/// All mutable algorithm state for a single BOBYQA run.
///
/// Maps directly to the Fortran/Java internal variables:
/// - `xpt` = interpolation points (relative to `origin_shift`)
/// - `b_matrix` + `z_matrix` = factored inverse interpolation matrix (H)
/// - `gopt` = model gradient at trust region center
/// - `hq` + `pq` = model Hessian (explicit + implicit parts)
/// - `sl`, `su` = bounds relative to `origin_shift`
///
/// # State machine
///
/// The `bobyqb` method is the main loop, using `match state { 20 | 60 | … }`
/// to replicate the Fortran GOTO labels. Each label corresponds to a phase:
/// - **20**: Start of new iteration (choose geometry step vs trust region step)
/// - **60**: Evaluate trial point
/// - **90**: Successful step — update model
/// - **210/230**: Alternative iteration (geometry improvement)
/// - **360**: Reduce trust region radius
/// - **650/680/720**: Model update and return to iteration start
struct BobyqaState {
    n: usize,
    npt: usize,
    current_best: Vec<f64>,
    bound_difference: Vec<f64>,
    trust_region_center_index: usize,
    // H matrix components
    b_matrix: Vec<Vec<f64>>,    // (npt+n) x n
    z_matrix: Vec<Vec<f64>>,    // npt x (npt-n-1)
    // Interpolation points relative to origin_shift
    xpt: Vec<Vec<f64>>,         // npt x n
    origin_shift: Vec<f64>,     // n  (xbase)
    f_at_interp: Vec<f64>,      // npt (fval)
    xopt: Vec<f64>,             // n  (trust region center offset)
    gopt: Vec<f64>,             // n  (gradient at trust region center)
    // Bounds relative to origin_shift
    sl: Vec<f64>,               // n  (lower_difference)
    su: Vec<f64>,               // n  (upper_difference)
    // Second derivative model
    pq: Vec<f64>,               // npt (implicit second derivatives)
    hq: Vec<f64>,               // n*(n+1)/2 (explicit second derivatives)
    // Working points
    xnew: Vec<f64>,             // n
    xalt: Vec<f64>,             // n
    d: Vec<f64>,                // n (trial step)
    vlag: Vec<f64>,             // npt+n (lagrange values)
    // Counters
    evals: usize,
    max_eval: usize,
    initial_trust_region_radius: f64,
    stopping_trust_region_radius: f64,
}

impl BobyqaState {
    fn new(
        n: usize,
        npt: usize,
        initial_point: &[f64],
        lower_bounds: &[f64],
        upper_bounds: &[f64],
        mut initial_trust: f64,
        stopping_trust: f64,
        max_eval: usize,
    ) -> Self {
        // Validate
        assert!(npt >= n + 2 && npt <= (n + 1) * (n + 2) / 2);

        // Initialize bound differences
        let mut bound_difference = vec![0.0; n];
        let mut min_diff = f64::INFINITY;
        for i in 0..n {
            bound_difference[i] = upper_bounds[i] - lower_bounds[i];
            min_diff = min_diff.min(bound_difference[i]);
        }

        let required_min_diff = 2.0 * initial_trust;
        if min_diff < required_min_diff {
            initial_trust = min_diff / 3.0;
        }

        let ndim = npt + n;
        let nptm = npt - n - 1;
        let nh = n * (n + 1) / 2;

        BobyqaState {
            n,
            npt,
            current_best: initial_point.to_vec(),
            bound_difference,
            trust_region_center_index: 0,
            b_matrix: vec![vec![0.0; n]; ndim],
            z_matrix: vec![vec![0.0; nptm]; npt],
            xpt: vec![vec![0.0; n]; npt],
            origin_shift: vec![0.0; n],
            f_at_interp: vec![0.0; npt],
            xopt: vec![0.0; n],
            gopt: vec![0.0; n],
            sl: vec![0.0; n],
            su: vec![0.0; n],
            pq: vec![0.0; npt],
            hq: vec![0.0; nh],
            xnew: vec![0.0; n],
            xalt: vec![0.0; n],
            d: vec![0.0; n],
            vlag: vec![0.0; ndim],
            evals: 0,
            max_eval,
            initial_trust_region_radius: initial_trust,
            stopping_trust_region_radius: stopping_trust,
        }
    }

    fn compute_objective(&mut self, f: &mut dyn FnMut(&[f64]) -> f64, point: &[f64]) -> f64 {
        self.evals += 1;
        f(point)
    }

    // ── bobyqa: outer wrapper ───────────────────────────────────────
    //
    // Sets up the initial interpolation (via `prelim`) then delegates
    // to `bobyqb` for the main optimization loop.

    fn bobyqa(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        lower_bound: &[f64],
        upper_bound: &[f64],
    ) -> f64 {
        let n = self.n;
        let rho = self.initial_trust_region_radius;

        for j in 0..n {
            self.sl[j] = lower_bound[j] - self.current_best[j];
            self.su[j] = upper_bound[j] - self.current_best[j];

            if self.sl[j] >= -rho {
                if self.sl[j] >= 0.0 {
                    self.current_best[j] = lower_bound[j];
                    self.sl[j] = 0.0;
                    self.su[j] = self.bound_difference[j];
                } else {
                    self.current_best[j] = lower_bound[j] + rho;
                    self.sl[j] = -rho;
                    self.su[j] = (upper_bound[j] - self.current_best[j]).max(rho);
                }
            } else if self.su[j] <= rho {
                if self.su[j] <= 0.0 {
                    self.current_best[j] = upper_bound[j];
                    self.sl[j] = -self.bound_difference[j];
                    self.su[j] = 0.0;
                } else {
                    self.current_best[j] = upper_bound[j] - rho;
                    self.sl[j] = (lower_bound[j] - self.current_best[j]).min(-rho);
                    self.su[j] = rho;
                }
            }
        }

        self.bobyqb(f, lower_bound, upper_bound)
    }

    // ── bobyqb: main optimization loop ──────────────────────────────
    //
    // This is the core BOBYQA iteration. Uses a state machine (`match state`)
    // mapping to the Fortran GOTO labels. Each iteration either:
    // (a) solves the trust-region subproblem (`trsbox`) for a trial step, or
    // (b) performs a geometry-improvement step (`altmov`) to maintain model quality.
    // After evaluating the trial point, the model is updated via `update`.

    fn bobyqb(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        lower_bound: &[f64],
        upper_bound: &[f64],
    ) -> f64 {
        let n = self.n;
        let npt = self.npt;
        let np = n + 1;
        let nptm = npt - np;

        let mut work1 = vec![0.0; n];
        let mut work2 = vec![0.0; npt];
        let mut work3 = vec![0.0; npt];

        let mut cauchy: f64 = 0.0;
        let mut alpha: f64 = 0.0;
        let mut dsq: f64 = 0.0;
        let mut crvmin: f64 = 0.0;

        self.trust_region_center_index = 0;
        self.prelim(f, lower_bound, upper_bound);

        let mut xoptsq = 0.0;
        for i in 0..n {
            self.xopt[i] = self.xpt[self.trust_region_center_index][i];
            xoptsq += self.xopt[i] * self.xopt[i];
        }
        let mut fsave = self.f_at_interp[0];

        let mut ntrits: i32 = 0;
        let mut itest: i32 = 0;
        let mut knew: usize = 0;
        let mut nfsav = self.evals;
        let mut rho = self.initial_trust_region_radius;
        let mut delta = rho;
        let mut diffa = 0.0_f64;
        let mut diffb = 0.0_f64;
        let mut diffc = 0.0_f64;
        let mut f_val: f64 = 0.0;
        let mut beta: f64 = 0.0;
        let mut adelt: f64 = 0.0;
        let mut denom: f64 = 0.0;
        let mut ratio: f64 = 0.0;
        let mut dnorm: f64 = 0.0;
        let mut scaden: f64 = 0.0;
        let mut biglsq: f64 = 0.0;
        let mut distsq: f64 = 0.0;
        let mut vquad: f64 = 0.0;

        // State machine (matching Java/Fortran GOTO labels)
        let mut state: i32 = 20;
        loop {
            match state {
                20 => {
                    // Update gopt if trust region center changed
                    if self.trust_region_center_index != 0 {
                        let mut ih = 0;
                        for j in 0..n {
                            for i in 0..=j {
                                if i < j {
                                    self.gopt[j] += self.hq[ih] * self.xopt[i];
                                }
                                self.gopt[i] += self.hq[ih] * self.xopt[j];
                                ih += 1;
                            }
                        }
                        if self.evals > npt {
                            for k in 0..npt {
                                let mut temp = 0.0;
                                for j in 0..n {
                                    temp += self.xpt[k][j] * self.xopt[j];
                                }
                                temp *= self.pq[k];
                                for i in 0..n {
                                    self.gopt[i] += temp * self.xpt[k][i];
                                }
                            }
                        }
                    }
                    state = 60;
                }
                60 => {
                    // Trust region step via trsbox
                    let dsq_crvmin = self.trsbox(delta);
                    dsq = dsq_crvmin.0;
                    crvmin = dsq_crvmin.1;

                    dnorm = delta.min(dsq.sqrt());
                    if dnorm < 0.5 * rho {
                        ntrits = -1;
                        distsq = (10.0 * rho) * (10.0 * rho);
                        if self.evals <= nfsav + 2 {
                            state = 650;
                            continue;
                        }
                        let errbig = diffa.max(diffb).max(diffc);
                        let frhosq = rho * 0.125 * rho;
                        if crvmin > 0.0 && errbig > frhosq * crvmin {
                            state = 650;
                            continue;
                        }
                        let bdtol = errbig / rho;
                        let mut goto_650 = false;
                        for j in 0..n {
                            let mut bdtest = bdtol;
                            if self.xnew[j] == self.sl[j] {
                                bdtest = work1[j];
                            }
                            if self.xnew[j] == self.su[j] {
                                bdtest = -work1[j];
                            }
                            if bdtest < bdtol {
                                let mut curv = self.hq[(j + j * j) / 2];
                                for k in 0..npt {
                                    curv += self.pq[k] * self.xpt[k][j] * self.xpt[k][j];
                                }
                                bdtest += 0.5 * curv * rho;
                                if bdtest < bdtol {
                                    goto_650 = true;
                                    break;
                                }
                            }
                        }
                        if goto_650 {
                            state = 650;
                            continue;
                        }
                        state = 680;
                        continue;
                    }
                    ntrits += 1;

                    // Shift origin if xopt is far from xbase
                    state = 90;
                }
                90 => {
                    if dsq <= xoptsq * 1e-3 {
                        let fracsq = xoptsq * 0.25;
                        let mut sumpq = 0.0;
                        for k in 0..npt {
                            sumpq += self.pq[k];
                            let mut sum = -0.5 * xoptsq;
                            for i in 0..n {
                                sum += self.xpt[k][i] * self.xopt[i];
                            }
                            work2[k] = sum;
                            let temp = fracsq - 0.5 * sum;
                            for i in 0..n {
                                work1[i] = self.b_matrix[k][i];
                                self.vlag[i] = sum * self.xpt[k][i] + temp * self.xopt[i];
                                let ip = npt + i;
                                for j in 0..=i {
                                    self.b_matrix[ip][j] +=
                                        work1[i] * self.vlag[j] + self.vlag[i] * work1[j];
                                }
                            }
                        }

                        for m in 0..nptm {
                            let mut sumz = 0.0;
                            let mut sumw = 0.0;
                            for k in 0..npt {
                                sumz += self.z_matrix[k][m];
                                self.vlag[k] = work2[k] * self.z_matrix[k][m];
                                sumw += self.vlag[k];
                            }
                            for j in 0..n {
                                let mut sum = (fracsq * sumz - 0.5 * sumw) * self.xopt[j];
                                for k in 0..npt {
                                    sum += self.vlag[k] * self.xpt[k][j];
                                }
                                work1[j] = sum;
                                for k in 0..npt {
                                    self.b_matrix[k][j] += sum * self.z_matrix[k][m];
                                }
                            }
                            for i in 0..n {
                                let ip = i + npt;
                                let temp = work1[i];
                                for j in 0..=i {
                                    self.b_matrix[ip][j] += temp * work1[j];
                                }
                            }
                        }

                        let mut ih = 0;
                        for j in 0..n {
                            work1[j] = -0.5 * sumpq * self.xopt[j];
                            for k in 0..npt {
                                work1[j] += self.pq[k] * self.xpt[k][j];
                                self.xpt[k][j] -= self.xopt[j];
                            }
                            for i in 0..=j {
                                self.hq[ih] +=
                                    work1[i] * self.xopt[j] + self.xopt[i] * work1[j];
                                self.b_matrix[npt + i][j] = self.b_matrix[npt + j][i];
                                ih += 1;
                            }
                        }
                        for i in 0..n {
                            self.origin_shift[i] += self.xopt[i];
                            self.xnew[i] -= self.xopt[i];
                            self.sl[i] -= self.xopt[i];
                            self.su[i] -= self.xopt[i];
                            self.xopt[i] = 0.0;
                        }
                        xoptsq = 0.0;
                    }
                    if ntrits == 0 {
                        state = 210;
                        continue;
                    }
                    state = 230;
                }
                210 => {
                    // Pick alternative new position (altmov)
                    let alpha_cauchy = self.altmov(knew, adelt);
                    alpha = alpha_cauchy.0;
                    cauchy = alpha_cauchy.1;

                    for i in 0..n {
                        self.d[i] = self.xnew[i] - self.xopt[i];
                    }
                    state = 230;
                }
                230 => {
                    // Calculate vlag and beta
                    for k in 0..npt {
                        let mut suma = 0.0;
                        let mut sumb = 0.0;
                        let mut sum = 0.0;
                        for j in 0..n {
                            suma += self.xpt[k][j] * self.d[j];
                            sumb += self.xpt[k][j] * self.xopt[j];
                            sum += self.b_matrix[k][j] * self.d[j];
                        }
                        work3[k] = suma * (0.5 * suma + sumb);
                        self.vlag[k] = sum;
                        work2[k] = suma;
                    }
                    beta = 0.0;
                    for m in 0..nptm {
                        let mut sum = 0.0;
                        for k in 0..npt {
                            sum += self.z_matrix[k][m] * work3[k];
                        }
                        beta -= sum * sum;
                        for k in 0..npt {
                            self.vlag[k] += sum * self.z_matrix[k][m];
                        }
                    }
                    dsq = 0.0;
                    let mut bsum = 0.0;
                    let mut dx = 0.0;
                    for j in 0..n {
                        dsq += self.d[j] * self.d[j];
                        let mut sum = 0.0;
                        for k in 0..npt {
                            sum += work3[k] * self.b_matrix[k][j];
                        }
                        bsum += sum * self.d[j];
                        let jp = npt + j;
                        for i in 0..n {
                            sum += self.b_matrix[jp][i] * self.d[i];
                        }
                        self.vlag[jp] = sum;
                        bsum += sum * self.d[j];
                        dx += self.d[j] * self.xopt[j];
                    }

                    beta = dx * dx + dsq * (xoptsq + dx + dx + 0.5 * dsq) + beta - bsum;

                    self.vlag[self.trust_region_center_index] += 1.0;

                    if ntrits == 0 {
                        let d1 = self.vlag[knew];
                        denom = d1 * d1 + alpha * beta;
                        if denom < cauchy && cauchy > 0.0 {
                            for i in 0..n {
                                self.xnew[i] = self.xalt[i];
                                self.d[i] = self.xnew[i] - self.xopt[i];
                            }
                            state = 230;
                            cauchy = 0.0; // prevent infinite loop
                            continue;
                        }
                    } else {
                        let delsq = delta * delta;
                        scaden = 0.0;
                        biglsq = 0.0;
                        knew = 0;
                        for k in 0..npt {
                            if k == self.trust_region_center_index {
                                continue;
                            }
                            let mut hdiag = 0.0;
                            for m in 0..nptm {
                                hdiag += self.z_matrix[k][m] * self.z_matrix[k][m];
                            }
                            let den = beta * hdiag + self.vlag[k] * self.vlag[k];
                            distsq = 0.0;
                            for j in 0..n {
                                let d3 = self.xpt[k][j] - self.xopt[j];
                                distsq += d3 * d3;
                            }
                            let d4 = distsq / delsq;
                            let temp = (1.0_f64).max(d4 * d4);
                            if temp * den > scaden {
                                scaden = temp * den;
                                knew = k;
                                denom = den;
                            }
                            biglsq = biglsq.max(temp * self.vlag[k] * self.vlag[k]);
                        }
                    }

                    // Evaluate objective function
                    state = 360;
                }
                360 => {
                    // Compute point in original space
                    for i in 0..n {
                        let d3 = lower_bound[i];
                        let d4 = self.origin_shift[i] + self.xnew[i];
                        let d1 = d3.max(d4);
                        let d2 = upper_bound[i];
                        self.current_best[i] = d1.min(d2);
                        if self.xnew[i] == self.sl[i] {
                            self.current_best[i] = lower_bound[i];
                        }
                        if self.xnew[i] == self.su[i] {
                            self.current_best[i] = upper_bound[i];
                        }
                    }

                    if self.evals >= self.max_eval {
                        // Return best result found so far
                        state = 720;
                        continue;
                    }

                    f_val = self.compute_objective(f, &self.current_best.clone());

                    if ntrits == -1 {
                        fsave = f_val;
                        state = 720;
                        continue;
                    }

                    // Quadratic model prediction
                    let fopt = self.f_at_interp[self.trust_region_center_index];
                    vquad = 0.0;
                    let mut ih = 0;
                    for j in 0..n {
                        vquad += self.d[j] * self.gopt[j];
                        for i in 0..=j {
                            let mut temp = self.d[i] * self.d[j];
                            if i == j {
                                temp *= 0.5;
                            }
                            vquad += self.hq[ih] * temp;
                            ih += 1;
                        }
                    }
                    for k in 0..npt {
                        vquad += 0.5 * self.pq[k] * work2[k] * work2[k];
                    }
                    let diff = f_val - fopt - vquad;
                    diffc = diffb;
                    diffb = diffa;
                    diffa = diff.abs();
                    if dnorm > rho {
                        nfsav = self.evals;
                    }

                    // Update delta
                    if ntrits > 0 {
                        if vquad >= 0.0 {
                            // Trust region step failed — return current best
                            state = 720;
                            continue;
                        }
                        ratio = (f_val - fopt) / vquad;
                        let h_delta = 0.5 * delta;
                        if ratio <= 0.1 {
                            delta = h_delta.min(dnorm);
                        } else if ratio <= 0.7 {
                            delta = h_delta.max(dnorm);
                        } else {
                            delta = h_delta.max(2.0 * dnorm);
                        }
                        if delta <= rho * 1.5 {
                            delta = rho;
                        }

                        // Recalculate knew if f < fopt
                        if f_val < fopt {
                            let ksav = knew;
                            let densav = denom;
                            let delsq = delta * delta;
                            scaden = 0.0;
                            biglsq = 0.0;
                            knew = 0;
                            for k in 0..npt {
                                let mut hdiag = 0.0;
                                for m in 0..nptm {
                                    hdiag += self.z_matrix[k][m] * self.z_matrix[k][m];
                                }
                                let den = beta * hdiag + self.vlag[k] * self.vlag[k];
                                distsq = 0.0;
                                for j in 0..n {
                                    let d2 = self.xpt[k][j] - self.xnew[j];
                                    distsq += d2 * d2;
                                }
                                let d3 = distsq / delsq;
                                let temp = (1.0_f64).max(d3 * d3);
                                if temp * den > scaden {
                                    scaden = temp * den;
                                    knew = k;
                                    denom = den;
                                }
                                biglsq = biglsq.max(temp * self.vlag[k] * self.vlag[k]);
                            }
                            if scaden <= 0.5 * biglsq {
                                knew = ksav;
                                denom = densav;
                            }
                        }
                    }

                    // Update model
                    self.update(beta, denom, knew);

                    ih = 0;
                    let pqold = self.pq[knew];
                    self.pq[knew] = 0.0;
                    for i in 0..n {
                        let temp = pqold * self.xpt[knew][i];
                        for j in 0..=i {
                            self.hq[ih] += temp * self.xpt[knew][j];
                            ih += 1;
                        }
                    }
                    for m in 0..nptm {
                        let temp = diff * self.z_matrix[knew][m];
                        for k in 0..npt {
                            self.pq[k] += temp * self.z_matrix[k][m];
                        }
                    }

                    // Update interpolation point
                    self.f_at_interp[knew] = f_val;
                    for i in 0..n {
                        self.xpt[knew][i] = self.xnew[i];
                        work1[i] = self.b_matrix[knew][i];
                    }
                    for k in 0..npt {
                        let mut suma = 0.0;
                        for m in 0..nptm {
                            suma += self.z_matrix[knew][m] * self.z_matrix[k][m];
                        }
                        let mut sumb = 0.0;
                        for j in 0..n {
                            sumb += self.xpt[k][j] * self.xopt[j];
                        }
                        let temp = suma * sumb;
                        for i in 0..n {
                            work1[i] += temp * self.xpt[k][i];
                        }
                    }
                    for i in 0..n {
                        self.gopt[i] += diff * work1[i];
                    }

                    // Update xopt/gopt if f < fopt
                    if f_val < fopt {
                        self.trust_region_center_index = knew;
                        xoptsq = 0.0;
                        ih = 0;
                        for j in 0..n {
                            self.xopt[j] = self.xnew[j];
                            xoptsq += self.xopt[j] * self.xopt[j];
                            for i in 0..=j {
                                if i < j {
                                    self.gopt[j] += self.hq[ih] * self.d[i];
                                }
                                self.gopt[i] += self.hq[ih] * self.d[j];
                                ih += 1;
                            }
                        }
                        for k in 0..npt {
                            let mut temp = 0.0;
                            for j in 0..n {
                                temp += self.xpt[k][j] * self.d[j];
                            }
                            temp *= self.pq[k];
                            for i in 0..n {
                                self.gopt[i] += temp * self.xpt[k][i];
                            }
                        }
                    }

                    // Test for model replacement
                    if ntrits > 0 {
                        for k in 0..npt {
                            self.vlag[k] = self.f_at_interp[k]
                                - self.f_at_interp[self.trust_region_center_index];
                            work3[k] = 0.0;
                        }
                        for j in 0..nptm {
                            let mut sum = 0.0;
                            for k in 0..npt {
                                sum += self.z_matrix[k][j] * self.vlag[k];
                            }
                            for k in 0..npt {
                                work3[k] += sum * self.z_matrix[k][j];
                            }
                        }
                        for k in 0..npt {
                            let mut sum = 0.0;
                            for j in 0..n {
                                sum += self.xpt[k][j] * self.xopt[j];
                            }
                            work2[k] = work3[k];
                            work3[k] = sum * work3[k];
                        }
                        let mut gqsq = 0.0;
                        let mut gisq = 0.0;
                        for i in 0..n {
                            let mut sum = 0.0;
                            for k in 0..npt {
                                sum += self.b_matrix[k][i] * self.vlag[k]
                                    + self.xpt[k][i] * work3[k];
                            }
                            if self.xopt[i] == self.sl[i] {
                                gqsq += self.gopt[i].min(0.0).powi(2);
                                gisq += sum.min(0.0).powi(2);
                            } else if self.xopt[i] == self.su[i] {
                                gqsq += self.gopt[i].max(0.0).powi(2);
                                gisq += sum.max(0.0).powi(2);
                            } else {
                                gqsq += self.gopt[i] * self.gopt[i];
                                gisq += sum * sum;
                            }
                            self.vlag[npt + i] = sum;
                        }

                        itest += 1;
                        if gqsq < 10.0 * gisq {
                            itest = 0;
                        }
                        if itest >= 3 {
                            let nh = n * np / 2;
                            let max_val = npt.max(nh);
                            for i in 0..max_val {
                                if i < n {
                                    self.gopt[i] = self.vlag[npt + i];
                                }
                                if i < npt {
                                    self.pq[i] = work2[i];
                                }
                                if i < nh {
                                    self.hq[i] = 0.0;
                                }
                            }
                            itest = 0;
                        }
                    }

                    if ntrits == 0 {
                        state = 60;
                        continue;
                    }
                    if f_val <= fopt + 0.1 * vquad {
                        state = 60;
                        continue;
                    }

                    distsq = (2.0 * delta).powi(2).max((10.0 * rho).powi(2));
                    state = 650;
                }
                650 => {
                    // Check for distant interpolation points
                    knew = npt; // sentinel: invalid
                    let mut max_distsq = distsq;
                    for k in 0..npt {
                        let mut sum = 0.0;
                        for j in 0..n {
                            let d1 = self.xpt[k][j] - self.xopt[j];
                            sum += d1 * d1;
                        }
                        if sum > max_distsq {
                            knew = k;
                            max_distsq = sum;
                        }
                    }
                    distsq = max_distsq;

                    if knew < npt {
                        let dist = distsq.sqrt();
                        if ntrits == -1 {
                            delta = (0.1 * delta).min(0.5 * dist);
                            if delta <= rho * 1.5 {
                                delta = rho;
                            }
                        }
                        ntrits = 0;
                        adelt = (0.1 * dist).min(delta).max(rho);
                        dsq = adelt * adelt;
                        state = 90;
                        continue;
                    }
                    if ntrits == -1 {
                        state = 680;
                        continue;
                    }
                    if ratio > 0.0 {
                        state = 60;
                        continue;
                    }
                    if delta.max(dnorm) > rho {
                        state = 60;
                        continue;
                    }
                    state = 680;
                }
                680 => {
                    // Reduce rho or terminate
                    if rho > self.stopping_trust_region_radius {
                        delta = 0.5 * rho;
                        ratio = rho / self.stopping_trust_region_radius;
                        if ratio <= 16.0 {
                            rho = self.stopping_trust_region_radius;
                        } else if ratio <= 250.0 {
                            rho = ratio.sqrt() * self.stopping_trust_region_radius;
                        } else {
                            rho *= 0.1;
                        }
                        delta = delta.max(rho);
                        ntrits = 0;
                        nfsav = self.evals;
                        state = 60;
                        continue;
                    }
                    if ntrits == -1 {
                        state = 360;
                        continue;
                    }
                    state = 720;
                }
                720 => {
                    // Return
                    if self.f_at_interp[self.trust_region_center_index] <= fsave {
                        for i in 0..n {
                            let d3 = lower_bound[i];
                            let d4 = self.origin_shift[i] + self.xopt[i];
                            let d1 = d3.max(d4);
                            let d2 = upper_bound[i];
                            self.current_best[i] = d1.min(d2);
                            if self.xopt[i] == self.sl[i] {
                                self.current_best[i] = lower_bound[i];
                            }
                            if self.xopt[i] == self.su[i] {
                                self.current_best[i] = upper_bound[i];
                            }
                        }
                        f_val = self.f_at_interp[self.trust_region_center_index];
                    } else {
                        f_val = fsave;
                    }
                    return f_val;
                }
                _ => {
                    // Should not happen
                    return self.f_at_interp[self.trust_region_center_index];
                }
            }
        }
    }

    // ── prelim: initialize interpolation ────────────────────────────
    //
    // Builds the initial set of `npt` interpolation points and evaluates
    // the objective at each. Initializes the quadratic model (gopt, hq, pq)
    // and the factored inverse interpolation matrix (b_matrix, z_matrix).

    fn prelim(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        lower_bound: &[f64],
        upper_bound: &[f64],
    ) {
        let n = self.n;
        let npt = self.npt;
        let rhosq = self.initial_trust_region_radius * self.initial_trust_region_radius;
        let recip = 1.0 / rhosq;
        let np = n + 1;

        // Initialize arrays to zero
        for j in 0..n {
            self.origin_shift[j] = self.current_best[j];
            for k in 0..npt {
                self.xpt[k][j] = 0.0;
            }
            for i in 0..(npt + n) {
                self.b_matrix[i][j] = 0.0;
            }
        }
        for i in 0..(n * np / 2) {
            self.hq[i] = 0.0;
        }
        for k in 0..npt {
            self.pq[k] = 0.0;
            for j in 0..(npt - np) {
                self.z_matrix[k][j] = 0.0;
            }
        }

        let mut ipt: usize = 0;
        let mut jpt: usize = 0;
        let mut fbeg = 0.0_f64;

        loop {
            let nfm = self.evals;
            let nfx = if nfm >= n { nfm - n } else { 0 };
            let nfmm = if nfm >= 1 { nfm - 1 } else { 0 };
            let nfxm = if nfx >= 1 { nfx - 1 } else { 0 };
            let mut stepa = 0.0_f64;
            let mut stepb = 0.0_f64;

            if nfm <= 2 * n {
                if nfm >= 1 && nfm <= n {
                    stepa = self.initial_trust_region_radius;
                    if self.su[nfmm] == 0.0 {
                        stepa = -stepa;
                    }
                    self.xpt[nfm][nfmm] = stepa;
                } else if nfm > n {
                    stepa = self.xpt[nfx][nfxm];
                    stepb = -self.initial_trust_region_radius;
                    if self.sl[nfxm] == 0.0 {
                        stepb = (2.0 * self.initial_trust_region_radius).min(self.su[nfxm]);
                    }
                    if self.su[nfxm] == 0.0 {
                        stepb = (-2.0 * self.initial_trust_region_radius).max(self.sl[nfxm]);
                    }
                    self.xpt[nfm][nfxm] = stepb;
                }
            } else {
                let tmp1 = (nfm - np) / n;
                jpt = nfm - tmp1 * n - n;
                ipt = jpt + tmp1;
                if ipt > n {
                    let tmp2 = jpt;
                    jpt = ipt - n;
                    ipt = tmp2;
                }
                let ipt_m1 = ipt - 1;
                let jpt_m1 = jpt - 1;
                self.xpt[nfm][ipt_m1] = self.xpt[ipt][ipt_m1];
                self.xpt[nfm][jpt_m1] = self.xpt[jpt][jpt_m1];
            }

            // Evaluate function
            for j in 0..n {
                self.current_best[j] = (self.origin_shift[j] + self.xpt[nfm][j])
                    .max(lower_bound[j])
                    .min(upper_bound[j]);
                if self.xpt[nfm][j] == self.sl[j] {
                    self.current_best[j] = lower_bound[j];
                }
                if self.xpt[nfm][j] == self.su[j] {
                    self.current_best[j] = upper_bound[j];
                }
            }

            let fv = self.compute_objective(f, &self.current_best.clone());
            self.f_at_interp[nfm] = fv;
            let num_eval = self.evals;

            if num_eval == 1 {
                fbeg = fv;
                self.trust_region_center_index = 0;
            } else if fv < self.f_at_interp[self.trust_region_center_index] {
                self.trust_region_center_index = nfm;
            }

            // Set bmat and model elements
            if num_eval <= 2 * n + 1 {
                if num_eval >= 2 && num_eval <= n + 1 {
                    self.gopt[nfmm] = (fv - fbeg) / stepa;
                    if npt < num_eval + n {
                        let one_over_step_a = 1.0 / stepa;
                        self.b_matrix[0][nfmm] = -one_over_step_a;
                        self.b_matrix[nfm][nfmm] = one_over_step_a;
                        self.b_matrix[npt + nfmm][nfmm] = -0.5 * rhosq;
                    }
                } else if num_eval >= n + 2 {
                    let ih = nfx * (nfx + 1) / 2 - 1;
                    let tmp = (fv - fbeg) / stepb;
                    let ddiff = stepb - stepa;
                    self.hq[ih] = 2.0 * (tmp - self.gopt[nfxm]) / ddiff;
                    self.gopt[nfxm] = (self.gopt[nfxm] * stepb - tmp * stepa) / ddiff;
                    if stepa * stepb < 0.0 && fv < self.f_at_interp[nfm - n] {
                        self.f_at_interp[nfm] = self.f_at_interp[nfm - n];
                        self.f_at_interp[nfm - n] = fv;
                        if self.trust_region_center_index == nfm {
                            self.trust_region_center_index = nfm - n;
                        }
                        self.xpt[nfm - n][nfxm] = stepb;
                        self.xpt[nfm][nfxm] = stepa;
                    }
                    self.b_matrix[0][nfxm] = -(stepa + stepb) / (stepa * stepb);
                    self.b_matrix[nfm][nfxm] = -0.5 / self.xpt[nfm - n][nfxm];
                    self.b_matrix[nfm - n][nfxm] =
                        -self.b_matrix[0][nfxm] - self.b_matrix[nfm][nfxm];
                    self.z_matrix[0][nfxm] = (2.0_f64).sqrt() / (stepa * stepb);
                    self.z_matrix[nfm][nfxm] = (0.5_f64).sqrt() / rhosq;
                    self.z_matrix[nfm - n][nfxm] =
                        -self.z_matrix[0][nfxm] - self.z_matrix[nfm][nfxm];
                }
            } else {
                self.z_matrix[0][nfxm] = recip;
                self.z_matrix[nfm][nfxm] = recip;
                self.z_matrix[ipt][nfxm] = -recip;
                self.z_matrix[jpt][nfxm] = -recip;

                let ih = ipt * (ipt - 1) / 2 + jpt - 1;
                let tmp = self.xpt[nfm][ipt - 1] * self.xpt[nfm][jpt - 1];
                self.hq[ih] =
                    (fbeg - self.f_at_interp[ipt] - self.f_at_interp[jpt] + fv) / tmp;
            }

            if self.evals >= npt {
                break;
            }
        }
    }

    // ── trsbox: trust region subproblem ─────────────────────────────
    //
    // Minimizes the quadratic model within the intersection of the trust
    // region and the bound constraints. Uses a truncated conjugate gradient
    // method. Returns `(dsq, crvmin)` where `dsq` is |d|^2 and `crvmin` is
    // the minimum curvature encountered (0.0 if a bound was hit).

    fn trsbox(&mut self, delta: f64) -> (f64, f64) {
        let n = self.n;
        let npt = self.npt;

        let mut gnew = vec![0.0; n];
        let mut xbdi = vec![0.0; n];
        let mut s = vec![0.0; n];
        let mut hs = vec![0.0; n];
        let mut hred = vec![0.0; n];

        let mut dsq: f64;
        let mut crvmin: f64;

        let mut ds: f64;
        let mut dhd: f64;
        let mut dhs: f64;
        let mut cth: f64;
        let mut shs: f64;
        let mut sth: f64;
        let mut ssq: f64;
        let mut beta: f64 = 0.0;
        let mut sdec: f64;
        let mut blen: f64;
        let mut iact: i32 = -1;
        let mut nact: usize = 0;
        let mut angt: f64 = 0.0;
        let mut qred: f64;
        let mut temp: f64;
        let mut xsav: f64 = 0.0;
        let mut xsum: f64;
        let mut angbd: f64 = 0.0;
        let mut dredg: f64 = 0.0;
        let mut sredg: f64 = 0.0;
        let mut iterc: usize;
        let mut delsq: f64;
        let mut ggsav: f64 = 0.0;
        let mut tempa: f64;
        let mut tempb: f64;
        let mut dredsq: f64 = 0.0;
        let mut gredsq: f64 = 0.0;
        let mut stplen: f64;
        let mut stepsq: f64 = 0.0;
        let mut itermax: usize = 0;
        let mut itcsav: usize = 0;
        let mut rdprev: f64 = 0.0;
        let mut rdnext: f64 = 0.0;
        let mut redmax: f64;
        let mut redsav: f64;
        let mut rednew: f64;
        let mut isav: i32;
        let mut iu: usize = 0;

        // Initialize
        iterc = 0;
        nact = 0;
        for i in 0..n {
            xbdi[i] = 0.0;
            if self.xopt[i] <= self.sl[i] {
                if self.gopt[i] >= 0.0 {
                    xbdi[i] = -1.0;
                }
            } else if self.xopt[i] >= self.su[i] && self.gopt[i] <= 0.0 {
                xbdi[i] = 1.0;
            }
            if xbdi[i] != 0.0 {
                nact += 1;
            }
            self.d[i] = 0.0;
            gnew[i] = self.gopt[i];
        }
        delsq = delta * delta;
        qred = 0.0;
        crvmin = -1.0;

        let mut state: i32 = 20;
        loop {
            match state {
                20 => {
                    beta = 0.0;
                    state = 30;
                }
                30 => {
                    stepsq = 0.0;
                    for i in 0..n {
                        if xbdi[i] != 0.0 {
                            s[i] = 0.0;
                        } else if beta == 0.0 {
                            s[i] = -gnew[i];
                        } else {
                            s[i] = beta * s[i] - gnew[i];
                        }
                        stepsq += s[i] * s[i];
                    }
                    if stepsq == 0.0 {
                        state = 190;
                        continue;
                    }
                    if beta == 0.0 {
                        gredsq = stepsq;
                        itermax = iterc + n - nact;
                    }
                    if gredsq * delsq <= qred * 1e-4 * qred {
                        state = 190;
                        continue;
                    }
                    state = 210;
                    continue;
                }
                50 => {
                    let mut resid = delsq;
                    ds = 0.0;
                    shs = 0.0;
                    for i in 0..n {
                        if xbdi[i] == 0.0 {
                            resid -= self.d[i] * self.d[i];
                            ds += s[i] * self.d[i];
                            shs += s[i] * hs[i];
                        }
                    }
                    if resid <= 0.0 {
                        state = 90;
                        continue;
                    }
                    temp = (stepsq * resid + ds * ds).sqrt();
                    if ds < 0.0 {
                        blen = (temp - ds) / stepsq;
                    } else {
                        blen = resid / (temp + ds);
                    }
                    stplen = blen;
                    if shs > 0.0 {
                        stplen = blen.min(gredsq / shs);
                    }

                    iact = -1;
                    for i in 0..n {
                        if s[i] != 0.0 {
                            xsum = self.xopt[i] + self.d[i];
                            if s[i] > 0.0 {
                                temp = (self.su[i] - xsum) / s[i];
                            } else {
                                temp = (self.sl[i] - xsum) / s[i];
                            }
                            if temp < stplen {
                                stplen = temp;
                                iact = i as i32;
                            }
                        }
                    }

                    sdec = 0.0;
                    if stplen > 0.0 {
                        iterc += 1;
                        temp = shs / stepsq;
                        if iact == -1 && temp > 0.0 {
                            crvmin = crvmin.min(temp);
                            if crvmin == -1.0 {
                                crvmin = temp;
                            }
                        }
                        ggsav = gredsq;
                        gredsq = 0.0;
                        for i in 0..n {
                            gnew[i] += stplen * hs[i];
                            if xbdi[i] == 0.0 {
                                gredsq += gnew[i] * gnew[i];
                            }
                            self.d[i] += stplen * s[i];
                        }
                        sdec = (stplen * (ggsav - 0.5 * stplen * shs)).max(0.0);
                        qred += sdec;
                    }

                    if iact >= 0 {
                        nact += 1;
                        let ia = iact as usize;
                        xbdi[ia] = 1.0;
                        if s[ia] < 0.0 {
                            xbdi[ia] = -1.0;
                        }
                        delsq -= self.d[ia] * self.d[ia];
                        if delsq <= 0.0 {
                            state = 190;
                            continue;
                        }
                        state = 20;
                        continue;
                    }

                    if stplen < blen {
                        if iterc == itermax {
                            state = 190;
                            continue;
                        }
                        if sdec <= qred * 0.01 {
                            state = 190;
                            continue;
                        }
                        beta = gredsq / ggsav;
                        state = 30;
                        continue;
                    }
                    state = 90;
                }
                90 => {
                    crvmin = 0.0;
                    state = 100;
                }
                100 => {
                    if nact >= n - 1 {
                        state = 190;
                        continue;
                    }
                    dredsq = 0.0;
                    dredg = 0.0;
                    gredsq = 0.0;
                    for i in 0..n {
                        if xbdi[i] == 0.0 {
                            dredsq += self.d[i] * self.d[i];
                            dredg += self.d[i] * gnew[i];
                            gredsq += gnew[i] * gnew[i];
                            s[i] = self.d[i];
                        } else {
                            s[i] = 0.0;
                        }
                    }
                    itcsav = iterc;
                    state = 210;
                    continue;
                }
                120 => {
                    iterc += 1;
                    temp = gredsq * dredsq - dredg * dredg;
                    if temp <= qred * 1e-4 * qred {
                        state = 190;
                        continue;
                    }
                    temp = temp.sqrt();
                    for i in 0..n {
                        if xbdi[i] == 0.0 {
                            s[i] = (dredg * self.d[i] - dredsq * gnew[i]) / temp;
                        } else {
                            s[i] = 0.0;
                        }
                    }
                    sredg = -temp;

                    angbd = 1.0;
                    iact = -1;
                    let mut goto_100 = false;
                    for i in 0..n {
                        if xbdi[i] == 0.0 {
                            tempa = self.xopt[i] + self.d[i] - self.sl[i];
                            tempb = self.su[i] - self.xopt[i] - self.d[i];
                            if tempa <= 0.0 {
                                nact += 1;
                                xbdi[i] = -1.0;
                                goto_100 = true;
                                break;
                            } else if tempb <= 0.0 {
                                nact += 1;
                                xbdi[i] = 1.0;
                                goto_100 = true;
                                break;
                            }
                            ssq = self.d[i] * self.d[i] + s[i] * s[i];
                            temp = ssq - (self.xopt[i] - self.sl[i]).powi(2);
                            if temp > 0.0 {
                                temp = temp.sqrt() - s[i];
                                if angbd * temp > tempa {
                                    angbd = tempa / temp;
                                    iact = i as i32;
                                    xsav = -1.0;
                                }
                            }
                            temp = ssq - (self.su[i] - self.xopt[i]).powi(2);
                            if temp > 0.0 {
                                temp = temp.sqrt() + s[i];
                                if angbd * temp > tempb {
                                    angbd = tempb / temp;
                                    iact = i as i32;
                                    xsav = 1.0;
                                }
                            }
                        }
                    }
                    if goto_100 {
                        state = 100;
                        continue;
                    }

                    state = 210;
                    continue;
                }
                150 => {
                    shs = 0.0;
                    dhs = 0.0;
                    dhd = 0.0;
                    for i in 0..n {
                        if xbdi[i] == 0.0 {
                            shs += s[i] * hs[i];
                            dhs += self.d[i] * hs[i];
                            dhd += self.d[i] * hred[i];
                        }
                    }

                    redmax = 0.0;
                    isav = -1;
                    redsav = 0.0;
                    iu = (angbd * 17.0 + 3.1) as usize;
                    for i in 0..iu {
                        angt = angbd * (i as f64) / (iu as f64);
                        sth = (angt + angt) / (1.0 + angt * angt);
                        temp = shs + angt * (angt * dhd - dhs - dhs);
                        rednew = sth * (angt * dredg - sredg - 0.5 * sth * temp);
                        if rednew > redmax {
                            redmax = rednew;
                            isav = i as i32;
                            rdprev = redsav;
                        } else if i as i32 == isav + 1 {
                            rdnext = rednew;
                        }
                        redsav = rednew;
                    }

                    if isav < 0 {
                        state = 190;
                        continue;
                    }
                    if (isav as usize) < iu {
                        temp = (rdnext - rdprev) / (redmax + redmax - rdprev - rdnext);
                        angt = angbd * (isav as f64 + 0.5 * temp) / (iu as f64);
                    }
                    cth = (1.0 - angt * angt) / (1.0 + angt * angt);
                    sth = (angt + angt) / (1.0 + angt * angt);
                    temp = shs + angt * (angt * dhd - dhs - dhs);
                    sdec = sth * (angt * dredg - sredg - 0.5 * sth * temp);
                    if sdec <= 0.0 {
                        state = 190;
                        continue;
                    }

                    dredg = 0.0;
                    gredsq = 0.0;
                    for i in 0..n {
                        gnew[i] += (cth - 1.0) * hred[i] + sth * hs[i];
                        if xbdi[i] == 0.0 {
                            self.d[i] = cth * self.d[i] + sth * s[i];
                            dredg += self.d[i] * gnew[i];
                            gredsq += gnew[i] * gnew[i];
                        }
                        hred[i] = cth * hred[i] + sth * hs[i];
                    }
                    qred += sdec;
                    if iact >= 0 && isav as usize == iu {
                        nact += 1;
                        xbdi[iact as usize] = xsav;
                        state = 100;
                        continue;
                    }

                    if sdec > qred * 0.01 {
                        state = 120;
                        continue;
                    }
                    state = 190;
                }
                190 => {
                    dsq = 0.0;
                    for i in 0..n {
                        let min_val = (self.xopt[i] + self.d[i]).min(self.su[i]);
                        self.xnew[i] = min_val.max(self.sl[i]);
                        if xbdi[i] == -1.0 {
                            self.xnew[i] = self.sl[i];
                        }
                        if xbdi[i] == 1.0 {
                            self.xnew[i] = self.su[i];
                        }
                        self.d[i] = self.xnew[i] - self.xopt[i];
                        dsq += self.d[i] * self.d[i];
                    }
                    return (dsq, crvmin);
                }
                210 => {
                    // Multiply S by the model Hessian
                    let mut ih = 0;
                    for j in 0..n {
                        hs[j] = 0.0;
                        for i in 0..=j {
                            if i < j {
                                hs[j] += self.hq[ih] * s[i];
                            }
                            hs[i] += self.hq[ih] * s[j];
                            ih += 1;
                        }
                    }
                    // xpt.operate(s) * pq
                    let mut tmp_vec = vec![0.0; npt];
                    for k in 0..npt {
                        let mut sum = 0.0;
                        for j in 0..n {
                            sum += self.xpt[k][j] * s[j];
                        }
                        tmp_vec[k] = sum * self.pq[k];
                    }
                    for k in 0..npt {
                        if self.pq[k] != 0.0 {
                            for i in 0..n {
                                hs[i] += tmp_vec[k] * self.xpt[k][i];
                            }
                        }
                    }
                    if crvmin != 0.0 {
                        state = 50;
                        continue;
                    }
                    if iterc > itcsav {
                        state = 150;
                        continue;
                    }
                    for i in 0..n {
                        hred[i] = hs[i];
                    }
                    state = 120;
                }
                _ => {
                    return (0.0, 0.0);
                }
            }
        }
    }

    // ── altmov: alternative move ────────────────────────────────────
    //
    // Computes an alternative trial step to replace interpolation point
    // `knew`, maximizing |Lagrange function| to improve model geometry.
    // Returns `(alpha, cauchy)` where `alpha` is the Lagrange value for
    // the Newton step and `cauchy` for the Cauchy (gradient) step.

    fn altmov(&mut self, knew: usize, adelt: f64) -> (f64, f64) {
        let n = self.n;
        let npt = self.npt;

        let mut glag = vec![0.0; n];
        let mut hcol = vec![0.0; npt];
        let mut work1 = vec![0.0; n];
        let mut work2 = vec![0.0; n];

        for k in 0..npt {
            hcol[k] = 0.0;
        }
        for j in 0..(npt - n - 1) {
            let tmp = self.z_matrix[knew][j];
            for k in 0..npt {
                hcol[k] += tmp * self.z_matrix[k][j];
            }
        }
        let alpha = hcol[knew];
        let ha = 0.5 * alpha;

        // Gradient of the KNEW-th Lagrange function at xopt
        for i in 0..n {
            glag[i] = self.b_matrix[knew][i];
        }
        for k in 0..npt {
            let mut tmp = 0.0;
            for j in 0..n {
                tmp += self.xpt[k][j] * self.xopt[j];
            }
            tmp *= hcol[k];
            for i in 0..n {
                glag[i] += tmp * self.xpt[k][i];
            }
        }

        // Search for large denominator along lines through xopt and other points
        let mut presav = 0.0_f64;
        let mut ksav: usize = 0;
        let mut ibdsav: i32 = 0;
        let mut stpsav = 0.0_f64;

        for k in 0..npt {
            if k == self.trust_region_center_index {
                continue;
            }
            let mut dderiv = 0.0;
            let mut distsq = 0.0;
            for i in 0..n {
                let tmp = self.xpt[k][i] - self.xopt[i];
                dderiv += glag[i] * tmp;
                distsq += tmp * tmp;
            }
            let mut subd = adelt / distsq.sqrt();
            let mut slbd = -subd;
            let mut ilbd: i32 = 0;
            let mut iubd: i32 = 0;
            let sumin = 1.0_f64.min(subd);

            for i in 0..n {
                let tmp = self.xpt[k][i] - self.xopt[i];
                if tmp > 0.0 {
                    if slbd * tmp < self.sl[i] - self.xopt[i] {
                        slbd = (self.sl[i] - self.xopt[i]) / tmp;
                        ilbd = -(i as i32) - 1;
                    }
                    if subd * tmp > self.su[i] - self.xopt[i] {
                        subd = sumin.max((self.su[i] - self.xopt[i]) / tmp);
                        iubd = (i as i32) + 1;
                    }
                } else if tmp < 0.0 {
                    if slbd * tmp > self.su[i] - self.xopt[i] {
                        slbd = (self.su[i] - self.xopt[i]) / tmp;
                        ilbd = (i as i32) + 1;
                    }
                    if subd * tmp < self.sl[i] - self.xopt[i] {
                        subd = sumin.max((self.sl[i] - self.xopt[i]) / tmp);
                        iubd = -(i as i32) - 1;
                    }
                }
            }

            let mut step = slbd;
            let mut isbd = ilbd;
            let vlag_val;
            if k == knew {
                let ddiff = dderiv - 1.0;
                let mut vl = slbd * (dderiv - slbd * ddiff);
                let d1 = subd * (dderiv - subd * ddiff);
                if d1.abs() > vl.abs() {
                    step = subd;
                    vl = d1;
                    isbd = iubd;
                }
                let d2 = 0.5 * dderiv;
                let d3 = d2 - ddiff * slbd;
                let d4 = d2 - ddiff * subd;
                if d3 * d4 < 0.0 {
                    let d5 = d2 * d2 / ddiff;
                    if d5.abs() > vl.abs() {
                        step = d2 / ddiff;
                        vl = d5;
                        isbd = 0;
                    }
                }
                vlag_val = vl;
            } else {
                let mut vl = slbd * (1.0 - slbd);
                let tmp = subd * (1.0 - subd);
                if tmp.abs() > vl.abs() {
                    step = subd;
                    vl = tmp;
                    isbd = iubd;
                }
                if subd > 0.5 && vl.abs() < 0.25 {
                    step = 0.5;
                    vl = 0.25;
                    isbd = 0;
                }
                vlag_val = vl * dderiv;
            }

            let tmp = step * (1.0 - step) * distsq;
            let predsq = vlag_val * vlag_val * (vlag_val * vlag_val + ha * tmp * tmp);
            if predsq > presav {
                presav = predsq;
                ksav = k;
                stpsav = step;
                ibdsav = isbd;
            }
        }

        // Construct xnew
        for i in 0..n {
            let tmp = self.xopt[i]
                + stpsav * (self.xpt[ksav][i] - self.xopt[i]);
            self.xnew[i] = self.sl[i].max(self.su[i].min(tmp));
        }
        if ibdsav < 0 {
            let idx = (-ibdsav - 1) as usize;
            self.xnew[idx] = self.sl[idx];
        }
        if ibdsav > 0 {
            let idx = (ibdsav - 1) as usize;
            self.xnew[idx] = self.su[idx];
        }

        // Constrained Cauchy step
        let bigstp = adelt + adelt;
        let mut iflag = 0;
        let mut cauchy: f64 = 0.0;
        let mut csave = 0.0_f64;

        loop {
            let mut wfixsq = 0.0;
            let mut ggfree = 0.0;
            for i in 0..n {
                work1[i] = 0.0;
                if (self.xopt[i] - self.sl[i]).min(glag[i]) > 0.0
                    || (self.xopt[i] - self.su[i]).max(glag[i]) < 0.0
                {
                    work1[i] = bigstp;
                    ggfree += glag[i] * glag[i];
                }
            }
            if ggfree == 0.0 {
                return (alpha, 0.0);
            }

            let tmp1 = adelt * adelt - wfixsq;
            let step;
            if tmp1 > 0.0 {
                step = (tmp1 / ggfree).sqrt();
                ggfree = 0.0;
                for i in 0..n {
                    if work1[i] == bigstp {
                        let tmp2 = self.xopt[i] - step * glag[i];
                        if tmp2 <= self.sl[i] {
                            work1[i] = self.sl[i] - self.xopt[i];
                            wfixsq += work1[i] * work1[i];
                        } else if tmp2 >= self.su[i] {
                            work1[i] = self.su[i] - self.xopt[i];
                            wfixsq += work1[i] * work1[i];
                        } else {
                            ggfree += glag[i] * glag[i];
                        }
                    }
                }
            } else {
                step = 0.0;
            }

            let mut gw = 0.0;
            for i in 0..n {
                if work1[i] == bigstp {
                    work1[i] = -step * glag[i];
                    let min_val = self.su[i].min(self.xopt[i] + work1[i]);
                    self.xalt[i] = self.sl[i].max(min_val);
                } else if work1[i] == 0.0 {
                    self.xalt[i] = self.xopt[i];
                } else if glag[i] > 0.0 {
                    self.xalt[i] = self.sl[i];
                } else {
                    self.xalt[i] = self.su[i];
                }
                gw += glag[i] * work1[i];
            }

            // Curvature of KNEW-th Lagrange function along W
            let mut curv = 0.0;
            for k in 0..npt {
                let mut tmp = 0.0;
                for j in 0..n {
                    tmp += self.xpt[k][j] * work1[j];
                }
                curv += hcol[k] * tmp * tmp;
            }
            if iflag == 1 {
                curv = -curv;
            }
            if curv > -gw && curv < -gw * (1.0 + 2.0_f64.sqrt()) {
                let scale = -gw / curv;
                for i in 0..n {
                    let tmp = self.xopt[i] + scale * work1[i];
                    self.xalt[i] = self.sl[i].max(self.su[i].min(tmp));
                }
                cauchy = (0.5 * gw * scale).powi(2);
            } else {
                cauchy = (gw + 0.5 * curv).powi(2);
            }

            if iflag == 0 {
                for i in 0..n {
                    glag[i] = -glag[i];
                    work2[i] = self.xalt[i];
                }
                csave = cauchy;
                iflag = 1;
            } else {
                break;
            }
        }

        if csave > cauchy {
            for i in 0..n {
                self.xalt[i] = work2[i];
            }
            cauchy = csave;
        }

        (alpha, cauchy)
    }

    // ── update: model update ────────────────────────────────────────
    //
    // Updates the factored inverse interpolation matrix (b_matrix, z_matrix)
    // when interpolation point `knew` is replaced. Uses the Sherman–Morrison
    // formula to maintain H = Z Z^T + (terms in b_matrix).

    fn update(&mut self, beta: f64, denom: f64, knew: usize) {
        let n = self.n;
        let npt = self.npt;
        let nptm = npt - n - 1;

        let mut work = vec![0.0; npt + n];

        let mut ztest = 0.0_f64;
        for k in 0..npt {
            for j in 0..nptm {
                ztest = ztest.max(self.z_matrix[k][j].abs());
            }
        }
        ztest *= 1e-20;

        // Apply rotations to zero out the KNEW-th row of Z
        for j in 1..nptm {
            if self.z_matrix[knew][j].abs() > ztest {
                let d2 = self.z_matrix[knew][0];
                let d3 = self.z_matrix[knew][j];
                let d4 = (d2 * d2 + d3 * d3).sqrt();
                let d5 = self.z_matrix[knew][0] / d4;
                let d6 = self.z_matrix[knew][j] / d4;
                for i in 0..npt {
                    let d7 = d5 * self.z_matrix[i][0] + d6 * self.z_matrix[i][j];
                    self.z_matrix[i][j] = d5 * self.z_matrix[i][j] - d6 * self.z_matrix[i][0];
                    self.z_matrix[i][0] = d7;
                }
            }
            self.z_matrix[knew][j] = 0.0;
        }

        // Put the first NPT components of the KNEW-th column of H into work
        for i in 0..npt {
            work[i] = self.z_matrix[knew][0] * self.z_matrix[i][0];
        }
        let alpha = work[knew];
        let tau = self.vlag[knew];
        self.vlag[knew] -= 1.0;

        // Complete updating of Z
        // Match Java/Fortran: sqrt(denom). Denom should always be positive
        // because the algorithm selects `knew` to maximize it. If negative
        // (ill-conditioned), sqrt produces NaN and we bail out below.
        let sqrt_denom = denom.sqrt();
        if sqrt_denom == 0.0 || sqrt_denom.is_nan() {
            return;
        }
        let d1 = tau / sqrt_denom;
        let d2 = self.z_matrix[knew][0] / sqrt_denom;
        for i in 0..npt {
            self.z_matrix[i][0] = d1 * self.z_matrix[i][0] - d2 * self.vlag[i];
        }

        // Update B matrix
        for j in 0..n {
            let jp = npt + j;
            work[jp] = self.b_matrix[knew][j];
            let d3 =
                (alpha * self.vlag[jp] - tau * work[jp]) / denom;
            let d4 =
                (-beta * work[jp] - tau * self.vlag[jp]) / denom;
            for i in 0..=jp {
                self.b_matrix[i][j] += d3 * self.vlag[i] + d4 * work[i];
                if i >= npt {
                    self.b_matrix[jp][i - npt] = self.b_matrix[i][j];
                }
            }
        }

        // Restore vlag
        self.vlag[knew] = tau;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;
    use std::f64::consts::PI;

    // ── Test function definitions (matching ACM3 exactly) ───────────────

    fn sphere(x: &[f64]) -> f64 {
        x.iter().map(|v| v * v).sum()
    }

    fn cigar(x: &[f64]) -> f64 {
        let factor = 1e6;
        x[0] * x[0] + factor * x[1..].iter().map(|v| v * v).sum::<f64>()
    }

    fn tablet(x: &[f64]) -> f64 {
        let factor = 1e6;
        factor * x[0] * x[0] + x[1..].iter().map(|v| v * v).sum::<f64>()
    }

    fn cigtab(x: &[f64]) -> f64 {
        let factor = 1e4;
        let n = x.len();
        x[0] * x[0] / factor
            + factor * x[n - 1] * x[n - 1]
            + x[1..n - 1].iter().map(|v| v * v).sum::<f64>()
    }

    fn two_axes(x: &[f64]) -> f64 {
        let factor = 1e12;
        let n = x.len();
        let mut sum = 0.0;
        for i in 0..n {
            if i < n / 2 {
                sum += factor * x[i] * x[i];
            } else {
                sum += x[i] * x[i];
            }
        }
        sum
    }

    fn elli(x: &[f64]) -> f64 {
        let factor: f64 = 1e6;
        let n = x.len();
        let mut sum = 0.0;
        for i in 0..n {
            sum += factor.powf(i as f64 / (n - 1) as f64) * x[i] * x[i];
        }
        sum
    }

    fn rosenbrock(x: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..x.len() - 1 {
            let t = x[i] * x[i] - x[i + 1];
            sum += 100.0 * t * t + (x[i] - 1.0) * (x[i] - 1.0);
        }
        sum
    }

    fn ackley(x: &[f64]) -> f64 {
        let n = x.len() as f64;
        let sum1: f64 = x.iter().map(|v| v * v).sum();
        let sum2: f64 = x.iter().map(|v| (2.0 * PI * v).cos()).sum();
        20.0 - 20.0 * (-0.2 * (sum1 / n).sqrt()).exp()
            + std::f64::consts::E
            - (sum2 / n).exp()
    }

    fn rastrigin(x: &[f64]) -> f64 {
        let a = 10.0;
        x.iter()
            .map(|v| v * v + a * (1.0 - (2.0 * PI * v).cos()))
            .sum()
    }

    fn diff_pow(x: &[f64]) -> f64 {
        let n = x.len();
        let mut sum = 0.0;
        for i in 0..n {
            sum += x[i].abs().powf(2.0 + 10.0 * i as f64 / (n - 1) as f64);
        }
        sum
    }

    /// Powell's "points in square" test function — sum of reciprocal distances.
    fn powell_points_in_square(x: &[f64], m: usize) -> f64 {
        let mut f = 0.0;
        for i in 1..m {
            for j in 0..i {
                let dx = x[2 * i] - x[2 * j];
                let dy = x[2 * i + 1] - x[2 * j + 1];
                let t = (dx * dx + dy * dy).max(1e-6);
                f += 1.0 / t.sqrt();
            }
        }
        f
    }

    fn powell_start(m: usize) -> Vec<f64> {
        let mut start = vec![0.0; 2 * m];
        for j in 0..m {
            let temp = (j + 1) as f64 * 2.0 * PI / m as f64;
            start[2 * j] = temp.cos();
            start[2 * j + 1] = temp.sin();
        }
        // Clamp to [-1, 1]
        for v in &mut start {
            *v = v.clamp(-1.0, 1.0);
        }
        start
    }

    // Helper: run BOBYQA with wide bounds (ACM3-style "unbounded")
    fn run_unbounded(
        func: &mut dyn FnMut(&[f64]) -> f64,
        dim: usize,
        start: &[f64],
        max_eval: usize,
    ) -> BobyqaResult {
        let lo = vec![-1e6; dim];
        let hi = vec![1e6; dim];
        bobyqa_minimize(func, start, &lo, &hi, 2 * dim + 1, 10.0, 1e-8, max_eval)
            .expect("BOBYQA should converge")
    }

    // ── ACM3 reference values (from Apache Commons Math 3.6.1) ──────────
    //
    // These are the exact evaluation counts and function values produced by
    // the Java BOBYQAOptimizer on the same test functions.
    // The Rust port should match these closely (same algorithm, same FP ops).

    // ── 1. Simple 2D tests ──────────────────────────────────────────────

    #[test]
    fn quadratic_2d() {
        // ACM3 ref: value=0.0, evals=28, point=(3,4)
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 3.0).powi(2) + (x[1] - 4.0).powi(2),
            &[0.0, 0.0],
            &[-10.0, -10.0],
            &[10.0, 10.0],
            5,
            1.0,
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 3.0, epsilon = 1e-6);
        assert_abs_diff_eq!(result.point[1], 4.0, epsilon = 1e-6);
        assert_abs_diff_eq!(result.value, 0.0, epsilon = 1e-10);
        assert!(
            result.evaluations <= 50,
            "too many evals: {} (ACM3: 28)",
            result.evaluations
        );
    }

    #[test]
    fn quadratic_2d_bounded() {
        // ACM3 ref: value=5.0, evals=27, point=(2,2)
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 3.0).powi(2) + (x[1] - 4.0).powi(2),
            &[1.0, 1.0],
            &[0.0, 0.0],
            &[2.0, 2.0],
            5,
            0.5,
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 2.0, epsilon = 1e-6);
        assert_abs_diff_eq!(result.point[1], 2.0, epsilon = 1e-6);
        assert_abs_diff_eq!(result.value, 5.0, epsilon = 1e-10);
        assert!(
            result.evaluations <= 50,
            "too many evals: {} (ACM3: 27)",
            result.evaluations
        );
    }

    #[test]
    fn rosenbrock_2d() {
        // ACM3 ref: value≈2.12e-23, evals=150, point≈(1,1)
        let result = bobyqa_minimize(
            &mut |x| rosenbrock(x),
            &[-1.0, -1.0],
            &[-5.0, -5.0],
            &[5.0, 5.0],
            5,
            1.0,
            1e-8,
            5000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 1.0, epsilon = 1e-6);
        assert_abs_diff_eq!(result.point[1], 1.0, epsilon = 1e-6);
        assert!(result.value < 1e-12, "value too high: {}", result.value);
        assert!(
            result.evaluations <= 300,
            "too many evals: {} (ACM3: 150)",
            result.evaluations
        );
    }

    // ── 2. ACM3 13-dimensional test suite ───────────────────────────────
    //
    // All use: dim=13, nInterp=27, initialTrust=10.0, stoppingTrust=1e-8
    // Bounds: [-1e6, 1e6] per dimension

    #[test]
    fn sphere_13d() {
        // ACM3 ref: value=0.0, evals=56
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut sphere, 13, &start, 1000);
        assert!(result.value < 1e-13, "sphere value: {}", result.value);
        for i in 0..13 {
            assert!(result.point[i].abs() < 1e-6, "sphere dim {i}: {}", result.point[i]);
        }
        assert!(result.evaluations <= 100, "sphere evals: {} (ACM3: 56)", result.evaluations);
    }

    #[test]
    fn cigar_13d() {
        // ACM3 ref: value≈4.93e-32, evals=56
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut cigar, 13, &start, 1000);
        assert!(result.value < 1e-13, "cigar value: {}", result.value);
        for i in 0..13 {
            assert!(result.point[i].abs() < 1e-6, "cigar dim {i}: {}", result.point[i]);
        }
        assert!(result.evaluations <= 100, "cigar evals: {} (ACM3: 56)", result.evaluations);
    }

    #[test]
    fn tablet_13d() {
        // ACM3 ref: value≈5.55e-28, evals=57
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut tablet, 13, &start, 1000);
        assert!(result.value < 1e-13, "tablet value: {}", result.value);
        for i in 0..13 {
            assert!(result.point[i].abs() < 1e-6, "tablet dim {i}: {}", result.point[i]);
        }
        assert!(result.evaluations <= 100, "tablet evals: {} (ACM3: 57)", result.evaluations);
    }

    #[test]
    fn cigtab_13d() {
        // ACM3 ref: value≈1.69e-38, evals=61
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut cigtab, 13, &start, 1000);
        assert!(result.value < 1e-13, "cigtab value: {}", result.value);
        for i in 0..13 {
            assert!(
                result.point[i].abs() < 5e-5,
                "cigtab dim {i}: {}",
                result.point[i]
            );
        }
        assert!(result.evaluations <= 100, "cigtab evals: {} (ACM3: 61)", result.evaluations);
    }

    #[test]
    fn two_axes_13d() {
        // ACM3 ref: value≈2.21e-29, evals=70
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut two_axes, 13, &start, 1000);
        assert!(result.value < 2e-13, "two_axes value: {}", result.value);
        for i in 0..13 {
            assert!(
                result.point[i].abs() < 1e-6,
                "two_axes dim {i}: {}",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 120,
            "two_axes evals: {} (ACM3: 70)",
            result.evaluations
        );
    }

    #[test]
    fn elli_13d() {
        // ACM3 ref: value≈1.67e-17, evals=147
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut elli, 13, &start, 1000);
        assert!(result.value < 1e-12, "elli value: {}", result.value);
        for i in 0..13 {
            assert!(result.point[i].abs() < 1e-6, "elli dim {i}: {}", result.point[i]);
        }
        assert!(result.evaluations <= 250, "elli evals: {} (ACM3: 147)", result.evaluations);
    }

    #[test]
    fn rosenbrock_13d() {
        // ACM3 ref: value≈7.04e-15, evals=1312
        let start = vec![0.1; 13];
        let result = run_unbounded(&mut rosenbrock, 13, &start, 5000);
        assert!(result.value < 1e-6, "rosenbrock value: {}", result.value);
        for i in 0..13 {
            assert!(
                (result.point[i] - 1.0).abs() < 1e-3,
                "rosenbrock dim {i}: {} (expected 1.0)",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 3000,
            "rosenbrock evals: {} (ACM3: 1312)",
            result.evaluations
        );
    }

    #[test]
    fn ackley_13d() {
        // ACM3 ref: value≈9.61e-9, evals=442
        let start = vec![0.1; 13];
        let result = run_unbounded(&mut ackley, 13, &start, 5000);
        assert!(result.value < 1e-7, "ackley value: {}", result.value);
        for i in 0..13 {
            assert!(
                result.point[i].abs() < 1e-5,
                "ackley dim {i}: {}",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 800,
            "ackley evals: {} (ACM3: 442)",
            result.evaluations
        );
    }

    #[test]
    fn rastrigin_13d() {
        // ACM3 ref: value=0.0, evals=166
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut rastrigin, 13, &start, 5000);
        assert!(result.value < 1e-13, "rastrigin value: {}", result.value);
        for i in 0..13 {
            assert!(
                result.point[i].abs() < 1e-6,
                "rastrigin dim {i}: {}",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 300,
            "rastrigin evals: {} (ACM3: 166)",
            result.evaluations
        );
    }

    #[test]
    fn diffpow_6d() {
        // ACM3 ref: value≈1.36e-18, evals=6016
        let start = vec![1.0; 6];
        let lo = vec![-1e6; 6];
        let hi = vec![1e6; 6];
        let result = bobyqa_minimize(&mut diff_pow, &start, &lo, &hi, 13, 10.0, 1e-8, 25000)
            .unwrap();
        assert!(result.value < 1e-8, "diffpow value: {}", result.value);
        // DiffPow is harder — point tolerance is looser (ACM3 uses 1e-1)
        for i in 0..6 {
            assert!(
                result.point[i].abs() < 0.1,
                "diffpow dim {i}: {}",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 12000,
            "diffpow evals: {} (ACM3: 6016)",
            result.evaluations
        );
    }

    // ── 3. Constrained Rosenbrock ───────────────────────────────────────

    #[test]
    fn rosenbrock_bounded_13d() {
        // ACM3 ref: value≈2.21e-14, evals=1311, bounds=[-1, 2]
        let start = vec![0.1; 13];
        let lo = vec![-1.0; 13];
        let hi = vec![2.0; 13];
        let result =
            bobyqa_minimize(&mut rosenbrock, &start, &lo, &hi, 27, 10.0, 1e-8, 5000).unwrap();
        assert!(
            result.value < 1e-6,
            "bounded rosen value: {}",
            result.value
        );
        for i in 0..13 {
            assert!(
                (result.point[i] - 1.0).abs() < 1e-3,
                "bounded rosen dim {i}: {}",
                result.point[i]
            );
        }
        assert!(
            result.evaluations <= 3000,
            "bounded rosen evals: {} (ACM3: 1311)",
            result.evaluations
        );
    }

    // ── 4. Powell's "points in square" test ─────────────────────────────
    //
    // From Powell's original Fortran BOBYQA test driver.
    // Tests with different m (points) and npt (interpolation pts).

    #[test]
    fn powell_m5_npt16() {
        // ACM3 ref: value=5.68035388808428, evals=123
        let m = 5;
        let start = powell_start(m);
        let n = 2 * m;
        let lo = vec![-1.0; n];
        let hi = vec![1.0; n];
        let result = bobyqa_minimize(
            &mut |x| powell_points_in_square(x, m),
            &start, &lo, &hi,
            16,   // n+6
            0.1,
            1e-6,
            500000,
        )
        .unwrap();

        // Same local minimum as Fortran reference
        assert!(
            (result.value - 5.6804).abs() < 0.01,
            "powell m5 npt16 value: {} (expected ~5.6804)",
            result.value
        );
        assert!(
            result.evaluations <= 300,
            "powell m5 npt16 evals: {} (ACM3: 123)",
            result.evaluations
        );
    }

    #[test]
    fn powell_m5_npt21() {
        // ACM3 ref: value=5.60153397218646, evals=98
        let m = 5;
        let start = powell_start(m);
        let n = 2 * m;
        let lo = vec![-1.0; n];
        let hi = vec![1.0; n];
        let result = bobyqa_minimize(
            &mut |x| powell_points_in_square(x, m),
            &start, &lo, &hi,
            21,   // 2n+1
            0.1,
            1e-6,
            500000,
        )
        .unwrap();

        // Better local minimum with more interpolation points
        assert!(
            (result.value - 5.6015).abs() < 0.01,
            "powell m5 npt21 value: {} (expected ~5.6015)",
            result.value
        );
        assert!(
            result.evaluations <= 200,
            "powell m5 npt21 evals: {} (ACM3: 98)",
            result.evaluations
        );
    }

    #[test]
    fn powell_m10_npt26() {
        // ACM3 ref: value=32.2030533688304, evals=235
        let m = 10;
        let start = powell_start(m);
        let n = 2 * m;
        let lo = vec![-1.0; n];
        let hi = vec![1.0; n];
        let result = bobyqa_minimize(
            &mut |x| powell_points_in_square(x, m),
            &start, &lo, &hi,
            26,   // n+6
            0.1,
            1e-6,
            500000,
        )
        .unwrap();

        assert!(
            (result.value - 32.203).abs() < 0.01,
            "powell m10 npt26 value: {} (expected ~32.203)",
            result.value
        );
        assert!(
            result.evaluations <= 500,
            "powell m10 npt26 evals: {} (ACM3: 235)",
            result.evaluations
        );
    }

    #[test]
    fn powell_m10_npt41() {
        // ACM3 ref: value=32.2030533688304, evals=194
        let m = 10;
        let start = powell_start(m);
        let n = 2 * m;
        let lo = vec![-1.0; n];
        let hi = vec![1.0; n];
        let result = bobyqa_minimize(
            &mut |x| powell_points_in_square(x, m),
            &start, &lo, &hi,
            41,   // 2n+1
            0.1,
            1e-6,
            500000,
        )
        .unwrap();

        assert!(
            (result.value - 32.203).abs() < 0.01,
            "powell m10 npt41 value: {} (expected ~32.203)",
            result.value
        );
        assert!(
            result.evaluations <= 400,
            "powell m10 npt41 evals: {} (ACM3: 194)",
            result.evaluations
        );
    }

    // ── 5. 13D quadratic (NAF geometry-scale test) ──────────────────────

    #[test]
    fn quadratic_13d() {
        let target: Vec<f64> = vec![
            0.30, 0.25, 0.025, 0.025, 0.04, 0.03, 0.03, 0.005, 0.006, 0.005, 0.007, 0.007,
            0.007,
        ];
        let lower: Vec<f64> = vec![
            0.10, 0.05, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001, 0.001,
            0.001,
        ];
        let upper: Vec<f64> = vec![
            0.70, 0.50, 0.070, 0.070, 0.070, 0.070, 0.070, 0.013, 0.013, 0.013, 0.013, 0.013,
            0.013,
        ];
        let start: Vec<f64> = vec![
            0.35, 0.30, 0.020, 0.020, 0.050, 0.020, 0.020, 0.003, 0.003, 0.003, 0.003, 0.003,
            0.003,
        ];
        let result = bobyqa_minimize(
            &mut |x| {
                x.iter()
                    .zip(&target)
                    .map(|(xi, ti)| (xi - ti).powi(2))
                    .sum()
            },
            &start,
            &lower,
            &upper,
            27,
            0.05,
            1e-8,
            5000,
        )
        .unwrap();

        for i in 0..13 {
            assert!(
                (result.point[i] - target[i]).abs() < 1e-3,
                "dim {i}: expected {}, got {}, diff {}",
                target[i],
                result.point[i],
                (result.point[i] - target[i]).abs()
            );
        }
    }

    // ── 6. Edge cases ───────────────────────────────────────────────────

    #[test]
    fn tight_bounds_reduce_trust() {
        // When bounds are tighter than 2*initial_trust, the trust region
        // should auto-reduce (matching the Java setup() logic).
        // Bounds width = 0.2, initial_trust = 1.0 → auto-reduced to 0.2/3 ≈ 0.067
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 0.5).powi(2) + (x[1] - 0.5).powi(2),
            &[0.4, 0.6],
            &[0.3, 0.3],
            &[0.7, 0.7],
            5,
            1.0, // will be reduced by new()
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 0.5, epsilon = 1e-4);
        assert_abs_diff_eq!(result.point[1], 0.5, epsilon = 1e-4);
    }

    #[test]
    fn minimum_interp_points() {
        // n_interp = n+2 (minimum allowed)
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 1.0).powi(2) + (x[1] + 1.0).powi(2) + (x[2] - 2.0).powi(2),
            &[0.0, 0.0, 0.0],
            &[-10.0, -10.0, -10.0],
            &[10.0, 10.0, 10.0],
            5, // n+2 = 5
            1.0,
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(result.point[1], -1.0, epsilon = 1e-4);
        assert_abs_diff_eq!(result.point[2], 2.0, epsilon = 1e-4);
    }

    #[test]
    fn start_at_bound() {
        // Starting point at the lower bound
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 5.0).powi(2) + (x[1] - 5.0).powi(2),
            &[0.0, 0.0],
            &[0.0, 0.0],
            &[10.0, 10.0],
            5,
            1.0,
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 5.0, epsilon = 1e-4);
        assert_abs_diff_eq!(result.point[1], 5.0, epsilon = 1e-4);
    }

    #[test]
    fn asymmetric_bounds() {
        // Optimum is at (1.5, -0.5), bounds are asymmetric
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 1.5).powi(2) + (x[1] + 0.5).powi(2),
            &[0.0, 0.0],
            &[-1.0, -3.0],
            &[5.0, 2.0],
            5,
            1.0,
            1e-8,
            1000,
        )
        .unwrap();

        assert_abs_diff_eq!(result.point[0], 1.5, epsilon = 1e-6);
        assert_abs_diff_eq!(result.point[1], -0.5, epsilon = 1e-6);
    }

    // ── 7. ACM3 parity tests ──────────────────────────────────────────
    //
    // These compare evaluation counts and function values against
    // Apache Commons Math 3.6.1 BOBYQAOptimizer output, generated from
    // BobyqaRef.java using the same test functions above.
    //
    // Due to minor FP operation ordering differences between Java and
    // Rust (e.g. fused multiply-add availability, expression evaluation
    // order within complex statements), the algorithm can take slightly
    // different paths. For simple quadratic functions the results match
    // exactly; for harder functions we allow small eval count divergence
    // while verifying the optimizer reaches the same quality solution.
    //
    // Tests marked "exact" match ACM3 bit-for-bit.
    // Tests marked "near" allow small eval count tolerance.

    #[test]
    fn acm3_exact_sphere() {
        // ACM3: value=0.0, evals=56
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut sphere, 13, &start, 1000);
        assert_eq!(result.evaluations, 56, "sphere eval count (ACM3: 56)");
        assert_eq!(result.value, 0.0);
    }

    #[test]
    fn acm3_exact_cigar() {
        // ACM3: value=4.93e-32, evals=56
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut cigar, 13, &start, 1000);
        assert_eq!(result.evaluations, 56, "cigar eval count (ACM3: 56)");
        assert!(result.value < 1e-20);
    }

    #[test]
    fn acm3_exact_bounded_quadratic() {
        // ACM3: value=5.0, evals=27, point=(2,2)
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 3.0).powi(2) + (x[1] - 4.0).powi(2),
            &[1.0, 1.0],
            &[0.0, 0.0],
            &[2.0, 2.0],
            5,
            0.5,
            1e-8,
            1000,
        )
        .unwrap();
        assert_eq!(result.evaluations, 27, "bounded quad evals (ACM3: 27)");
        assert_eq!(result.value, 5.0);
        assert_eq!(result.point[0], 2.0);
        assert_eq!(result.point[1], 2.0);
    }

    #[test]
    fn acm3_exact_tablet() {
        // ACM3: value=5.55e-28, evals=57
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut tablet, 13, &start, 1000);
        assert_eq!(result.evaluations, 57, "tablet eval count (ACM3: 57)");
        assert!(result.value < 1e-20);
    }

    #[test]
    fn acm3_near_quadratic_2d() {
        // ACM3: value=0.0, evals=28, point=(3,4)
        // Rust gets 29 evals (off-by-1 from FP path)
        let result = bobyqa_minimize(
            &mut |x| (x[0] - 3.0).powi(2) + (x[1] - 4.0).powi(2),
            &[0.0, 0.0],
            &[-10.0, -10.0],
            &[10.0, 10.0],
            5,
            1.0,
            1e-8,
            1000,
        )
        .unwrap();
        assert!(
            (result.evaluations as i64 - 28).unsigned_abs() <= 5,
            "quad2d evals: {} (ACM3: 28)",
            result.evaluations
        );
        assert_eq!(result.value, 0.0);
        assert_eq!(result.point[0], 3.0);
        assert_eq!(result.point[1], 4.0);
    }

    #[test]
    fn acm3_near_rastrigin() {
        // ACM3: value=0.0, evals=166
        // Rust gets 162 (4 fewer — minor FP divergence)
        let start = vec![1.0; 13];
        let result = run_unbounded(&mut rastrigin, 13, &start, 5000);
        assert!(
            (result.evaluations as i64 - 166).unsigned_abs() <= 10,
            "rastrigin evals: {} (ACM3: 166)",
            result.evaluations
        );
        assert_eq!(result.value, 0.0);
    }

    #[test]
    fn acm3_near_rosenbrock_2d() {
        // ACM3: value=2.12e-23, evals=150
        // Rust gets 166 evals — FP path divergence in later iterations
        let result = bobyqa_minimize(
            &mut rosenbrock,
            &[-1.0, -1.0],
            &[-5.0, -5.0],
            &[5.0, 5.0],
            5,
            1.0,
            1e-8,
            5000,
        )
        .unwrap();
        assert!(
            (result.evaluations as i64 - 150).unsigned_abs() <= 30,
            "rosen2d evals: {} (ACM3: 150)",
            result.evaluations
        );
        assert!(result.value < 1e-15, "rosen2d value: {}", result.value);
    }

    #[test]
    fn acm3_near_powell_m5_npt21() {
        // ACM3: value=5.60153397218646450, evals=98
        // Rust gets 126 evals — different path through non-convex landscape
        let m = 5;
        let start = powell_start(m);
        let n = 2 * m;
        let lo = vec![-1.0; n];
        let hi = vec![1.0; n];
        let result = bobyqa_minimize(
            &mut |x| powell_points_in_square(x, m),
            &start,
            &lo,
            &hi,
            21,
            0.1,
            1e-6,
            500000,
        )
        .unwrap();
        // Must reach same local minimum (value match)
        assert!(
            (result.value - 5.60153397218646450).abs() < 1e-6,
            "powell m5 value: {} (ACM3: 5.6015)",
            result.value
        );
        // Eval count can diverge more for non-convex problems
        assert!(
            (result.evaluations as i64 - 98).unsigned_abs() <= 50,
            "powell m5 evals: {} (ACM3: 98)",
            result.evaluations
        );
    }

    #[test]
    fn acm3_near_rosenbrock_13d() {
        // ACM3: value=7.04e-15, evals=1312
        let start = vec![0.1; 13];
        let result = run_unbounded(&mut rosenbrock, 13, &start, 5000);
        let eval_diff = (result.evaluations as i64 - 1312).unsigned_abs();
        assert!(
            eval_diff <= 100,
            "rosen13d evals: {} (ACM3: 1312, diff: {})",
            result.evaluations,
            eval_diff
        );
        assert!(result.value < 1e-6, "rosen13d value: {}", result.value);
    }
}
