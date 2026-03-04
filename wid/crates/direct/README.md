# direct

Pure Rust implementation of the **DIRECT-C** (DIviding RECTangles, Centred variant)
algorithm for derivative-free global optimization with bound constraints.

## What is DIRECT?

DIRECT is a deterministic global optimizer for the Lipschitz optimization problem:
minimize f(x) over a box [lower, upper] ⊂ ℝⁿ without knowing the Lipschitz constant K.

Unlike local optimizers (BOBYQA, Nelder-Mead), DIRECT is designed to find the
**global** minimum by systematically exploring the entire search space. It does
this by simultaneously considering all possible Lipschitz constants, balancing
exploration of unexplored regions with exploitation near known good points.

### Algorithm Intuition

1. Start with one hyperrectangle covering the entire search space, evaluated at its center
2. At each iteration, select **potentially optimal hyperrectangles** (POH) — those on
   the lower-right convex hull of (size, value) space
3. Trisect each POH along its longest side(s), evaluating f at two new center points
4. The convex hull naturally balances:
   - **Exploration**: large rectangles (might contain unexplored basins)
   - **Exploitation**: rectangles with low function values (refining known good regions)

### DIRECT-C Variant

This implementation includes three algorithm layers:

- **Base DIRECT** (Jones 1993): Standard convex hull POH selection and trisection
- **DIRECT-1** (Gablonsky 2001): When a rectangle is not a hypercube, divide only
  one long side — the one with highest improvement "potential"
- **DIRECT-C** (Patkau): When standard POH selection stagnates, switch to alternate
  strategies that search near the current best point:
  - **Large & Near**: For each size level, prefer rectangles closest to the best point
  - **Low Value & Near**: Bin rectangles by distance to best, keep lowest-value per bin

## Usage

```rust
use direct::direct_minimize;

// Minimize the 2D Rosenbrock function over [-5, 5]²
let result = direct_minimize(
    &mut |x| {
        let t = x[0] * x[0] - x[1];
        100.0 * t * t + (x[0] - 1.0) * (x[0] - 1.0)
    },
    &[-5.0, -5.0],    // lower bounds
    &[5.0, 5.0],      // upper bounds
    1e-6,              // convergence threshold (rectangle side length)
    10000,             // max function evaluations
    None,              // no target value
).unwrap();

assert!((result.point[0] - 1.0).abs() < 0.1);
assert!(result.value < 0.01);
```

## When to Use DIRECT

**Good for:**
- Black-box functions with unknown landscape (multimodal, non-smooth)
- Moderate dimensionality (2–15 variables)
- Finding the approximate basin of the global minimum
- As the first stage of a two-stage strategy: DIRECT → local refinement (e.g., BOBYQA)

**Not ideal for:**
- Known smooth, unimodal problems (use BOBYQA directly)
- Very high dimensionality (>20 variables) — curse of dimensionality
- Tight convergence (DIRECT converges slowly near the optimum — refine with a local method)

## Parameters

| Parameter | Description | Guidance |
|-----------|-------------|----------|
| `convergence_threshold` | Stop when all sides of the best rectangle are below this | 1e-6 to 1e-8 typical |
| `max_evaluations` | Budget cap on function evaluations | 10K–100K depending on dimension |
| `target_value` | Stop immediately when f(x) ≤ target | Use `Some(0.001)` in two-stage pipelines |

## Convergence Properties

DIRECT provides a proven dense-sampling guarantee (Finkel & Kelley, 2004): for any
point x in the search domain and any δ > 0, DIRECT will eventually sample a point
within δ of x. This means the algorithm will find any global minimum given sufficient
evaluations.

In practice, DIRECT finds the correct basin quickly but converges slowly to the
exact minimum. For precise results, follow DIRECT with a local optimizer like BOBYQA.

## References

1. Jones, D.R., Perttunen, C.D. & Stuckman, B.E. "Lipschitzian optimization without
   the Lipschitz constant." *J. Optim. Theory Appl.* 79, 157–181 (1993).
2. Gablonsky, J.M. & Kelley, C.T. "A locally-biased form of the DIRECT algorithm."
   *J. Global Optim.* 21, 27–37 (2001).
3. Finkel, D.E. & Kelley, C.T. "Convergence analysis of the DIRECT algorithm." (2004).
4. Jones, D.R. & Martins, J.R.R.A. "The DIRECT algorithm: 25 years later."
   *J. Global Optim.* 79, 521–566 (2021).

## License

Apache-2.0
