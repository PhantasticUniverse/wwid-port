# Architecture

## Data flow

```
Instrument XML ──► InstrumentRaw ──► compile() ──► InstrumentCompiled
                       (wid-types)      (wid-compile)      │
                                                           ▼
Tuning XML ──► Tuning ──────────────────────────► calc_z(freq, fingering)
                (wid-types)                          (wid-eval)
                                                       │
                                                       ▼
PhysicalParameters ─────────────────────────► impedance Z(f)
   (wid-physics)                                 │
                                                 ▼
                                         Im(Z) = 0 root finding
                                           (Brent-Dekker)
                                                 │
                                                 ▼
                                         predicted frequency
                                                 │
                                                 ▼
                                     1200 × log₂(pred/target)
                                         = cents deviation
```

### Pipeline summary

1. **Parse** — `wid-types` deserializes WIDesigner XML into `InstrumentRaw` and `Tuning`
2. **Compile** — `wid-compile::compile()` converts raw geometry to metres, sorts bore points, extracts headspace, interleaves bore sections with toneholes
3. **Impedance** — `wid-eval::calc_z()` walks the compiled component chain from termination to mouthpiece, cascading transfer matrices through the state vector
4. **Root finding** — `predicted_frequency()` brackets and solves Im(Z) = 0 using Brent-Dekker
5. **Evaluation** — `calculate_error_vector()` computes cents deviation per fingering

---

## Crate-by-crate guide

### wid-math

**Role**: Foundation types for acoustic transfer matrix calculations.

| Type / Function | Description |
|----------------|-------------|
| `TransferMatrix` | 2×2 complex matrix (`pp`, `pu`, `up`, `uu`). `Copy`. |
| `TransferMatrix::multiply()` | Matrix-matrix product |
| `TransferMatrix::multiply_sv()` | Matrix-vector product |
| `StateVector` | Pressure `p` + volume-flow `u` pair. `Copy`. |
| `StateVector::from_impedance()` | Dickens (2007) normalization for numerical robustness |
| `StateVector::impedance()` | Z = P/U |
| `StateVector::series()` / `parallel()` | Impedance combination on state vectors directly |

**Invariants**: Both types are `Copy` — no heap allocation in the inner loop. Determinant of physical transfer matrices should be ~1.

**Tests**: 22 unit tests (algebra, boundary cases, composition).

---

### wid-physics

**Role**: Air properties and acoustic wave parameters from environmental conditions.

| Type / Function | Description |
|----------------|-------------|
| `PhysicalParameters` | Full CIPM-2007 moist air model |
| `PhysicalParameters::calc_wave_number()` | k = 2πf/c |
| `PhysicalParameters::calc_z0()` | Bore characteristic impedance ρc/πr² |
| `PhysicalParameters::get_complex_wave_number()` | Lossy propagation constant with viscothermal damping |
| `SimplePhysicalParameters` | Polynomial approximation used by fipple mouthpiece |
| `TemperatureType` | °C or °F input |

**Academic references**:
- CIPM-2007 (Picard et al. 2008) for density, compressibility, vapour pressure
- Tsilingiris (2008) for viscosity/conductivity mixing rules
- Owen Cramer (JASA 1993) for speed of sound polynomial

**Default conditions**: 72 °F, 101.325 kPa, 45% RH, 390 ppm CO₂ — matches the NAF study model defaults.

**Tests**: 20 unit tests, all validated against golden `internals_0.json` to 12+ digit precision.

---

### wid-types

**Role**: Serde structs mapping directly to the WIDesigner XML schema.

| Type / Function | Description |
|----------------|-------------|
| `InstrumentRaw` | Top-level instrument (name, bore points, holes, mouthpiece, termination) |
| `Tuning` | Note targets + fingering patterns |
| `Fingering` | Note + open-hole pattern + optimization weight |
| `LengthType` | Unit system (inches, cm, mm, m) with `to_metres()` |
| `parse_instrument_xml()` | Deserialize instrument from WIDesigner XML |
| `parse_tuning_xml()` | Deserialize tuning from WIDesigner XML |
| `strip_xml_namespaces()` | Remove `ns2:` prefix before deserialization |

**Gotcha**: WIDesigner XML uses a namespace prefix on the root element (`<ns2:instrument xmlns:ns2="...">`), but child elements are unqualified. The namespace must be stripped before serde can parse it.

**Tests**: 17 tests (parsing oracle XMLs, namespace stripping, unit conversion, constraints parsing for all 16 NAF constraint XMLs across 8 objective function types).

---

### wid-compile

**Role**: Convert `InstrumentRaw` → `InstrumentCompiled` — the explicit compile step that prevents "forgot to call updateComponents" bugs. Also provides geometry mutation functions for the optimization loop.

| Type / Function | Description |
|----------------|-------------|
| `compile()` | Main entry point: raw → compiled |
| `InstrumentCompiled` | Component chain + mouthpiece + termination |
| `Component` | `Bore(BoreSection)` or `Hole(CompiledHole)` |
| `BoreSection` | Length + left/right radius (metres) |
| `CompiledHole` | Position, diameter, height, interpolated bore diameter |
| `CompiledMouthpiece` | Position, headspace sections, mouthpiece type |
| `MouthpieceType` | `Fipple { ... }` or `EmbouchureHole { ... }` |
| `get_hole_geometry_from_top()` | Extract geometry vector (metres) for HoleFromTop optimizer |
| `set_hole_geometry_from_top()` | Apply geometry vector to `InstrumentRaw` |
| `get_fipple_factor()` / `set_fipple_factor()` | Read/write fipple factor on instrument |

**Compilation steps**:
1. Convert all dimensions to metres
2. Sort bore points by position (ascending)
3. Extract headspace: bore sections above the mouthpiece position
4. Sort holes by position, interleave bore sections between them
5. Set termination diameter from last bore point

**Invariants**:
- Components alternate `Bore, Hole, Bore, Hole, ..., Bore`
- Holes are in ascending position order
- All bore sections have positive length (≥ `MINIMUM_CONE_LENGTH`)
- Headspace ends exactly at mouthpiece position

**Tests**: 22 tests (component count, ordering, interpolation, validation, geometry get/set round-trip, fipple factor mutation). Component count (13) and headspace sections (1) match golden values for the 6-hole NAF.

---

### wid-acoustics

**Role**: Transfer matrix and state vector calculations for each acoustic element.

| Module | Model | Description |
|--------|-------|-------------|
| `tube` | Lossy cylinder/cone TMs | Viscothermal losses via complex propagation constant. Radiation impedance via Padé approximants (Silva et al. 2008). |
| `bore` | Bore section TM | Delegates to `tube::calc_cone_matrix()` with section geometry |
| `hole` | Tonehole T-network | Lefebvre & Scavone (2012): series impedance Za + shunt admittance Ys. Open vs closed behaviour. |
| `termination` | Thick-flanged open end | Reflection coefficient model with flange diameter correction |
| `mouthpiece` | Fipple (NAF) mouthpiece | Headspace volume (×4 end correction), fipple factor scaling, window impedance, radiation resistance |

**NAF-specific constants**:
- `NAF_HOLE_SIZE_MULT = 0.9605` — empirical hole size correction
- `AIR_GAMMA = 1.4018...` — hardcoded adiabatic index (differs from CIPM-2007 derived value)
- Headspace ×4: headspace bore sections are applied 4× in the transfer matrix chain (2× physical + 2× end correction)
- `DEFAULT_WINDWAY_HEIGHT = 0.00078740 m` — fallback when XML omits windway height

**Academic references**:
- Lefebvre & Scavone (2012) for tonehole model
- Silva et al. (2008) for radiation impedance Padé approximants
- Lefebvre & Kergomard for lossy conical tube formulation

**Tests**: No unit tests — validated entirely through `wid-eval` integration tests against golden Z-samples and evaluation results.

---

### wid-eval

**Role**: Top-level impedance pipeline, frequency prediction, and evaluation.

| Type / Function | Description |
|----------------|-------------|
| `calc_z()` | Input impedance at a frequency for a fingering |
| `calc_z_samples()` | Impedance at multiple frequencies |
| `predicted_frequency()` | Playing frequency via Im(Z)=0 root finding |
| `calculate_error_vector()` | Cents deviation per fingering |
| `cents()` | 1200 × log₂(predicted/target) |

**Impedance pipeline** (`calc_z`):
1. Initialize state vector at termination
2. Walk components in reverse (termination → mouthpiece), applying transfer matrices
3. Apply mouthpiece transfer matrix (headspace + fipple)
4. Return Z = P/U

**Root finding**: Bracket search + Brent-Dekker solver for Im(Z) = 0 crossings.
- `SEARCH_BOUND_RATIO = 2.0` (within an octave)
- `PREFERRED_SOLUTION_RATIO = 1.12` (~200 cents)
- `GRANULARITY = 0.012` (~20 cents step)
- Bracket preference logic matches upstream exactly: when primary-direction bracket is outside preferred ratio, fallback direction is preferred unconditionally

**Tests**: 8 unit tests (Z-samples against golden, evaluation within 0.5 cents for all 15 fingerings, error vector, cents utility) + 4 integration tests (bulk evaluation of all 36 NAF instrument×tuning combinations — 540 fingerings — against golden oracle).

---

### wid-optimize

**Role**: Calibration and optimization for NAF instrument design.

| Type / Function | Description |
|----------------|-------------|
| `calibrate_fipple()` | 1D fipple factor calibration using Brent minimizer |
| `optimize_holes()` | Multi-variable hole geometry optimization using BOBYQA |
| `calc_norm()` | Weighted L2 norm (sum of weighted squared errors) |
| `fingering_weights()` | Extract optimization weights from fingerings |
| `CalibrationResult` | Initial/final fipple factor and norm |
| `OptimizationResult` | Initial/final norm, geometry, evaluation count |

**Fipple calibration** (`fipple.rs`):
- Uses only the lowest-frequency fingering (matching Java `getLowestNote`)
- 1D Brent optimizer (golden section + parabolic interpolation)
- Objective: compile → evaluate → weighted norm
- Trust region: 10.0 initial, 1e-8 stopping

**Hole optimization** (`hole_from_top.rs`):
- 13-dimensional for 6-hole NAF: `[bore_length, top_hole_fraction, 5 spacings, 6 diameters]`
- BOBYQA optimizer with bounds from constraints XML
- Trust region: 10.0 initial, 1e-8 stopping, max_eval = 20000 + (n-1)×5000, n_interp = 2n+1
- Objective: set geometry → compile → evaluate → weighted norm

**Dependencies**: `bobyqa`, `wid-eval`, `wid-compile`, `wid-physics`, `wid-types`

**Tests**: 20 tests — Brent minimizer (4), fipple calibration against NAF-FF-02/03 golden (6), hole optimization against NAF-OPT-01/02 golden (7), norm calculation (3).

---

### bobyqa

**Role**: Standalone pure-Rust implementation of Powell's BOBYQA algorithm (Bound Optimization BY Quadratic Approximation). Zero dependencies, suitable for WASM.

Ported from Apache Commons Math 3.6.1 `BOBYQAOptimizer`. The Fortran GOTO control flow is mapped to a Rust state machine: `loop { match state { 20 | 60 | 90 | ... } }`.

**Public API**: `bobyqa_minimize(f, initial_point, lower_bounds, upper_bounds, n_interp, initial_trust, stopping_trust, max_eval) → Option<BobyqaResult>`

**Tests**: 32 unit tests + 1 doc test — 2D quadratics, Rosenbrock 2D/13D, ACM3 13D suite (sphere, cigar, tablet, cigtab, two_axes, elli, ackley, rastrigin), DiffPow 6D, Powell's "points in square", edge cases. 4 evaluation-count-exact matches with Apache Commons Math.

---

## Testing strategy

### Golden fixtures

Tests consume golden fixtures from `golden/expected/`. The golden harness (Java CLI in `golden-harness/`) generates these by running the oracle WIDesigner v2.6.0 JARs.

### Tolerance definitions

| Layer | Tolerance |
|-------|-----------|
| Z-samples | `abs_err ≤ 1.0 + 1e-6 × max(\|expected\|, \|actual\|)` |
| Evaluation (cents) | ≤ 0.5 cents per fingering |
| Predicted frequency | ≤ 0.01% relative error |
| Physics constants | 12+ digit match against oracle internals |
| Calibration (fipple factor) | ≤ 1e-6 absolute, norm within 1% |
| Optimization (geometry) | ≤ 5e-3 metres per element, norm ≤ oracle × 1.2 |

### How tests consume fixtures

Tests use `include_str!()` to embed golden JSON at compile time. Each test deserializes the expected values, runs the equivalent Rust pipeline, and asserts within tolerance. No network or filesystem access at test time.

---

## Parity methodology

1. **Oracle**: WIDesigner v2.6.0 release JARs are the source of truth
2. **Golden harness**: Java CLI generates fixture outputs by wiring the oracle classes exactly as the study model does
3. **Layered testing**: Physics → types → compile → acoustics → eval, each layer validated before building on it
4. **Tolerance-based comparison**: Cents-based for frequencies, abs+rel for impedance, exact for structural properties (component count, ordering)
5. **No approximation**: We reproduce baseline behavior, not approximate it
