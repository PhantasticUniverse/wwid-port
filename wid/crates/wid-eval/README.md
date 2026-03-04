# wid-eval

Top-level impedance evaluation pipeline: computes input impedance, finds playing frequencies, and produces cents deviations for each fingering. Supports NAF, Whistle, Flute, and Reed study models.

## Public API

| Function | Description |
|----------|-------------|
| `calc_z()` | Input impedance Z(f) for a compiled instrument + fingering |
| `calc_z_samples()` | Impedance at multiple frequencies |
| `predicted_frequency()` | Playing frequency via Im(Z)=0 root finding (NAF: simple tuner) |
| `predicted_frequency_linear_v()` | Playing frequency via gain-aware fmin search (Whistle/Flute: LinearV tuner) |
| `calculate_error_vector()` | Cents deviation per fingering in a tuning |
| `cents()` | 1200 × log₂(predicted/target) |

## CalculatorParams

Study model parameters that control the impedance pipeline:

| Const | `hole_size_mult` | `finger_adjustment` | Termination | Mouthpiece | Blowing level |
|-------|----------------:|--------------------:|-------------|------------|:-------------:|
| `NAF` | 0.9605 | 0.0 | ThickFlanged | DefaultFipple | — |
| `WHISTLE` | 1.0 | 0.010 | Unflanged | SimpleFipple | 5 |
| `FLUTE` | 1.0 | 0.010 | Unflanged | SimpleFipple | 5 |
| `REED` | 1.0 | 0.010 | Unflanged | SimpleReed | — |

`FLUTE` is identical to `WHISTLE` — `FluteStudyModel extends WhistleStudyModel` in the Java baseline. The mouthpiece difference (EmbouchureHole vs Fipple) is handled internally by `simple_fipple` parameter extraction.

`REED` uses `SimpleInstrumentTuner` (standard reactance-zero search, same as NAF), NOT the LinearV tuner. `finger_adjustment=0.010` matches Java's `DefaultHoleCalculator()` no-arg constructor default.

## Tuner models

| Tuner | Study models | Algorithm |
|-------|-------------|-----------|
| Simple | NAF | Bracket search for Im(Z)=0, Brent-Dekker root finding |
| LinearV | Whistle, Flute | Strouhal model with velocity interpolation, gain-aware fmin search |

The LinearV tuner (`linear_v.rs`) computes gain factor (Auvray 2012 model) from mouthpiece geometry and finds the frequency that minimizes `|fmin - freq|` where fmin accounts for Strouhal number and airstream velocity.

## Dependencies

- `wid-math` — TransferMatrix and StateVector
- `wid-physics` — PhysicalParameters
- `wid-types` — Fingering type
- `wid-compile` — InstrumentCompiled and Component types
- `wid-acoustics` — bore, hole, termination, mouthpiece, simple_fipple calculators
- `num-complex` — Complex64

## Impedance pipeline

1. Initialize state vector at termination (ThickFlanged or Unflanged)
2. Walk components in reverse (termination → mouthpiece), applying transfer matrices
3. Apply mouthpiece model (DefaultFipple: headspace + fipple; SimpleFipple: window impedance)
4. Return Z = P/U

## Root finding

Bracket search + Brent-Dekker solver for Im(Z) = 0 crossings. Bracket preference logic matches upstream exactly — when the primary-direction bracket is outside the preferred ratio, the fallback direction is preferred unconditionally.

## Tests

12 unit tests + 19 integration tests:

**Unit tests**: Brent solver (5), calculator params (2), Z-samples (1), evaluation (1), error vector (1), cents utility (2).

**Integration tests**:
- `bulk_naf_eval.rs` (4): All 36 NAF combos (6×6 = 540 fingerings). Max cents deviation: 0.000003.
- `bulk_whistle_eval.rs` (4): All 16 Whistle combos (2×8 = 272 fingerings). Max cents deviation: 0.000002.
- `whistle_z_sample.rs` (1): Z-sample parity for SamplePVC-Whistle.
- `bulk_flute_eval.rs` (4): All 8 Flute combos (2×4 = 110 fingerings). Max cents deviation: 0.058.
- `flute_z_sample.rs` (1): Z-sample parity for SamplePVC-Flute (exact match).
- `bulk_reed_eval.rs` (4): All 7 Reed combos (72 fingerings). Max cents deviation: 0.000011.
- `reed_z_sample.rs` (1): Z-sample parity for SampleChanter-Reed.
