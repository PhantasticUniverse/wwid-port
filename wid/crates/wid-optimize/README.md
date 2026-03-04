# wid-optimize

Calibration and optimization infrastructure for instrument design across all study models. Provides 1D Brent and N-dimensional BOBYQA optimizers for mouthpiece calibration and hole geometry optimization.

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

## Hole Optimizers

Hole optimizers adjust geometry (positions, diameters, or both) within constraint bounds to minimize evaluation error using BOBYQA.

| Module | Study Models | Dimensions | Geometry |
|--------|-------------|------------|----------|
| `hole_from_top` | NAF | 2N+1 | bore length + top hole fraction + spacings + diameters |
| `hole_size` | Whistle, Flute | N | diameters only |
| `hole_position` | Whistle, Flute | N+1 | bore end position + inter-hole spacings |
| `hole_combined` | Whistle, Flute | 2N+1 | positions + diameters (merged) |

All hole optimizers support progress callbacks via `*_with_progress()` variants.

## Evaluator Dispatch

Calibrators and hole optimizers use different evaluator functions:

| Evaluator | Function | Used By |
|-----------|----------|---------|
| CentDeviation | `calculate_error_vector` | Hole optimizers, fipple calibrator |
| Fmax | `calculate_fmax_error_vector` | Window height, airstream length calibrators |
| Fmin | `calculate_fmin_error_vector` | Beta calibrator |
| Fminmax | `calculate_fminmax_error_vector` | Joint whistle/flute calibrators |

## BOBYQA Configuration

All BOBYQA-based optimizers use consistent settings matching the Java baseline:

| Parameter | Value |
|-----------|-------|
| Initial trust region | 10.0 |
| Stopping trust region | 1e-8 |
| Max evaluations | `20000 + (n_dims - 1) × 5000` |
| Interpolation points | `2 × n_dims + 1` |
| Initial point | Clamped to bounds |

## Result Types

| Type | Fields |
|------|--------|
| `CalibrationResult` | initial/final fipple factor, norms |
| `WhistleCalibrationResult` | initial/final window height, beta, norms |
| `FluteCalibrationResult` | initial/final airstream length, beta, norms |
| `OptimizationResult` | initial/final norms, geometry vectors, evaluation count |

## Dependencies

- `bobyqa` — BOBYQA multivariate optimizer
- `wid-eval` — impedance evaluation (error vectors)
- `wid-compile` — geometry mutation and compilation
- `wid-physics` — physical parameters
- `wid-types` — instrument, tuning, constraints types

## Tests

54 tests:
- Brent minimizer: quadratic, cosine, tolerance matching, boundary start (4)
- Fipple calibration: NAF-FF-02 (0-hole), NAF-FF-03 (6-hole), post-calibration eval (6)
- Hole from top: NAF-OPT-01/02 golden, post-optimization eval, constraints bounds (7)
- Norm calculation: weighted, uniform, empty (3)
- Window height: initial value, initial norm, calibration golden (3)
- Beta: initial value, initial norm, Whistle golden, Flute golden (4)
- Whistle joint: initial norm, calibration golden (2)
- Airstream length: initial value, roundtrip, fmax norm, calibration, fife smoke (5)
- Flute joint: initial norm, calibration golden, fife smoke (3)
- Hole size: Whistle initial norm + golden, Flute initial norm + golden + fife smoke (5)
- Hole position: Whistle initial norm + geometry + golden, Flute initial norm + golden + fife smoke (6)
- Hole combined: Whistle initial norm + geometry + golden, Flute initial norm + golden + fife smoke (6)
