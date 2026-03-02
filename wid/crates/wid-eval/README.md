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

8 unit tests + 4 integration tests:

**Unit tests**: Z-samples (11 frequencies within tolerance), evaluation (15 fingerings within 0.5 cents of oracle), error vector consistency, cents utility.

**Integration tests** (`tests/bulk_naf_eval.rs`): All 36 NAF instrument×tuning combinations (6 bore sizes × 6 keys = 540 fingerings) verified against golden oracle data. Max cents deviation: 0.000003. Max predicted frequency relative error: 1.78e-9. Handles null predictions for mismatched bore/tuning combos.
