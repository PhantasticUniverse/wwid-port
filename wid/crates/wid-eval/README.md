# wid-eval

Top-level impedance evaluation pipeline: computes input impedance, finds playing frequencies, and produces cents deviations for each fingering. Supports NAF, Whistle, Flute, and Reed study models.

## Quick Start

The evaluation pipeline takes a compiled instrument + fingering and produces a playing frequency:

```
InstrumentRaw → compile() → InstrumentCompiled
                                    ↓
Fingering + PhysicalParameters + CalculatorParams
                                    ↓
                              calc_z(freq)         — impedance Z(f) at one frequency
                                    ↓
                         predicted_frequency()      — find Im(Z)=0 crossing (NAF/Reed)
                    or   predicted_frequency_linear_v() — gain-aware fmin (Whistle/Flute)
                                    ↓
                              cents(predicted, target)  — 1200 × log₂(pred/target)
                                    ↓
                         calculate_error_vector()   — cents for all fingerings
```

**Typical usage** (from wid-optimize or wid-session):

```rust
let compiled = compile(&instrument_raw)?;
let errors = calculate_error_vector(&compiled, &fingerings, &params, &calc_params);
let norm = calc_norm(&errors, &weights);  // weighted RMS
```

## Public API

| Function | Description |
|----------|-------------|
| `calc_z()` | Input impedance Z(f) for a compiled instrument + fingering |
| `calc_z_samples()` | Impedance at multiple frequencies (for Z-sample parity tests) |
| `predicted_frequency()` | Playing frequency via Im(Z)=0 root finding (NAF, Reed: simple tuner) |
| `predicted_frequency_linear_v()` | Playing frequency via gain-aware fmin search (Whistle, Flute: LinearV tuner) |
| `predicted_fmax()` | Maximum playing frequency (resonance frequency from Z) |
| `calculate_error_vector()` | Cents deviation per fingering using CentDeviationEvaluator logic |
| `calculate_fmax_error_vector()` | Fmax cents deviation per fingering (for calibrators) |
| `calculate_fmin_error_vector()` | Fmin cents deviation per fingering (for calibrators) |
| `calculate_fminmax_error_vector()` | Combined fmax + fmin with configurable weights (for joint calibrators) |
| `cents()` | 1200 × log₂(predicted/target) |

## CalculatorParams

Study model parameters that control the impedance pipeline:

| Const | `hole_size_mult` | `finger_adjustment` | Termination | Mouthpiece | Blowing level |
|-------|----------------:|--------------------:|-------------|------------|:-------------:|
| `NAF` | 0.9605 | 0.0 | ThickFlanged | DefaultFipple | — |
| `WHISTLE` | 1.0 | 0.010 | Unflanged | SimpleFipple | 5 |
| `FLUTE` | 1.0 | 0.010 | Unflanged | SimpleFipple | 5 |
| `REED` | 1.0 | 0.010 | Unflanged | SimpleReed | — |

`FLUTE` is identical to `WHISTLE` — `FluteStudyModel extends WhistleStudyModel` in Java. The mouthpiece difference (EmbouchureHole vs Fipple) is handled by parameter extraction in `simple_fipple`.

`REED` uses `SimpleInstrumentTuner` (standard reactance-zero search, same as NAF), NOT the LinearV tuner. `finger_adjustment=0.010` matches Java's `DefaultHoleCalculator()` no-arg constructor.

## Tuner models

| Tuner | Study models | Algorithm |
|-------|-------------|-----------|
| **Simple** | NAF, Reed | Bracket search for Im(Z)=0 crossing, Brent-Dekker root finding |
| **LinearV** | Whistle, Flute | Strouhal model with velocity interpolation, gain-aware fmin search |

### Simple tuner (NAF, Reed)

Searches for the frequency where the imaginary part of input impedance crosses zero (a reactance resonance). Uses a bracket search to find the crossing, then Brent-Dekker for precise root finding. The bracket preference logic matches Java's `PlayingRange.findBracket()` exactly — when the primary-direction bracket exceeds `PreferredSolutionRatio`, the fallback direction is preferred unconditionally.

### LinearV tuner (Whistle, Flute)

The Auvray (2012) model computes a gain factor from mouthpiece geometry (beta, windway height, window dimensions). The tuner finds the frequency that minimizes `|fmin - freq|` where fmin accounts for the Strouhal number and airstream velocity at the given blowing level. Includes a gain check: steps down while `gain >= 1.0 && ratio < minRatio`, matching Java `PlayingRange.findFmin()`.

## Impedance Pipeline

```
  termination (open end)
       ↑
  bore section N  ←  transfer matrix multiplication
       ↑
  hole N          ←  T-network shunt
       ↑
  bore section N-1
       ↑
     ...          ←  walk components in reverse order
       ↑
  bore section 0
       ↑
  mouthpiece      ←  model-specific (headspace, fipple, embouchure, reed)
       ↓
  Z = P/U         ←  input impedance
```

1. Initialize state vector at termination (ThickFlanged or Unflanged radiation impedance)
2. Walk components in reverse (termination → mouthpiece), applying transfer matrices
3. Apply mouthpiece model:
   - **DefaultFipple** (NAF): headspace TMs + fipple factor scaling + window impedance
   - **SimpleFipple** (Whistle/Flute): empirical Xw/Rw + headspace parallel combination
   - **SimpleReed** (Reed): linear reactance `X = α×10⁻³×f + β`, pressure-node boundary
4. Return Z = P/U

## Dependencies

- `wid-math` — TransferMatrix and StateVector
- `wid-physics` — PhysicalParameters
- `wid-types` — Fingering type
- `wid-compile` — InstrumentCompiled and Component types
- `wid-acoustics` — bore, hole, termination, mouthpiece calculators
- `num-complex` — Complex64

## Tests

12 unit tests + 19 integration tests:

**Unit tests**: Brent solver (5), calculator params (2), Z-samples (1), evaluation (1), error vector (1), cents utility (2).

**Integration tests** (all validated against Java oracle golden fixtures):
- `bulk_naf_eval.rs` (4): All 36 NAF combos (540 fingerings). Max error: 0.000003 cents.
- `bulk_whistle_eval.rs` (4): All 16 Whistle combos (272 fingerings). Max error: 0.000002 cents.
- `whistle_z_sample.rs` (1): Z-sample parity for SamplePVC-Whistle.
- `bulk_flute_eval.rs` (4): All 8 Flute combos (110 fingerings). Max error: 0.058 cents.
- `flute_z_sample.rs` (1): Z-sample parity for SamplePVC-Flute (exact match).
- `bulk_reed_eval.rs` (4): All 7 Reed combos (72 fingerings). Max error: 0.000011 cents.
- `reed_z_sample.rs` (1): Z-sample parity for SampleChanter-Reed.
