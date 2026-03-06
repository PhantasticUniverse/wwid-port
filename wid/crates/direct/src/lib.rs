//! Pure Rust implementation of the DIRECT-C global optimizer.
//!
//! **DIRECT** (DIviding RECTangles) is a deterministic global optimization
//! algorithm for minimizing a black-box function over a hyper-rectangular
//! domain. It requires no derivatives and no knowledge of the function's
//! Lipschitz constant.
//!
//! This crate implements the **DIRECT-C** (DIRECT Centred) variant, which
//! extends the DIRECT-1 algorithm with alternate strategies for selecting
//! potentially optimal hyperrectangles (POH). When the standard convex hull
//! selection stagnates, DIRECT-C switches to strategies that search near the
//! current best point: picking large nearby rectangles, or picking rectangles
//! with low function values that are nearby.
//!
//! # Algorithm overview
//!
//! 1. Start with one rectangle covering the entire search space
//! 2. At each iteration, select "potentially optimal" rectangles (those on
//!    the lower-right convex hull of the (diameter, f-value) space)
//! 3. Divide each selected rectangle by trisecting along its longest side(s)
//! 4. Repeat until convergence or the budget is exhausted
//!
//! The convex hull selection automatically balances exploration (large
//! rectangles) and exploitation (low function values). The DIRECT-C variant
//! adds centred strategies that accelerate convergence on many problems.
//!
//! # When to use DIRECT
//!
//! - Global optimization of black-box functions with bound constraints
//! - Moderate dimensionality (2-20 variables)
//! - Unknown landscape where local optimizers would get stuck
//! - As a first stage before local refinement (e.g., DIRECT-C then BOBYQA)
//!
//! # Example
//!
//! ```
//! use direct::direct_minimize;
//!
//! // Minimize the six-hump camel function (2 global minima)
//! let result = direct_minimize(
//!     &mut |x| {
//!         let x1 = x[0]; let x2 = x[1];
//!         (4.0 - 2.1*x1*x1 + x1*x1*x1*x1/3.0) * x1*x1
//!             + x1*x2
//!             + (-4.0 + 4.0*x2*x2) * x2*x2
//!     },
//!     &[-3.0, -2.0],   // lower bounds
//!     &[3.0, 2.0],     // upper bounds
//!     1e-4,            // convergence threshold
//!     10000,           // max function calls
//!     None,            // no target value
//! ).expect("optimization should converge");
//!
//! assert!((result.value - (-1.0316)).abs() < 0.01);
//! ```
//!
//! # Evaluation budget
//!
//! The `max_evaluations` parameter is a soft budget: it is checked between
//! rectangle divisions within each iteration. A single rectangle division
//! (which evaluates 2 new points per eligible dimension) will run to
//! completion even if it crosses the budget. The maximum overshoot is
//! bounded by `2 * n_dimensions` function evaluations. The actual count
//! is always reported in [`DirectResult::evaluations`].
//!
//! # Implementation lineage
//!
//! This is a clean-room Rust implementation based on:
//!
//! - **Jones, Perttunen & Stuckman (1993)**: original DIRECT algorithm
//! - **Gablonsky & Kelley (2001)**: DIRECT-L (locally-biased) variant and
//!   single-side division strategy (DIRECT-1)
//! - **NLopt** (Steven G. Johnson, MIT license): reference C implementation
//!   for base algorithm structure
//! - **WIDesigner** (Burton Patkau, GPL v3): algorithm description for the
//!   DIRECT-C centred variant selection strategies
//!
//! The Rust code is original work, licensed Apache-2.0.
//!
//! # References
//!
//! 1. Jones, D.R., Perttunen, C.D. & Stuckman, B.E. "Lipschitzian optimization
//!    without the Lipschitz constant." *J. Optim. Theory Appl.* 79, 157-181 (1993).
//! 2. Gablonsky, J.M. & Kelley, C.T. "A locally-biased form of the DIRECT
//!    algorithm." *J. Global Optim.* 21, 27-37 (2001).
//! 3. Finkel, D.E. & Kelley, C.T. "Convergence analysis of the DIRECT
//!    algorithm." (2004).
//! 4. NLopt library, Steven G. Johnson. <https://github.com/stevengj/nlopt>

use std::collections::BTreeMap;

// ── Constants ────────────────────────────────────────────────────────

const THIRD: f64 = 1.0 / 3.0;

/// Tolerance for treating rectangle sides as equal length.
const EQUAL_SIDE_TOL: f64 = 5e-2;

/// Granularity for diameter-based key lookups.
const DIAMETER_GRANULARITY: f64 = 1.0e-13;

/// Default iterations without improvement before declaring convergence
/// (when x has already converged).
const DEFAULT_CONVERGED_ITERATIONS: usize = 20;

/// Default iteration interval for DIRECT-C variant selection.
const DEFAULT_ITERATION_INTERVAL: usize = 3;

/// Number of distance bins for "low value & near" variant.
const DEFAULT_DISTANCE_BINS: usize = 50;

// ── Public types ─────────────────────────────────────────────────────

/// Result of a DIRECT-C global optimization.
#[derive(Debug, Clone)]
pub struct DirectResult {
    /// The point at which the minimum was found.
    pub point: Vec<f64>,
    /// The function value at that point.
    pub value: f64,
    /// Total number of function calls performed.
    pub evaluations: usize,
}

/// Progress information during DIRECT-C optimization.
#[derive(Debug, Clone)]
pub struct DirectProgress {
    /// Total function calls so far.
    pub evaluations: usize,
    /// Current best function value.
    pub best_value: f64,
    /// Number of active hyperrectangles.
    pub num_rectangles: usize,
}

// ── Public API ───────────────────────────────────────────────────────

/// Minimize `f` over `[lower_bounds, upper_bounds]` using the DIRECT-C algorithm.
///
/// Returns `None` if the problem has fewer than 1 dimension or bounds are invalid.
///
/// # Parameters
///
/// - `f`: objective function to minimize
/// - `lower_bounds`, `upper_bounds`: box constraints
/// - `convergence_threshold`: converge when best rectangle has all sides
///   smaller than this fraction of the bound range
/// - `max_evaluations`: budget for function calls. The budget is checked
///   between rectangle divisions within each iteration. A single rectangle
///   division may overshoot by up to `2 * n_dimensions` evaluations, but
///   no new rectangles will be selected for division once the budget is
///   reached. The actual evaluation count is reported in
///   [`DirectResult::evaluations`].
/// - `target_value`: optional early stop when f <= target
pub fn direct_minimize(
    f: &mut dyn FnMut(&[f64]) -> f64,
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    convergence_threshold: f64,
    max_evaluations: usize,
    target_value: Option<f64>,
) -> Option<DirectResult> {
    direct_minimize_with_callback(
        f,
        lower_bounds,
        upper_bounds,
        convergence_threshold,
        max_evaluations,
        target_value,
        &mut |_| true,
    )
}

/// Like [`direct_minimize`], but with a progress callback.
///
/// The callback receives [`DirectProgress`] after each iteration
/// and returns `true` to continue or `false` to cancel.
pub fn direct_minimize_with_callback(
    f: &mut dyn FnMut(&[f64]) -> f64,
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    convergence_threshold: f64,
    max_evaluations: usize,
    target_value: Option<f64>,
    on_progress: &mut dyn FnMut(DirectProgress) -> bool,
) -> Option<DirectResult> {
    let n = lower_bounds.len();
    if n < 1 || upper_bounds.len() != n {
        return None;
    }
    // Validate bounds
    for i in 0..n {
        if lower_bounds[i] > upper_bounds[i] {
            return None;
        }
    }

    let mut optimizer = DirectOptimizer::new(
        n,
        lower_bounds,
        upper_bounds,
        convergence_threshold,
        max_evaluations,
        target_value,
    );

    optimizer.run(f, on_progress)
}

// ── Internal data structures ─────────────────────────────────────────

/// Key for rectangle storage in BTreeMap.
///
/// Sorted by (diameter, f_value, serial) for efficient convex hull queries.
/// This ordering groups rectangles of the same size together and sorts them
/// by function value, which is exactly what the POH selection algorithm needs.
#[derive(Clone)]
struct RectKey {
    diameter: f64,
    f_value: f64,
    serial: u32,
}

impl RectKey {
    fn new(diameter: f64, f_value: f64, serial: &mut u32) -> Self {
        *serial += 1;
        RectKey {
            diameter,
            f_value,
            serial: *serial,
        }
    }

    /// Create a search key (for BTreeMap range queries).
    fn search(diameter: f64) -> Self {
        RectKey {
            diameter,
            f_value: f64::NEG_INFINITY,
            serial: 0,
        }
    }
}

impl PartialEq for RectKey {
    fn eq(&self, other: &Self) -> bool {
        self.serial == other.serial
            && self.diameter == other.diameter
            && self.f_value == other.f_value
    }
}

impl Eq for RectKey {}

impl PartialOrd for RectKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RectKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Compare by diameter, then f_value, then serial
        match self.diameter.partial_cmp(&other.diameter) {
            Some(std::cmp::Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        match self.f_value.partial_cmp(&other.f_value) {
            Some(std::cmp::Ordering::Equal) | None => {}
            Some(ord) => return ord,
        }
        self.serial.cmp(&other.serial)
    }
}

/// State of a hyperrectangle in the search space.
struct RectValue {
    /// Coordinates of centre point (absolute).
    centre: Vec<f64>,
    /// Width per dimension (relative to bound range, 0..1).
    width: Vec<f64>,
    /// Per-dimension improvement potential (for DIRECT-1 side selection).
    potential: Option<Vec<f64>>,
    /// Length of longest side.
    max_width: f64,
    /// Number of sides near max_width.
    long_count: usize,
    /// Index of first longest side.
    long_idx: usize,
}

impl RectValue {
    fn new(centre: Vec<f64>, width: Vec<f64>) -> Self {
        let mut rv = RectValue {
            centre,
            width,
            potential: None,
            max_width: 0.0,
            long_count: 0,
            long_idx: 0,
        };
        rv.update_long_sides();
        rv
    }

    fn new_with_potential(centre: Vec<f64>, width: Vec<f64>, potential: Vec<f64>) -> Self {
        let mut rv = RectValue {
            centre,
            width,
            potential: Some(potential),
            max_width: 0.0,
            long_count: 0,
            long_idx: 0,
        };
        rv.update_long_sides();
        rv
    }

    fn update_long_sides(&mut self) {
        self.max_width = self.width[0];
        self.long_idx = 0;
        for i in 1..self.width.len() {
            if self.width[i] > self.max_width {
                self.max_width = self.width[i];
                self.long_idx = i;
            }
        }
        self.long_count = 0;
        for i in 0..self.width.len() {
            if self.width[i] >= self.max_width * (1.0 - EQUAL_SIDE_TOL) {
                self.long_count += 1;
            }
        }
    }

    fn is_long_side(&self, i: usize) -> bool {
        if i == self.long_idx {
            return true;
        }
        self.width[i] >= self.max_width * (1.0 - EQUAL_SIDE_TOL)
    }

    /// All sides smaller than convergence threshold.
    fn is_small(&self, threshold: f64) -> bool {
        self.width.iter().all(|&w| w <= threshold)
    }

    fn is_hypercube(&self, bound_diff: &[f64]) -> bool {
        for i in 0..self.width.len() {
            if bound_diff[i] > 0.0 && !self.is_long_side(i) {
                return false;
            }
        }
        true
    }
}

/// A reference to a rectangle (key + centre copy for distance calculations).
struct HullRect {
    key: RectKey,
    centre: Vec<f64>,
}

// ── Main optimizer state ─────────────────────────────────────────────

struct DirectOptimizer {
    n: usize,
    lower_bounds: Vec<f64>,
    upper_bounds: Vec<f64>,
    bound_diff: Vec<f64>,
    convergence_threshold: f64,
    converged_iterations_threshold: usize,
    max_evaluations: usize,
    target_value: Option<f64>,

    // Rectangle tree
    rtree: BTreeMap<RectKey, RectValue>,
    next_serial: u32,

    // Current best
    best_point: Vec<f64>,
    best_value: f64,
    f_max: f64,
    num_evals: usize,
    iterations: usize,
    iteration_of_last_improvement: usize,

    // Convergence tracking
    is_x_converged: bool,

    // DIRECT-C variant selection
    iteration_interval_no_improvement: usize,
    iteration_interval_max: usize,
    iteration_of_last_variant: usize,
    next_variant_index: usize,
    type_of_variant: Vec<usize>,
    relative_distance: Vec<f64>,
}

impl DirectOptimizer {
    fn new(
        n: usize,
        lower_bounds: &[f64],
        upper_bounds: &[f64],
        convergence_threshold: f64,
        max_evaluations: usize,
        target_value: Option<f64>,
    ) -> Self {
        let mut bound_diff = vec![0.0; n];
        for i in 0..n {
            bound_diff[i] = upper_bounds[i] - lower_bounds[i];
        }

        // DIRECT-C distance bins: relativeDistance[i] = 1.7^(i - 50)
        let mut relative_distance = vec![0.0; DEFAULT_DISTANCE_BINS];
        for i in 0..DEFAULT_DISTANCE_BINS {
            relative_distance[i] = 1.7_f64.powi(i as i32 - DEFAULT_DISTANCE_BINS as i32);
        }

        DirectOptimizer {
            n,
            lower_bounds: lower_bounds.to_vec(),
            upper_bounds: upper_bounds.to_vec(),
            bound_diff,
            convergence_threshold,
            converged_iterations_threshold: DEFAULT_CONVERGED_ITERATIONS,
            max_evaluations,
            target_value,

            rtree: BTreeMap::new(),
            next_serial: 0,

            best_point: vec![0.0; n],
            best_value: f64::MAX,
            f_max: 1.0,
            num_evals: 0,
            iterations: 0,
            iteration_of_last_improvement: 0,

            is_x_converged: false,

            iteration_interval_no_improvement: DEFAULT_ITERATION_INTERVAL,
            iteration_interval_max: 4 * DEFAULT_ITERATION_INTERVAL,
            iteration_of_last_variant: 0,
            next_variant_index: 0,
            type_of_variant: vec![2, 1, 1, 2, 1],
            relative_distance,
        }
    }

    /// Call the objective function and track best/worst values.
    fn call_objective(&mut self, f: &mut dyn FnMut(&[f64]) -> f64, point: &[f64]) -> f64 {
        self.num_evals += 1;
        let fval = f(point);
        if fval < self.best_value {
            self.best_value = fval;
            self.best_point = point.to_vec();
            self.iteration_of_last_improvement = self.iterations;
        }
        if fval > self.f_max {
            self.f_max = fval;
        }
        fval
    }

    /// Compute the "diameter" of a rectangle (distance from centre to vertex).
    ///
    /// Rounded to single precision to group rectangles with the same
    /// dimension pattern together (matching NLopt/Java behavior).
    fn rectangle_diameter(&self, w: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..w.len() {
            if self.bound_diff[i] > 0.0 {
                sum += w[i] * w[i];
            }
        }
        (sum.sqrt() * 0.5) as f32 as f64
    }

    /// Threshold diameter for convergence.
    fn threshold_diameter(&self) -> f64 {
        if self.convergence_threshold <= 0.0 {
            return 0.0;
        }
        // Round threshold down to next smaller power of 1/3
        let a_iterations = (self.convergence_threshold.ln() / THIRD.ln()).ceil();
        let threshold = THIRD.powf(a_iterations);
        0.5 * (self.n as f64).sqrt() * threshold
    }

    // ── Setup ────────────────────────────────────────────────────────

    fn setup(&mut self, f: &mut dyn FnMut(&[f64]) -> f64) {
        let mut centre = vec![0.0; self.n];
        let mut width = vec![0.0; self.n];

        for i in 0..self.n {
            centre[i] = 0.5 * (self.upper_bounds[i] + self.lower_bounds[i]);
            if self.bound_diff[i] > 0.0 {
                width[i] = 1.0;
            }
        }

        // Call objective at centre
        let f_centre = self.call_objective(f, &centre);
        self.f_max = f_centre;

        let key = RectKey::new(
            self.rectangle_diameter(&width),
            f_centre,
            &mut self.next_serial,
        );
        let rect = RectValue::new(centre, width);
        self.rtree.insert(key.clone(), rect);

        // Divide the initial rectangle
        self.divide_rectangle(f, &key);
    }

    // ── Main loop ────────────────────────────────────────────────────

    fn run(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        on_progress: &mut dyn FnMut(DirectProgress) -> bool,
    ) -> Option<DirectResult> {
        self.setup(f);
        self.iteration_of_last_variant = 0;
        self.next_variant_index = 0;

        let convergence_diameter = self.threshold_diameter();

        loop {
            self.iterations += 1;
            let nr_promising = self.divide_potentially_optimal(f, convergence_diameter);

            // Progress callback
            let keep_going = on_progress(DirectProgress {
                evaluations: self.num_evals,
                best_value: self.best_value,
                num_rectangles: self.rtree.len(),
            });
            if !keep_going {
                break;
            }

            if self.has_converged(nr_promising) {
                break;
            }

            if self.num_evals >= self.max_evaluations {
                break;
            }
        }

        Some(DirectResult {
            point: self.best_point.clone(),
            value: self.best_value,
            evaluations: self.num_evals,
        })
    }

    // ── Convergence check ────────────────────────────────────────────

    fn has_converged(&self, nr_promising: usize) -> bool {
        // Target value reached?
        if let Some(target) = self.target_value {
            if self.best_value <= target {
                return true;
            }
        }

        // X not converged yet?
        if !self.is_x_converged {
            return false;
        }

        // No promising divisions and enough iterations since last improvement?
        if nr_promising == 0
            && self.iterations >= self.iteration_of_last_improvement + 1 + self.n
        {
            return true;
        }

        // Stagnated for converged_iterations_threshold?
        if self.iterations >= self.iteration_of_last_improvement + self.converged_iterations_threshold {
            return true;
        }

        false
    }

    // ── POH selection + division ──────────────────────────────────────

    fn divide_potentially_optimal(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        convergence_diameter: f64,
    ) -> usize {
        self.is_x_converged = false;
        let mut nr_promising_divisions = 0;

        // Select POH (potentially with DIRECT-C variant)
        let hull = self.get_potentially_optimal();

        for hr in &hull {
            // Stop dividing if we've exhausted the evaluation budget
            if self.num_evals >= self.max_evaluations {
                break;
            }
            // Look up the rectangle in the tree
            if let Some(rect) = self.rtree.get(&hr.key) {
                if hr.key.diameter < convergence_diameter && rect.is_small(self.convergence_threshold) {
                    self.is_x_converged = true;
                } else {
                    nr_promising_divisions += self.divide_rectangle(f, &hr.key);
                }
            }
        }

        nr_promising_divisions
    }

    // ── DIRECT-C variant selection ───────────────────────────────────

    fn select_variant(&mut self) -> usize {
        if self.iteration_of_last_variant + self.iteration_interval_no_improvement
            > self.iterations
        {
            // Too soon to use a variant
            return 0;
        }
        if self.iteration_of_last_improvement + self.iteration_interval_no_improvement
            <= self.iterations
            || self.iteration_of_last_variant + self.iteration_interval_max <= self.iterations
        {
            // No improvement or no variant for too long
            self.iteration_of_last_variant = self.iterations;
            if self.next_variant_index >= self.type_of_variant.len() {
                self.next_variant_index = 0;
            }
            let variant = self.type_of_variant[self.next_variant_index];
            self.next_variant_index += 1;
            return variant;
        }
        0
    }

    fn get_potentially_optimal(&mut self) -> Vec<HullRect> {
        let variant = self.select_variant();
        match variant {
            1 => self.get_poh_large_and_near(),
            2 => self.get_poh_near_by_value(),
            _ => self.get_poh_standard(),
        }
    }

    // ── Standard POH (convex hull on diameter vs f-value) ────────────

    fn get_poh_standard(&self) -> Vec<HullRect> {
        let mut hull: Vec<HullRect> = Vec::new();

        if self.rtree.is_empty() {
            return hull;
        }

        // Get xmax (largest diameter)
        let last = self.rtree.iter().next_back().unwrap();
        let xmax = last.0.diameter;

        // Find first entry at xmax
        let search_key = RectKey::search(xmax * (1.0 - DIAMETER_GRANULARITY));
        let nmax_entry = self.rtree.range(search_key..).next();
        let Some((nmax_key, _)) = nmax_entry else {
            return hull;
        };
        assert_eq!(nmax_key.diameter, xmax);
        let ymaxmin = nmax_key.f_value;

        let mut xlast = 0.0_f64;
        let mut ylast = self.best_value;
        let mut minslope = if xmax > xlast {
            (ymaxmin - ylast) / (xmax - xlast)
        } else {
            0.0
        };

        // Iterate through all rectangles except those at xmax
        for (k, v) in self.rtree.iter() {
            if k.diameter >= xmax {
                break;
            }

            // Performance hack: skip points at same x with higher y
            if !hull.is_empty() && k.diameter == xlast {
                if k.f_value > ylast {
                    continue;
                }
                // Equal y values: allow duplicates (Jones mode)
                hull.push(HullRect {
                    key: k.clone(),
                    centre: v.centre.clone(),
                });
                continue;
            }

            // Skip points above the line from last point to nmax
            if !hull.is_empty()
                && k.f_value > ylast + (k.diameter - xlast) * minslope
            {
                continue;
            }

            // Prune hull: remove points until we're making a "left turn"
            while !hull.is_empty() {
                let nhull = hull.len();
                let t1 = &hull[nhull - 1].key;

                // Find t2 (previous point with different (diameter, f_value))
                let it2 = Self::find_prune_point(&hull, nhull, t1);

                if it2.is_none() {
                    // First segment: keep if positive slope
                    if t1.f_value < k.f_value {
                        break;
                    }
                } else {
                    let it2_idx = it2.unwrap();
                    let t2 = &hull[it2_idx].key;
                    // Cross product (t1-t2) x (k-t2) >= 0 means left turn
                    let cross = (t1.diameter - t2.diameter) * (k.f_value - t2.f_value)
                        - (t1.f_value - t2.f_value) * (k.diameter - t2.diameter);
                    if cross >= 0.0 {
                        break;
                    }
                }
                // Prune
                let keep = it2.map_or(0, |i| i + 1);
                hull.truncate(keep);
            }

            hull.push(HullRect {
                key: k.clone(),
                centre: v.centre.clone(),
            });
            xlast = k.diameter;
            ylast = k.f_value;
            if xmax > xlast {
                minslope = (ymaxmin - ylast) / (xmax - xlast);
            }
        }

        // Add all points at xmax with ymaxmin (Jones: allow duplicates)
        let search_key = RectKey::search(xmax * (1.0 - DIAMETER_GRANULARITY));
        for (k, v) in self.rtree.range(search_key..) {
            if k.diameter != xmax {
                break;
            }
            if k.f_value != ymaxmin {
                break;
            }
            hull.push(HullRect {
                key: k.clone(),
                centre: v.centre.clone(),
            });
        }

        hull
    }

    /// Find the previous point in the hull with different (diameter, f_value).
    fn find_prune_point(hull: &[HullRect], nhull: usize, t1: &RectKey) -> Option<usize> {
        let mut it2 = nhull.wrapping_sub(2);
        while it2 < nhull {
            // wrapping handles underflow
            let t2 = &hull[it2].key;
            if t2.diameter != t1.diameter || t2.f_value != t1.f_value {
                return Some(it2);
            }
            it2 = it2.wrapping_sub(1);
        }
        None
    }

    // ── DIRECT-C Variant 1: Large and Near ───────────────────────────

    fn get_poh_large_and_near(&self) -> Vec<HullRect> {
        let target = &self.best_point;
        let mut hull: Vec<HullRect> = Vec::new();

        let mut nearest: Option<(&RectKey, &RectValue)> = None;
        let mut n_dist = 0.0_f64;

        for (k, v) in self.rtree.iter() {
            if v.is_small(self.convergence_threshold) {
                // Skip rectangles too small to divide further
                continue;
            }

            if nearest.is_none() {
                nearest = Some((k, v));
                n_dist = self.distance(&v.centre, target);
                continue;
            }

            let (near_key, _) = nearest.unwrap();
            if k.diameter == near_key.diameter {
                // Same diameter: check if nearer to best
                let y = self.distance(&v.centre, target);
                if y < n_dist {
                    nearest = Some((k, v));
                    n_dist = y;
                }
                continue;
            }

            // New diameter level: add nearest to hull with pruning
            let (near_key, near_val) = nearest.unwrap();
            Self::prune_hull_large_and_near(&mut hull, near_key, n_dist, target, self);
            hull.push(HullRect {
                key: near_key.clone(),
                centre: near_val.centre.clone(),
            });

            nearest = Some((k, v));
            n_dist = self.distance(&v.centre, target);
        }

        // Add last diameter level
        if let Some((near_key, near_val)) = nearest {
            Self::prune_hull_large_and_near(&mut hull, near_key, n_dist, target, self);
            hull.push(HullRect {
                key: near_key.clone(),
                centre: near_val.centre.clone(),
            });
        }

        hull
    }

    /// Prune the Large & Near hull to convex hull on (diameter, distance).
    fn prune_hull_large_and_near(
        hull: &mut Vec<HullRect>,
        n_key: &RectKey,
        n_distance: f64,
        target: &[f64],
        opt: &DirectOptimizer,
    ) {
        while !hull.is_empty() {
            let nhull = hull.len();
            let t1 = &hull[nhull - 1];
            let t1_dist = opt.distance(&t1.centre, target);

            if t1_dist > n_distance {
                // Not even monotone — keep pruning
            } else if nhull < 2 {
                // First segment with positive slope — stop
                break;
            } else {
                // Convex hull check
                let t2 = &hull[nhull - 2];
                let t2_dist = opt.distance(&t2.centre, target);
                let cross = (t1.key.diameter - t2.key.diameter) * (n_distance - t2_dist)
                    - (t1_dist - t2_dist) * (n_key.diameter - t2.key.diameter);
                if cross >= 0.0 {
                    break;
                }
            }
            hull.truncate(nhull - 1);
        }
    }

    // ── DIRECT-C Variant 2: Low Value and Near ───────────────────────

    fn get_poh_near_by_value(&self) -> Vec<HullRect> {
        let target = &self.best_point;
        let max_dist = self.max_distance(target);

        // Bin sort: for each distance bin, keep the rectangle with lowest f-value
        let mut lowest: Vec<Option<HullRect>> = (0..DEFAULT_DISTANCE_BINS).map(|_| None).collect();

        for (k, v) in self.rtree.iter() {
            if v.is_small(self.convergence_threshold) {
                continue;
            }
            let bin = self.get_distance_bin(&v.centre, target, max_dist);
            let is_better = match &lowest[bin] {
                None => true,
                Some(existing) => k.f_value < existing.key.f_value,
            };
            if is_better {
                lowest[bin] = Some(HullRect {
                    key: k.clone(),
                    centre: v.centre.clone(),
                });
            }
        }

        // Build monotone hull on (distance, f_value) from the bin-sorted results
        let mut hull: Vec<HullRect> = Vec::new();
        for entry in lowest.into_iter().flatten() {
            let n_value = entry.key.f_value;

            // Prune hull to monotone (not necessarily convex)
            while !hull.is_empty() {
                let nhull = hull.len();
                let t1_value = hull[nhull - 1].key.f_value;

                if t1_value > n_value {
                    // Not monotone — prune
                } else if nhull < 2 {
                    // First segment with positive slope — stop
                    break;
                } else {
                    // Monotone hull is sufficient — stop
                    break;
                }
                hull.truncate(nhull - 1);
            }
            hull.push(entry);
        }

        hull
    }

    // ── Distance calculations ────────────────────────────────────────

    /// Normalized Cartesian distance between two points.
    fn distance(&self, x1: &[f64], x2: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..x1.len() {
            if self.bound_diff[i] > 0.0 {
                let side = (x1[i] - x2[i]) / self.bound_diff[i];
                sum += side * side;
            }
        }
        sum.sqrt()
    }

    /// Maximum possible distance from a point to any corner of the bounds.
    fn max_distance(&self, x: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..x.len() {
            if self.bound_diff[i] > 0.0 {
                let lower_side = x[i] - self.lower_bounds[i];
                let upper_side = self.upper_bounds[i] - x[i];
                let side = if lower_side >= upper_side {
                    lower_side / self.bound_diff[i]
                } else {
                    upper_side / self.bound_diff[i]
                };
                sum += side * side;
            }
        }
        sum.sqrt()
    }

    /// Determine which distance bin a point falls into.
    fn get_distance_bin(&self, x: &[f64], target: &[f64], max_dist: f64) -> usize {
        if max_dist <= 0.0 {
            return DEFAULT_DISTANCE_BINS - 1;
        }
        let dist = self.distance(x, target) / max_dist;
        for i in 0..self.relative_distance.len() {
            if dist <= self.relative_distance[i] {
                return i;
            }
        }
        self.relative_distance.len() - 1
    }

    // ── DIRECT-1: Eligible sides selection ───────────────────────────

    /// Select which sides of a rectangle to divide.
    ///
    /// DIRECT-1 strategy: for hypercubes, divide all long sides (original
    /// DIRECT behavior). For non-hypercubes, divide only the single long
    /// side with highest potential improvement.
    fn select_eligible_sides(&self, rect: &RectValue) -> (Vec<bool>, usize) {
        let n = rect.width.len();
        let mut eligible = vec![false; n];

        if rect.long_count == 1 {
            // Only one long side — must divide it
            eligible[rect.long_idx] = true;
            return (eligible, 1);
        }

        if rect.is_hypercube(&self.bound_diff) {
            // Hypercube: divide all long sides (original DIRECT behavior)
            for i in 0..n {
                eligible[i] = rect.is_long_side(i);
            }
            return (eligible, rect.long_count);
        }

        // DIRECT-1: non-hypercube, divide only the one long side with
        // highest potential
        let mut highest_potential = f64::NEG_INFINITY;
        let mut best_side = rect.long_idx;
        let potential = match &rect.potential {
            Some(p) => p.clone(),
            None => vec![0.0; n],
        };

        for i in 0..n {
            if rect.is_long_side(i) && potential[i] > highest_potential {
                highest_potential = potential[i];
                best_side = i;
            }
        }

        eligible[best_side] = true;
        (eligible, 1)
    }

    // ── Rectangle division ───────────────────────────────────────────

    /// Divide a rectangle into thirds along eligible sides.
    ///
    /// Returns the number of promising new points (where extrapolation
    /// suggests a better minimum might exist).
    fn divide_rectangle(
        &mut self,
        f: &mut dyn FnMut(&[f64]) -> f64,
        rect_key: &RectKey,
    ) -> usize {
        // Remove the rectangle from the tree for modification.
        let Some(mut rectangle) = self.rtree.remove(rect_key) else {
            return 0;
        };

        let n = rectangle.width.len();
        let centre_f = rect_key.f_value;
        let mut nr_promising = 0;

        let (eligible, nr_eligible) = self.select_eligible_sides(&rectangle);

        if nr_eligible > 1 {
            // Multi-side division: assess all eligible sides, then trisect
            // in order of minimum new function value
            let mut side_fv = vec![(f64::MAX, f64::MAX); n]; // (below, above)

            for i in 0..n {
                if eligible[i] {
                    let csave = rectangle.centre[i];

                    // Point below centre
                    rectangle.centre[i] = csave - rectangle.width[i] * THIRD * self.bound_diff[i];
                    let f_below = self.call_objective(f, &rectangle.centre);
                    if self.is_promising(centre_f, f_below) {
                        nr_promising += 1;
                    }

                    // Point above centre
                    rectangle.centre[i] = csave + rectangle.width[i] * THIRD * self.bound_diff[i];
                    let f_above = self.call_objective(f, &rectangle.centre);
                    if self.is_promising(centre_f, f_above) {
                        nr_promising += 1;
                    }

                    rectangle.centre[i] = csave;
                    side_fv[i] = (f_below, f_above);
                }
            }

            // Sort eligible dimensions by minimum new function value
            let mut isort: Vec<usize> = (0..n).collect();
            isort.sort_by(|&a, &b| {
                let fv_a = side_fv[a].0.min(side_fv[a].1);
                let fv_b = side_fv[b].0.min(side_fv[b].1);
                fv_a.partial_cmp(&fv_b).unwrap_or(std::cmp::Ordering::Equal)
            });

            // Trisect in sorted order
            let mut current_key = rect_key.clone();
            let base_potential = rectangle.potential.clone();
            let mut sides_done = 0;
            for &dim in &isort {
                if !eligible[dim] {
                    continue;
                }
                let (f_below, f_above) = side_fv[dim];

                // Shrink centre rectangle along this dimension
                rectangle.width[dim] *= THIRD;
                if sides_done > 0 {
                    self.rtree.remove(&current_key);
                }
                rectangle.update_long_sides();
                current_key = RectKey::new(
                    self.rectangle_diameter(&rectangle.width),
                    current_key.f_value,
                    &mut self.next_serial,
                );

                // Create child below centre
                let mut new_c = rectangle.centre.clone();
                let new_w = rectangle.width.clone();
                new_c[dim] = rectangle.centre[dim]
                    - rectangle.width[dim] * self.bound_diff[dim];
                let child_key = RectKey::new(
                    current_key.diameter,
                    f_below,
                    &mut self.next_serial,
                );
                let potential = Self::calculate_potential(
                    &base_potential, dim, f_below, centre_f, n,
                );
                let child = RectValue::new_with_potential(new_c, new_w, potential);
                self.rtree.insert(child_key, child);

                // Create child above centre
                let mut new_c = rectangle.centre.clone();
                let new_w = rectangle.width.clone();
                new_c[dim] = rectangle.centre[dim]
                    + rectangle.width[dim] * self.bound_diff[dim];
                let child_key = RectKey::new(
                    current_key.diameter,
                    f_above,
                    &mut self.next_serial,
                );
                let potential = Self::calculate_potential(
                    &base_potential, dim, f_above, centre_f, n,
                );
                let child = RectValue::new_with_potential(new_c, new_w, potential);
                self.rtree.insert(child_key, child);

                // Update centre rect's potential for this dimension.
                // Sign: neighbour_f - this_f = min_neighbor - centre_f
                // (matching Java DIRECT1Optimizer.calculatePotential)
                let min_neighbor = f_below.min(f_above);
                if let Some(ref mut p) = rectangle.potential {
                    let new_pot = min_neighbor - centre_f;
                    if new_pot >= p[dim] {
                        p[dim] = new_pot;
                    } else {
                        p[dim] = 0.5 * (new_pot + p[dim]);
                    }
                } else {
                    let mut p = match &base_potential {
                        Some(bp) => bp.clone(),
                        None => vec![0.0; n],
                    };
                    p[dim] = min_neighbor - centre_f;
                    rectangle.potential = Some(p);
                }

                sides_done += 1;
            }

            // Re-insert updated centre rectangle
            self.rtree.insert(current_key, rectangle);
        } else {
            // Single-side division (DIRECT-1 for non-hypercubes)
            let dim = eligible.iter().position(|&e| e).unwrap_or(rectangle.long_idx);
            let base_potential = rectangle.potential.clone();

            // Shrink centre rectangle
            rectangle.width[dim] *= THIRD;
            rectangle.update_long_sides();
            let new_diameter = self.rectangle_diameter(&rectangle.width);
            let new_centre_key = RectKey::new(
                new_diameter,
                rect_key.f_value,
                &mut self.next_serial,
            );

            // Child below centre
            let mut new_c = rectangle.centre.clone();
            let new_w = rectangle.width.clone();
            new_c[dim] = rectangle.centre[dim]
                - rectangle.width[dim] * self.bound_diff[dim];
            let f_below = self.call_objective(f, &new_c);
            let child_key = RectKey::new(new_diameter, f_below, &mut self.next_serial);
            let potential = Self::calculate_potential(
                &base_potential, dim, f_below, centre_f, n,
            );
            let child = RectValue::new_with_potential(new_c, new_w, potential);
            self.rtree.insert(child_key, child);
            if self.is_promising(centre_f, f_below) {
                nr_promising += 1;
            }

            // Child above centre
            let mut new_c = rectangle.centre.clone();
            let new_w = rectangle.width.clone();
            new_c[dim] = rectangle.centre[dim]
                + rectangle.width[dim] * self.bound_diff[dim];
            let f_above = self.call_objective(f, &new_c);
            let child_key = RectKey::new(new_diameter, f_above, &mut self.next_serial);
            let potential = Self::calculate_potential(
                &base_potential, dim, f_above, centre_f, n,
            );
            let child = RectValue::new_with_potential(new_c, new_w, potential);
            self.rtree.insert(child_key, child);
            if self.is_promising(centre_f, f_above) {
                nr_promising += 1;
            }

            // Update centre rect's potential.
            // Sign: neighbour_f - this_f = min_neighbor - centre_f
            let min_neighbor = f_below.min(f_above);
            if let Some(ref mut p) = rectangle.potential {
                let new_pot = min_neighbor - centre_f;
                if new_pot >= p[dim] {
                    p[dim] = new_pot;
                } else {
                    p[dim] = 0.5 * (new_pot + p[dim]);
                }
            } else {
                let mut p = match &base_potential {
                    Some(bp) => bp.clone(),
                    None => vec![0.0; n],
                };
                p[dim] = min_neighbor - centre_f;
                rectangle.potential = Some(p);
            }

            // Re-insert centre rectangle
            self.rtree.insert(new_centre_key, rectangle);
        }

        nr_promising
    }

    /// Check if a new function value suggests a better minimum exists.
    ///
    /// Extrapolates the line from centre through the new point to the edge
    /// of the original rectangle. Returns true if the extrapolation would
    /// give a value below the current best.
    fn is_promising(&self, centre_f: f64, new_f: f64) -> bool {
        if new_f < centre_f && centre_f - 1.5 * (centre_f - new_f) < self.best_value {
            return true;
        }
        if new_f > centre_f && centre_f - 0.1 * (new_f - centre_f) < self.best_value {
            return true;
        }
        false
    }

    /// Calculate per-dimension improvement potential for DIRECT-1.
    ///
    /// When potential increases, update immediately. When it decreases,
    /// decay half way (smooth tracking).
    fn calculate_potential(
        base: &Option<Vec<f64>>,
        dim: usize,
        this_f: f64,
        neighbour_f: f64,
        n: usize,
    ) -> Vec<f64> {
        let new_potential = neighbour_f - this_f;
        let mut potential = match base {
            Some(bp) => bp.clone(),
            None => vec![0.0; n],
        };
        if new_potential >= potential[dim] {
            potential[dim] = new_potential;
        } else {
            potential[dim] = 0.5 * (new_potential + potential[dim]);
        }
        potential
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Standard test functions ──────────────────────────────────────

    /// Sphere function: f(x) = sum(x_i^2), minimum at origin.
    fn sphere(x: &[f64]) -> f64 {
        x.iter().map(|xi| xi * xi).sum()
    }

    /// Rosenbrock 2D: f(x) = 100*(x2 - x1^2)^2 + (x1 - 1)^2
    /// Minimum: f(1, 1) = 0
    fn rosenbrock_2d(x: &[f64]) -> f64 {
        let t = x[0] * x[0] - x[1];
        100.0 * t * t + (x[0] - 1.0) * (x[0] - 1.0)
    }

    /// Rosenbrock N-D
    fn rosenbrock_nd(x: &[f64]) -> f64 {
        let mut sum = 0.0;
        for i in 0..x.len() - 1 {
            let t = x[i] * x[i] - x[i + 1];
            sum += 100.0 * t * t + (x[i] - 1.0) * (x[i] - 1.0);
        }
        sum
    }

    /// Six-hump camel function (standard test from NLopt tstc.c).
    /// Global minimum: f ~ -1.0316 at approximately (0.0898, -0.7126)
    /// or (-0.0898, 0.7126).
    fn six_hump_camel(x: &[f64]) -> f64 {
        let x1 = x[0];
        let x2 = x[1];
        (4.0 - 2.1 * x1 * x1 + x1 * x1 * x1 * x1 / 3.0) * x1 * x1
            + x1 * x2
            + (-4.0 + 4.0 * x2 * x2) * x2 * x2
    }

    /// Rastrigin 2D: highly multimodal.
    /// f(x) = 20 + sum(x_i^2 - 10*cos(2*pi*x_i))
    /// Global minimum: f(0, 0) = 0
    fn rastrigin_2d(x: &[f64]) -> f64 {
        let n = x.len() as f64;
        let mut sum = 10.0 * n;
        for &xi in x {
            sum += xi * xi - 10.0 * (2.0 * std::f64::consts::PI * xi).cos();
        }
        sum
    }

    /// Styblinski-Tang function.
    /// f(x) = 0.5 * sum(x_i^4 - 16*x_i^2 + 5*x_i)
    /// Global minimum: f(-2.9035, ...) ~ -39.16599 * n
    fn styblinski_tang(x: &[f64]) -> f64 {
        0.5 * x.iter().map(|&xi| {
            xi.powi(4) - 16.0 * xi * xi + 5.0 * xi
        }).sum::<f64>()
    }

    // ── Basic convergence tests ──────────────────────────────────────

    #[test]
    fn sphere_2d() {
        let result = direct_minimize(
            &mut sphere,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            5000,
            None,
        ).unwrap();

        assert!(result.value < 0.01, "sphere min should be near 0, got {}", result.value);
        assert!(result.point[0].abs() < 0.1, "x0 should be near 0, got {}", result.point[0]);
        assert!(result.point[1].abs() < 0.1, "x1 should be near 0, got {}", result.point[1]);
    }

    #[test]
    fn sphere_1d() {
        let result = direct_minimize(
            &mut |x: &[f64]| x[0] * x[0],
            &[-10.0],
            &[10.0],
            1e-4,
            1000,
            None,
        ).unwrap();

        assert!(result.value < 0.01, "1D sphere min should be near 0, got {}", result.value);
    }

    #[test]
    fn rosenbrock_2d_converges() {
        let result = direct_minimize(
            &mut rosenbrock_2d,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            20000,
            None,
        ).unwrap();

        assert!(result.value < 1.0, "Rosenbrock 2D should get close: value={}", result.value);
        assert!(
            (result.point[0] - 1.0).abs() < 0.5,
            "x0 should be near 1, got {}",
            result.point[0]
        );
    }

    #[test]
    fn rosenbrock_5d_converges() {
        let result = direct_minimize(
            &mut rosenbrock_nd,
            &[-2.0; 5],
            &[2.0; 5],
            1e-4,
            50000,
            None,
        ).unwrap();

        // DIRECT alone may not converge tightly on 5D Rosenbrock
        // but should get reasonably close
        assert!(
            result.value < 10.0,
            "5D Rosenbrock should get close: value={}",
            result.value
        );
    }

    // ── Six-hump camel (NLopt parity test) ───────────────────────────

    #[test]
    fn six_hump_camel_finds_global() {
        let result = direct_minimize(
            &mut six_hump_camel,
            &[-3.0, -2.0],
            &[3.0, 2.0],
            1e-4,
            10000,
            None,
        ).unwrap();

        assert!(
            (result.value - (-1.0316)).abs() < 0.01,
            "six-hump camel global min ~ -1.0316, got {}",
            result.value
        );

        // Should be near one of the two global minima
        let near_min1 = (result.point[0] - 0.0898).abs() < 0.1
            && (result.point[1] - (-0.7126)).abs() < 0.1;
        let near_min2 = (result.point[0] - (-0.0898)).abs() < 0.1
            && (result.point[1] - 0.7126).abs() < 0.1;
        assert!(
            near_min1 || near_min2,
            "should be near a global minimum, got ({}, {})",
            result.point[0],
            result.point[1]
        );
    }

    // ── Multimodal tests ─────────────────────────────────────────────

    #[test]
    fn rastrigin_2d_finds_global() {
        let result = direct_minimize(
            &mut rastrigin_2d,
            &[-5.12, -5.12],
            &[5.12, 5.12],
            1e-4,
            20000,
            None,
        ).unwrap();

        assert!(
            result.value < 1.0,
            "Rastrigin 2D global min = 0, got {}",
            result.value
        );
    }

    #[test]
    fn styblinski_tang_2d() {
        let result = direct_minimize(
            &mut styblinski_tang,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            10000,
            None,
        ).unwrap();

        let expected_min = -39.16599 * 2.0;
        assert!(
            (result.value - expected_min).abs() < 1.0,
            "Styblinski-Tang 2D min ~ {expected_min}, got {}",
            result.value
        );
    }

    // ── Target value (early stopping) ────────────────────────────────

    #[test]
    fn target_value_stops_early() {
        let result = direct_minimize(
            &mut sphere,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            50000,
            Some(1.0), // stop when f <= 1.0
        ).unwrap();

        assert!(
            result.value <= 1.0 + 1e-10,
            "should stop at or below target: got {}",
            result.value
        );
        // Should stop well before 50000 calls
        assert!(
            result.evaluations < 5000,
            "should stop early: {} calls",
            result.evaluations
        );
    }

    // ── Callback cancellation ────────────────────────────────────────

    #[test]
    fn callback_can_cancel() {
        let mut count = 0;
        let result = direct_minimize_with_callback(
            &mut sphere,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            50000,
            None,
            &mut |_| {
                count += 1;
                count < 3 // cancel after 3 iterations
            },
        ).unwrap();

        // Should have stopped early
        assert!(
            result.evaluations < 1000,
            "cancellation should limit calls: {}",
            result.evaluations
        );
    }

    // ── Edge cases ───────────────────────────────────────────────────

    #[test]
    fn tight_bounds() {
        // Bounds are already near the optimum
        let result = direct_minimize(
            &mut sphere,
            &[-0.1, -0.1],
            &[0.1, 0.1],
            1e-4,
            5000,
            None,
        ).unwrap();

        assert!(result.value < 0.001, "tight bounds: value={}", result.value);
    }

    #[test]
    fn asymmetric_bounds() {
        let result = direct_minimize(
            &mut |x: &[f64]| (x[0] - 3.0).powi(2) + (x[1] + 2.0).powi(2),
            &[0.0, -5.0],
            &[10.0, 0.0],
            1e-4,
            5000,
            None,
        ).unwrap();

        assert!(
            (result.point[0] - 3.0).abs() < 0.2,
            "x0 should be near 3, got {}",
            result.point[0]
        );
        assert!(
            (result.point[1] - (-2.0)).abs() < 0.2,
            "x1 should be near -2, got {}",
            result.point[1]
        );
    }

    #[test]
    fn fixed_dimension() {
        // One dimension has zero range (fixed)
        let result = direct_minimize(
            &mut |x: &[f64]| x[0] * x[0] + x[1] * x[1],
            &[-5.0, 3.0],  // x[1] is fixed at 3.0
            &[5.0, 3.0],
            1e-4,
            5000,
            None,
        ).unwrap();

        assert!(result.point[0].abs() < 0.1, "x0 should be near 0");
        assert!((result.point[1] - 3.0).abs() < 1e-10, "x1 should be exactly 3.0");
    }

    #[test]
    fn invalid_inputs_return_none() {
        // Empty dimensions
        assert!(direct_minimize(&mut |_: &[f64]| 0.0, &[], &[], 1e-4, 100, None).is_none());

        // Mismatched bounds
        assert!(direct_minimize(
            &mut |_: &[f64]| 0.0,
            &[0.0, 0.0],
            &[1.0],
            1e-4,
            100,
            None,
        ).is_none());

        // Inverted bounds
        assert!(direct_minimize(
            &mut |_: &[f64]| 0.0,
            &[5.0],
            &[0.0],
            1e-4,
            100,
            None,
        ).is_none());
    }

    // ── Convergence behavior tests ───────────────────────────────────

    #[test]
    fn tighter_threshold_gives_better_result() {
        let loose = direct_minimize(
            &mut rosenbrock_2d,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-2,
            10000,
            None,
        ).unwrap();

        let tight = direct_minimize(
            &mut rosenbrock_2d,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-5,
            50000,
            None,
        ).unwrap();

        assert!(
            tight.value <= loose.value + 0.01,
            "tighter threshold should give equal or better result: loose={}, tight={}",
            loose.value,
            tight.value,
        );
    }

    #[test]
    fn reasonable_call_count() {
        let result = direct_minimize(
            &mut six_hump_camel,
            &[-3.0, -2.0],
            &[3.0, 2.0],
            1e-4,
            50000,
            None,
        ).unwrap();

        // The Java test expects < 680 for convergence_threshold = 0.005
        // With 1e-4 threshold we expect more, but should be bounded
        assert!(
            result.evaluations < 20000,
            "six-hump camel should converge in reasonable calls: {}",
            result.evaluations
        );
    }

    // ── Progress callback tests ──────────────────────────────────────

    #[test]
    fn progress_reports_improvement() {
        let mut best_values = Vec::new();
        let _ = direct_minimize_with_callback(
            &mut sphere,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            5000,
            None,
            &mut |p| {
                best_values.push(p.best_value);
                true
            },
        );

        // Best value should be monotonically non-increasing
        for i in 1..best_values.len() {
            assert!(
                best_values[i] <= best_values[i - 1],
                "best value should be non-increasing: {} > {}",
                best_values[i],
                best_values[i - 1]
            );
        }
    }

    #[test]
    fn progress_reports_growing_rectangles() {
        let mut rect_counts = Vec::new();
        let _ = direct_minimize_with_callback(
            &mut sphere,
            &[-5.0, -5.0],
            &[5.0, 5.0],
            1e-4,
            5000,
            None,
            &mut |p| {
                rect_counts.push(p.num_rectangles);
                true
            },
        );

        // Number of rectangles should generally increase
        assert!(
            rect_counts.last().unwrap() > rect_counts.first().unwrap(),
            "rectangle count should increase over time"
        );
    }

    // ── Higher dimension test ────────────────────────────────────────

    #[test]
    fn sphere_5d() {
        let result = direct_minimize(
            &mut sphere,
            &[-5.0; 5],
            &[5.0; 5],
            1e-3,
            30000,
            None,
        ).unwrap();

        assert!(
            result.value < 0.5,
            "5D sphere should converge: value={}",
            result.value
        );
    }

    // ── Goldstein-Price (from Java tests) ─────────────────────────────

    #[test]
    fn goldstein_price() {
        let result = direct_minimize(
            &mut |x: &[f64]| {
                let x1 = x[0]; let x2 = x[1];
                let a = 1.0 + (x1 + x2 + 1.0).powi(2)
                    * (19.0 - 14.0*x1 + 3.0*x1*x1
                       - 14.0*x2 + 6.0*x1*x2 + 3.0*x2*x2);
                let b = 30.0 + (2.0*x1 - 3.0*x2).powi(2)
                    * (18.0 - 32.0*x1 + 12.0*x1*x1
                       + 48.0*x2 - 36.0*x1*x2 + 27.0*x2*x2);
                a * b
            },
            &[-2.0, -2.0],
            &[2.0, 2.0],
            5e-3,
            5000,
            None,
        ).unwrap();

        assert!(
            (result.value - 3.0).abs() < 0.1,
            "Goldstein-Price global min = 3, got {}",
            result.value
        );
    }
}
