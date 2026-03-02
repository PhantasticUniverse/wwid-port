# bobyqa

Pure Rust implementation of Powell's **BOBYQA** (Bound Optimization BY Quadratic
Approximation) algorithm for derivative-free optimization with bound constraints.

## What is BOBYQA?

BOBYQA minimizes an objective function of several variables when:

- **No derivatives are available** (the function is a black box).
- Variables are subject to **simple lower/upper bound constraints**.
- The function is **smooth** (at least approximately).

It maintains a quadratic interpolation model of the objective and uses a trust
region method to converge to a local minimum. The algorithm was developed by
M.J.D. Powell and published in 2009.

## Usage

```rust
use bobyqa::bobyqa_minimize;

// Minimize f(x) = (x0 - 3)^2 + (x1 - 4)^2 subject to 0 <= x <= 10
let result = bobyqa_minimize(
    &mut |x| (x[0] - 3.0).powi(2) + (x[1] - 4.0).powi(2),
    &[0.0, 0.0],       // starting point
    &[0.0, 0.0],       // lower bounds
    &[10.0, 10.0],     // upper bounds
    5,                   // interpolation points (2*n + 1)
    1.0,                 // initial trust region radius
    1e-8,                // stopping trust region radius
    1000,                // max evaluations
)
.expect("optimization should converge");

assert!((result.point[0] - 3.0).abs() < 1e-6);
assert!((result.point[1] - 4.0).abs() < 1e-6);
assert!(result.value < 1e-12);
println!("Found minimum at ({:.6}, {:.6}) in {} evaluations",
    result.point[0], result.point[1], result.evaluations);
```

## Parameters

| Parameter | Description | Typical value |
|---|---|---|
| `n_interp` | Number of interpolation points. Must be in `[n+2, (n+1)(n+2)/2]`. | `2*n + 1` |
| `initial_trust` | Initial trust region radius. Scale of expected distance to optimum. | Problem-dependent |
| `stopping_trust` | Convergence threshold for trust region radius. | `1e-6` to `1e-10` |
| `max_eval` | Budget for function evaluations. | `1000` to `100000` |

## Features

- **Zero dependencies** — all linear algebra is performed inline. No BLAS, no
  LAPACK, no C bindings.
- **WebAssembly ready** — compiles cleanly to `wasm32-unknown-unknown`.
- **Faithful port** — translated from Apache Commons Math 3.6.1's
  `BOBYQAOptimizer`, which was itself translated from Powell's original Fortran.
  Achieves bit-identical results on many test functions.
- **Comprehensive test suite** — 32 tests including convergence tests on
  standard optimization benchmarks (Rosenbrock, Ackley, Rastrigin, Powell, etc.)
  and floating-point parity tests against Apache Commons Math 3.

## Parity with Apache Commons Math 3

The implementation is validated against exact outputs from Apache Commons Math
3.6.1's `BOBYQAOptimizer`. On simple quadratic functions (sphere, cigar, tablet,
bounded quadratic), the Rust port produces **bit-identical** evaluation counts
and function values. On harder functions (Rosenbrock, Powell's points-in-square),
minor floating-point divergence leads to slightly different iteration paths, but
convergence to the same optima is verified.

## Algorithm overview

1. **Initialization** (`prelim`): Build `npt` interpolation points around the
   starting point, evaluate the objective at each, and initialize the quadratic
   model.

2. **Main loop** (`bobyqb`): At each iteration, either:
   - Solve a **trust region subproblem** (`trsbox`) — minimize the quadratic
     model within the trust region intersected with the bounds.
   - Perform a **geometry improvement** step (`altmov`) — move an interpolation
     point to improve the model's conditioning.

3. **Model update** (`update`): After evaluating the new point, update the
   quadratic model using a rank-one update of the interpolation system.

4. **Convergence**: The trust region radius shrinks over time. When it reaches
   `stopping_trust`, the algorithm terminates and returns the best point found.

## References

- Powell, M.J.D. (2009). "The BOBYQA algorithm for bound constrained
  optimization without derivatives." Cambridge NA Report NA2009/06.
- Apache Commons Math 3.6.1,
  `org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer`

## License

Apache-2.0 (same as the original Apache Commons Math implementation).
