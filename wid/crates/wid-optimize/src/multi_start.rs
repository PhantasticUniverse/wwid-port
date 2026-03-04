//! Multi-start optimization infrastructure.
//!
//! Provides random and grid-based starting point generators, and a
//! multi-start BOBYQA runner that tries multiple starting points and
//! returns the best result.

use bobyqa_impl::{BobyqaResult, bobyqa_minimize};

// ── Result types ─────────────────────────────────────────────────────

/// Result of a multi-start optimization.
#[derive(Debug, Clone)]
pub struct MultiStartResult {
    /// Best point found across all starts.
    pub point: Vec<f64>,
    /// Function value at the best point.
    pub value: f64,
    /// Total function calls across all starts.
    pub total_evaluations: usize,
    /// Number of starts completed.
    pub starts_completed: usize,
}

/// Progress information during multi-start optimization.
#[derive(Debug, Clone)]
pub struct MultiStartProgress {
    /// Index of the current start (0-based).
    pub start_index: usize,
    /// Total number of starts planned.
    pub total_starts: usize,
    /// Function calls in the current start.
    pub evaluations_this_start: usize,
    /// Total function calls across all starts so far.
    pub total_evaluations: usize,
    /// Best function value found so far (across all starts).
    pub best_value: f64,
}

// ── Starting point generators ────────────────────────────────────────

/// Generate `n_starts` random points uniformly in [lower, upper].
///
/// Uses a simple xoshiro256** seeded PRNG for reproducibility.
/// Zero external dependencies.
pub fn random_start_points(
    lower: &[f64],
    upper: &[f64],
    n_starts: usize,
    seed: u64,
) -> Vec<Vec<f64>> {
    let n = lower.len();
    let mut rng = Xoshiro256ss::new(seed);
    let mut points = Vec::with_capacity(n_starts);

    for _ in 0..n_starts {
        let mut point = vec![0.0; n];
        for i in 0..n {
            point[i] = lower[i] + rng.next_f64() * (upper[i] - lower[i]);
        }
        points.push(point);
    }
    points
}

/// Generate grid-spaced points within bounds.
///
/// Places points at interior positions matching Java's `GridRangeProcessor`:
/// for N points per dimension, divides the range into N+1 sub-intervals and
/// places points at positions `lower + range * k / (N+1)` for k = 1..N.
///
/// Fixed dimensions (where lower == upper) use the static_point value.
///
/// The granularity per dimension is `ceil(n_starts^(1/n_varying))`.
/// The total number of points may exceed n_starts to fill the grid.
pub fn grid_start_points(
    lower: &[f64],
    upper: &[f64],
    n_starts: usize,
    static_point: &[f64],
    varying_dims: Option<&[usize]>,
) -> Vec<Vec<f64>> {
    let n = lower.len();

    // Determine which dimensions vary
    let varying: Vec<usize> = match varying_dims {
        Some(dims) => dims.to_vec(),
        None => (0..n).filter(|&i| upper[i] > lower[i]).collect(),
    };

    if varying.is_empty() {
        return vec![static_point.to_vec()];
    }

    let n_varying = varying.len();
    // Grid points per dimension: ceil(n_starts^(1/n_varying))
    let points_per_dim = (n_starts as f64).powf(1.0 / n_varying as f64).ceil() as usize;
    let points_per_dim = points_per_dim.max(1);

    // Generate grid using recursive Cartesian product
    let mut grid_indices = vec![0usize; n_varying];
    let mut points = Vec::new();

    loop {
        let mut point = static_point.to_vec();
        for (vi, &dim) in varying.iter().enumerate() {
            let range = upper[dim] - lower[dim];
            // Interior point: range / (N+1) * k, for k = 1..N
            // Matches Java GridRangeProcessor.resetRangesToGrid()
            point[dim] = lower[dim]
                + range * (grid_indices[vi] as f64 + 1.0) / (points_per_dim as f64 + 1.0);
        }
        points.push(point);

        // Increment grid indices (odometer style)
        let mut carry = true;
        for vi in (0..n_varying).rev() {
            if carry {
                grid_indices[vi] += 1;
                if grid_indices[vi] >= points_per_dim {
                    grid_indices[vi] = 0;
                } else {
                    carry = false;
                }
            }
        }
        if carry {
            break; // All indices wrapped around
        }
    }

    points
}

// ── Multi-start BOBYQA runner ────────────────────────────────────────

/// Run BOBYQA from each start point, return the best result.
pub fn multi_start_bobyqa(
    f: &mut dyn FnMut(&[f64]) -> f64,
    start_points: &[Vec<f64>],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    n_interp: usize,
    initial_trust: f64,
    stopping_trust: f64,
    max_calls_per_start: usize,
) -> Option<MultiStartResult> {
    multi_start_bobyqa_with_progress(
        f,
        start_points,
        lower_bounds,
        upper_bounds,
        n_interp,
        initial_trust,
        stopping_trust,
        max_calls_per_start,
        &mut |_| true,
    )
}

/// Run BOBYQA from each start point with progress callback.
///
/// The callback receives [`MultiStartProgress`] after each start completes
/// and returns `true` to continue or `false` to cancel.
pub fn multi_start_bobyqa_with_progress(
    f: &mut dyn FnMut(&[f64]) -> f64,
    start_points: &[Vec<f64>],
    lower_bounds: &[f64],
    upper_bounds: &[f64],
    n_interp: usize,
    initial_trust: f64,
    stopping_trust: f64,
    max_calls_per_start: usize,
    on_progress: &mut dyn FnMut(MultiStartProgress) -> bool,
) -> Option<MultiStartResult> {
    if start_points.is_empty() {
        return None;
    }

    let n = lower_bounds.len();
    let mut best: Option<BobyqaResult> = None;
    let mut total_evals = 0usize;
    let mut starts_completed = 0usize;

    for (i, start) in start_points.iter().enumerate() {
        // Clamp start point to bounds
        let clamped: Vec<f64> = start
            .iter()
            .enumerate()
            .map(|(j, &v)| {
                if j < n {
                    v.clamp(lower_bounds[j], upper_bounds[j])
                } else {
                    v
                }
            })
            .collect();

        // Track per-start calls
        let mut start_evals = 0usize;
        let total_before = total_evals;

        let result = bobyqa_minimize(
            &mut |point: &[f64]| {
                start_evals += 1;
                total_evals += 1;
                f(point)
            },
            &clamped,
            lower_bounds,
            upper_bounds,
            n_interp,
            initial_trust,
            stopping_trust,
            max_calls_per_start,
        );

        // Fix up total_evals in case bobyqa_minimize counted differently
        total_evals = total_before + start_evals;
        starts_completed += 1;

        if let Some(res) = result {
            let is_better = match &best {
                None => true,
                Some(b) => res.value < b.value,
            };
            if is_better {
                best = Some(res);
            }
        }

        // Progress callback
        let keep_going = on_progress(MultiStartProgress {
            start_index: i,
            total_starts: start_points.len(),
            evaluations_this_start: start_evals,
            total_evaluations: total_evals,
            best_value: best.as_ref().map_or(f64::MAX, |b| b.value),
        });
        if !keep_going {
            break;
        }
    }

    best.map(|b| MultiStartResult {
        point: b.point,
        value: b.value,
        total_evaluations: total_evals,
        starts_completed,
    })
}

// ── Xoshiro256** PRNG ────────────────────────────────────────────────

/// Minimal xoshiro256** PRNG for reproducible random number generation.
///
/// Based on the reference implementation by Blackman and Vigna (2018).
/// Public domain / CC0.
struct Xoshiro256ss {
    s: [u64; 4],
}

impl Xoshiro256ss {
    fn new(seed: u64) -> Self {
        // SplitMix64 for seeding (Vigna, 2015)
        let mut sm = seed;
        let mut s = [0u64; 4];
        for item in &mut s {
            sm = sm.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = sm;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            *item = z ^ (z >> 31);
        }
        Xoshiro256ss { s }
    }

    fn next_u64(&mut self) -> u64 {
        let result = (self.s[1].wrapping_mul(5)).rotate_left(7).wrapping_mul(9);
        let t = self.s[1] << 17;
        self.s[2] ^= self.s[0];
        self.s[3] ^= self.s[1];
        self.s[1] ^= self.s[2];
        self.s[0] ^= self.s[3];
        self.s[2] ^= t;
        self.s[3] = self.s[3].rotate_left(45);
        result
    }

    fn next_f64(&mut self) -> f64 {
        // Uniform in [0, 1)
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn random_points_within_bounds() {
        let lower = vec![-5.0, -3.0, 0.0];
        let upper = vec![5.0, 3.0, 10.0];
        let points = random_start_points(&lower, &upper, 100, 42);

        assert_eq!(points.len(), 100);
        for pt in &points {
            assert_eq!(pt.len(), 3);
            for i in 0..3 {
                assert!(pt[i] >= lower[i], "point below lower bound");
                assert!(pt[i] < upper[i], "point above upper bound");
            }
        }
    }

    #[test]
    fn random_points_seeded_reproducibility() {
        let lower = vec![-1.0, -1.0];
        let upper = vec![1.0, 1.0];
        let a = random_start_points(&lower, &upper, 10, 42);
        let b = random_start_points(&lower, &upper, 10, 42);
        assert_eq!(a, b, "same seed should give same points");
    }

    #[test]
    fn random_points_different_seeds() {
        let lower = vec![-1.0, -1.0];
        let upper = vec![1.0, 1.0];
        let a = random_start_points(&lower, &upper, 10, 42);
        let b = random_start_points(&lower, &upper, 10, 99);
        assert_ne!(a, b, "different seeds should give different points");
    }

    #[test]
    fn grid_points_2d_4_starts() {
        let lower = vec![0.0, 0.0];
        let upper = vec![1.0, 1.0];
        let static_pt = vec![0.5, 0.5];
        let points = grid_start_points(&lower, &upper, 4, &static_pt, None);

        // ceil(4^(1/2)) = 2, so 2x2 = 4 grid points
        assert_eq!(points.len(), 4);
        // Interior points of 2x2 grid: (1/3, 1/3), (1/3, 2/3), (2/3, 1/3), (2/3, 2/3)
        // Matching Java GridRangeProcessor: range/(N+1) * k for k=1..N
        let expected_coords = [1.0 / 3.0, 2.0 / 3.0];
        for pt in &points {
            assert!(expected_coords.iter().any(|&c| (pt[0] - c).abs() < 1e-10));
            assert!(expected_coords.iter().any(|&c| (pt[1] - c).abs() < 1e-10));
        }
    }

    #[test]
    fn grid_points_fixed_dimension() {
        let lower = vec![0.0, 5.0]; // dim 1 is fixed
        let upper = vec![1.0, 5.0];
        let static_pt = vec![0.5, 5.0];
        let points = grid_start_points(&lower, &upper, 4, &static_pt, None);

        // Only 1 varying dimension, so 4 points along dim 0
        assert_eq!(points.len(), 4);
        for pt in &points {
            assert!((pt[1] - 5.0).abs() < 1e-10, "fixed dim should stay at 5.0");
        }
    }

    #[test]
    fn grid_points_with_varying_dims() {
        let lower = vec![0.0, 0.0, 0.0];
        let upper = vec![1.0, 1.0, 1.0];
        let static_pt = vec![0.5, 0.5, 0.5];
        // Only vary dims 0 and 2
        let points = grid_start_points(&lower, &upper, 4, &static_pt, Some(&[0, 2]));

        // ceil(4^(1/2)) = 2, so 2x2 = 4 grid points
        assert_eq!(points.len(), 4);
        for pt in &points {
            assert!((pt[1] - 0.5).abs() < 1e-10, "dim 1 should stay at static value");
        }
    }

    #[test]
    fn multi_start_improves_over_single() {
        // Rosenbrock with bad initial point
        let lower = vec![-5.0, -5.0];
        let upper = vec![5.0, 5.0];
        let starts = random_start_points(&lower, &upper, 5, 42);
        let n = 2;

        let result = multi_start_bobyqa(
            &mut |x: &[f64]| {
                let t = x[0] * x[0] - x[1];
                100.0 * t * t + (x[0] - 1.0) * (x[0] - 1.0)
            },
            &starts,
            &lower,
            &upper,
            2 * n + 1,
            1.0,
            1e-8,
            5000,
        );

        let result = result.unwrap();
        assert!(result.value < 1.0, "multi-start should find good minimum: {}", result.value);
        assert_eq!(result.starts_completed, 5);
    }

    #[test]
    fn multi_start_progress_reports() {
        let lower = vec![-5.0, -5.0];
        let upper = vec![5.0, 5.0];
        let starts = random_start_points(&lower, &upper, 3, 42);
        let n = 2;

        let mut progress_reports = Vec::new();
        let _ = multi_start_bobyqa_with_progress(
            &mut |x: &[f64]| x[0] * x[0] + x[1] * x[1],
            &starts,
            &lower,
            &upper,
            2 * n + 1,
            1.0,
            1e-8,
            1000,
            &mut |p| {
                progress_reports.push(p);
                true
            },
        );

        assert_eq!(progress_reports.len(), 3, "should get progress for each start");
        assert_eq!(progress_reports[0].start_index, 0);
        assert_eq!(progress_reports[1].start_index, 1);
        assert_eq!(progress_reports[2].start_index, 2);
        assert_eq!(progress_reports[2].total_starts, 3);
    }

    #[test]
    fn multi_start_cancellation() {
        let lower = vec![-5.0, -5.0];
        let upper = vec![5.0, 5.0];
        let starts = random_start_points(&lower, &upper, 10, 42);
        let n = 2;

        let result = multi_start_bobyqa_with_progress(
            &mut |x: &[f64]| x[0] * x[0] + x[1] * x[1],
            &starts,
            &lower,
            &upper,
            2 * n + 1,
            1.0,
            1e-8,
            1000,
            &mut |p| p.start_index < 2, // cancel after 2 starts
        ).unwrap();

        assert!(result.starts_completed <= 3, "should stop early: {} starts", result.starts_completed);
    }
}
