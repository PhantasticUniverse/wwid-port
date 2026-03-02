# wid-eval

Top-level impedance evaluation pipeline: computes input impedance, finds playing frequencies, and produces cents deviations for each fingering.

## Public API

| Function | Description |
|----------|-------------|
| `calc_z()` | Input impedance Z(f) for a compiled instrument + fingering |
| `calc_z_samples()` | Impedance at multiple frequencies |
| `predicted_frequency()` | Playing frequency via Im(Z)=0 root finding |
| `calculate_error_vector()` | Cents deviation per fingering in a tuning |
| `cents()` | 1200 × log₂(predicted/target) |

## Dependencies

- `wid-math` — TransferMatrix and StateVector
- `wid-physics` — PhysicalParameters
- `wid-types` — Fingering type
- `wid-compile` — InstrumentCompiled and Component types
- `wid-acoustics` — bore, hole, termination, mouthpiece calculators
- `num-complex` — Complex64

## Impedance pipeline

1. Initialize state vector at termination (open or closed end)
2. Walk components in reverse (termination → mouthpiece), applying transfer matrices
3. Apply mouthpiece transfer matrix (headspace + fipple)
4. Return Z = P/U

## Root finding

Bracket search + Brent-Dekker solver for Im(Z) = 0 crossings. Bracket preference logic matches upstream exactly — when the primary-direction bracket is outside the preferred ratio, the fallback direction is preferred unconditionally.

## Tests

8 tests validating against golden fixtures:
- Z-samples: all 11 frequencies within `abs_err ≤ 1.0 + 1e-6 × max(|expected|, |actual|)`
- Evaluation: all 15 fingerings within 0.5 cents of oracle
- Error vector consistency
