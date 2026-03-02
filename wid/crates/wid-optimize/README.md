# wid-optimize

Calibration and optimization for NAF instrument design. Provides fipple factor calibration (1D Brent) and hole geometry optimization (BOBYQA).

## Public API

| Type / Function | Description |
|----------------|-------------|
| `calibrate_fipple()` | 1D fipple factor calibration using Brent minimizer |
| `optimize_holes()` | Multi-variable hole geometry optimization using BOBYQA |
| `calc_norm()` | Weighted L2 norm matching Java `BaseObjectiveFunction.calcNorm()` |
| `fingering_weights()` | Extract optimization weights from fingerings |
| `CalibrationResult` | Initial/final fipple factor and norm |
| `OptimizationResult` | Initial/final norm, geometry, evaluation count |

## Fipple calibration (`fipple.rs`)

Matches Java `FippleFactorObjectiveFunction`:
- Uses only the lowest-frequency fingering (matching `getLowestNote`)
- 1D Brent optimizer (golden section + parabolic interpolation)
- Bounds from constraints XML (default: [0.2, 1.5])
- Objective: set fipple factor → compile → evaluate → weighted norm

## Hole optimization (`hole_from_top.rs`)

Matches Java `HoleFromTopObjectiveFunction` (a `MergedObjectiveFunction`):
- 13 dimensions for 6-hole NAF: `[bore_length, top_hole_fraction, 5 spacings, 6 diameters]`
- BOBYQA optimizer with bounds from constraints XML
- Trust region: 10.0 initial, 1e-8 stopping
- Max evaluations: `20000 + (n_dims - 1) × 5000`
- Interpolation points: `2 × n_dims + 1`
- Initial point clamped to bounds (matching Java `getInitialPoint`)

## Dependencies

- `bobyqa` — BOBYQA multivariate optimizer
- `wid-eval` — impedance evaluation (cents deviation)
- `wid-compile` — geometry mutation and compilation
- `wid-physics` — physical parameters
- `wid-types` — instrument, tuning, constraints types

## Tests

20 tests:
- Brent minimizer: quadratic, cosine, tolerance matching, boundary start (4)
- Fipple calibration: NAF-FF-02 (0-hole), NAF-FF-03 (6-hole), post-calibration eval (6)
- Hole optimization: NAF-OPT-01 (all weight=1), NAF-OPT-02 (weight=0 exclusion), post-optimization eval, constraints bounds (7)
- Norm calculation: weighted, uniform, empty (3)
