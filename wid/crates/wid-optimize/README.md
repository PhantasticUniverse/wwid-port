# wid-optimize

Calibration and optimization infrastructure for instrument design. Provides three solver types — 1D Brent, N-dimensional BOBYQA, and DIRECT-C global — to adjust mouthpiece parameters and hole geometry across all four study models.

## Which Optimizer to Use

| Goal | Optimizer | Algorithm | Speed |
|------|-----------|-----------|-------|
| Tune mouthpiece to match a specific note | Fipple / Window Height / Airstream / Beta calibrator | 1D Brent | < 1 sec |
| Tune mouthpiece (two params jointly) | Whistle / Flute calibration | 2D BOBYQA | ~ 1 sec |
| Adjust hole diameters only | Hole size | N-dim BOBYQA | ~ 2 sec |
| Adjust hole positions only | Hole position | (N+1)-dim BOBYQA | ~ 3 sec |
| Adjust positions + diameters together | Hole combined | (2N+1)-dim BOBYQA | ~ 5 sec |
| Global search (unknown starting point) | Global hole / Global hole position | DIRECT-C → BOBYQA | ~ 10 sec |

**Rule of thumb**: Use calibrators first to get the mouthpiece right, then hole optimizers to fine-tune geometry. Use global optimizers when the starting geometry is far from optimal or you want to explore the full search space.

## Algorithm Overview

### Brent (1D)

Golden-section search with parabolic interpolation. Guaranteed convergence for unimodal functions. Used for single-parameter calibrators (fipple factor, window height, airstream length, beta).

### BOBYQA (bounded, N-dim)

Powell's Bound Optimization BY Quadratic Approximation. Builds a quadratic model from function evaluations (no derivatives needed). Trust-region based. See the [bobyqa crate](../bobyqa/README.md) for algorithm details.

### DIRECT-C → BOBYQA (global, N-dim)

Two-stage pipeline for global optimization:
1. **DIRECT-C** (convergence 7e-8, target value 0.001, 2× eval budget) explores the entire search space by subdividing hyperrectangles
2. **BOBYQA** refines from DIRECT-C's best point to converge precisely
3. The better of the two results is kept

See the [direct crate](../direct/README.md) for the DIRECT-C algorithm.

### Multi-start BOBYQA

Run BOBYQA from multiple starting points (random or grid-spaced) and keep the best result. Used when the objective landscape has multiple local minima. Includes seeded PRNG (xoshiro256**) for reproducible random starts.

## Calibrators

Mouthpiece calibrators adjust physical parameters to minimize evaluation error. Each operates on a single instrument and tuning pair.

| Module | Study Models | Algorithm | Params | Evaluator |
|--------|-------------|-----------|--------|-----------|
| `fipple` | NAF | 1D Brent | fipple factor | CentDeviation (lowest note only) |
| `window_height` | Whistle | 1D Brent | window height | Fmax |
| `beta` | Whistle, Flute | 1D Brent | beta factor | Fmin |
| `whistle_calib` | Whistle | 2D BOBYQA | window height + beta | Fminmax |
| `airstream_length` | Flute | 1D Brent | airstream length | Fmax |
| `flute_calib` | Flute | 2D BOBYQA | airstream length + beta | Fminmax |
| `reed_calib` | Reed | 2D BOBYQA | alpha + beta | CentDeviation |

## Hole Optimizers

Hole optimizers adjust geometry (positions, diameters, or both) within constraint bounds to minimize evaluation error.

| Module | Study Models | Dimensions | Geometry | Algorithm |
|--------|-------------|------------|----------|-----------|
| `hole_from_top` | NAF | 2N+1 | bore length + top fraction + spacings + diameters | BOBYQA |
| `hole_size` | Whistle, Flute, Reed | N | diameters only | BOBYQA |
| `hole_position` | Whistle, Flute, Reed | N+1 | bore end position + inter-hole spacings | BOBYQA |
| `hole_combined` | Whistle, Flute, Reed | 2N+1 | positions + diameters (merged) | BOBYQA |
| `global_optimize` | Whistle, Flute, Reed | N+1 or 2N+1 | same as position/combined | DIRECT-C → BOBYQA |

All hole optimizers support progress callbacks via `*_with_progress()` variants.

## Evaluator Dispatch

Calibrators and hole optimizers use different evaluator functions:

| Evaluator | Function | Description | Used By |
|-----------|----------|-------------|---------|
| CentDeviation | `calculate_error_vector` | `1200 × log₂(predicted/target)` per fingering | Hole optimizers, fipple calibrator, reed calibrator |
| Fmax | `calculate_fmax_error_vector` | cents(target.frequencyMax, predicted fmax) | Window height, airstream length calibrators |
| Fmin | `calculate_fmin_error_vector` | cents(target.frequencyMin, predicted fmin) | Beta calibrator |
| Fminmax | `calculate_fminmax_error_vector` | RMS of weighted fmax (×4) + fmin (×1) | Joint whistle/flute calibrators |

## BOBYQA Configuration

Standard BOBYQA optimizers (non-global) use these settings matching the Java baseline:

| Parameter | Value |
|-----------|-------|
| Initial trust region | 10.0 |
| Stopping trust region | 1e-8 |
| Max evaluations | `20000 + (n_dims - 1) × 5000` |
| Interpolation points | `2 × n_dims + 1` |
| Initial point | Clamped to bounds |

Global optimizers compute trust radius from constraint bounds (matching Java `BaseObjectiveFunction.getInitialTrustRegionRadius()`).

## Result Types

| Type | Fields |
|------|--------|
| `CalibrationResult` | initial/final fipple factor, window height, airstream length, alpha, beta, norms |
| `WhistleCalibrationResult` | initial/final window height, beta, norms |
| `FluteCalibrationResult` | initial/final airstream length, beta, norms |
| `ReedCalibrationResult` | initial/final alpha, beta, norms |
| `OptimizationResult` | initial/final norms, geometry vectors, evaluation count |
| `GlobalOptResult` | best point/value, DIRECT + BOBYQA evaluation counts |
| `MultiStartResult` | best point/value, total evaluations, starts completed |

## Dependencies

- `bobyqa` — BOBYQA multivariate optimizer
- `direct` — DIRECT-C global optimizer
- `wid-eval` — impedance evaluation (error vectors)
- `wid-compile` — geometry mutation and compilation
- `wid-physics` — physical parameters
- `wid-types` — instrument, tuning, constraints types

## Tests

74 tests:
- Brent minimizer: quadratic, cosine, tolerance, boundary (4)
- Fipple calibration: NAF golden fixtures + post-eval (6)
- Hole from top: NAF golden fixtures + constraints (7)
- Norm calculation: weighted, uniform, empty (3)
- Window height: initial value + norm + golden (3)
- Beta: initial value + norm + Whistle/Flute golden (4)
- Whistle joint: initial norm + golden (2)
- Airstream length: initial + roundtrip + fmax + golden + fife (5)
- Flute joint: initial norm + golden + fife (3)
- Hole size: Whistle + Flute golden + fife (5)
- Hole position: Whistle + Flute golden + fife (6)
- Hole combined: Whistle + Flute golden + fife (6)
- Reed calibration: initial norm + golden (2)
- Multi-start: bounds, reproducibility, grid, improvement, progress, cancellation (7)
- Global optimize: Rosenbrock, six-hump camel, sphere 5D (3)
- Global golden: initial norm, improvement, both stages, position, geometry, progress (6)
