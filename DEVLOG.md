# Development Log

## Summary

| Metric | Value |
|--------|-------|
| **Total tests** | 280 |
| **Study models complete** | 4/4 (NAF, Whistle, Flute, Reed) |
| **Milestones done** | M0–M4 complete, M5 in progress |
| **Evaluation parity** | ≤ 0.058 cents across 994 fingerings |
| **Crates** | 11 (math, physics, types, compile, acoustics, eval, optimize, session, wasm, bobyqa, direct) |
| **Golden fixtures** | 14 scenarios |

### Entries (newest first)

- [DIRECT-C Stress Test + Golden Fixtures](#2026-03-04-cont-direct-c-global-optimizer--stress-test--golden-fixtures) — 6 bugs fixed, DIRECT-01 fixture
- [DIRECT-C + Multi-Start Infrastructure](#2026-03-04-cont-direct-c-global-optimizer--multi-start-infrastructure) — direct crate, multi-start, two-stage pipeline
- [M5.7 Reed Calibration + Optimization](#2026-03-04-cont-m57-reed-calibration--optimization) — reed calibrator, 3 hole optimizers
- [M5.5/M5.6 Parity Tests + Docs](#2026-03-04-cont-m55m56-parity-tests--documentation-sweep) — flute parity tests, FminmaxEvaluator fix
- [M5.6 Reed finger_adjustment Bug](#2026-03-04-cont-m56-reed-parity-bug-fix--finger_adjustment) — finger_adjustment=0.010 fix
- [M5.5 + M5.6 Flute Calibration + Reed Eval](#2026-03-04-m55--m56--flute-calibration--reed-evaluation) — flute calibrators, reed mouthpiece model
- [M5.4 Whistle Calibration + Optimization](#2026-03-04-m54--whistle-calibration--optimization) — 3 calibrators, 3 hole optimizers
- [M5.3 Flute Evaluation Parity](#2026-03-02-m53--flute-evaluation-parity) — embouchure hole, 0.058 cents
- [M5.2 Whistle Evaluation Parity](#2026-03-02-m52--whistle-evaluation-parity) — LinearV tuner, 0.000002 cents
- [M5.1 Study Model Infrastructure](#2026-03-02-m51--study-model-infrastructure-refactor) — CalculatorParams, TerminationType
- [Phase 4f Bug Fixes + Polish](#2026-03-02-phase-4f--bug-fixes--polish)
- [Phase 4e Optimization UI](#2026-03-02-phase-4e--optimization--calibration-ui)
- [M1 Golden Harness](#2026-03-02-m1--golden-harness--naf-fixture-suite)
- [M2 NAF Evaluation Parity](#2026-03-02-m2--naf-evaluation-parity-in-rust) — 76 tests, 0.5 cents
- [M3 NAF Calibration + Optimization](#2026-03-02-m3--naf-calibration--optimization-parity) — BOBYQA crate, 139 tests
- [NAF Bulk Test Coverage](#2026-03-02-expanded-naf-test-coverage-all-oracle-xmls) — 36 combos, 540 fingerings
- [M4 Browser MVP](#2026-03-02-m4--browser-hosted-mvp-naf-end-to-end)

---

## 2026-03-04 (cont): DIRECT-C Global Optimizer — Stress Test + Golden Fixtures

### Stress test findings (6 bugs fixed)
1. **Centre rectangle potential sign** (Critical): DIRECT-1 potential was `centre_f - min_neighbor`, should be `min_neighbor - centre_f`
2. **Grid spacing formula** (Important): Used cell-centre `(i+0.5)/N`, Java uses interior-point `(i+1)/(N+1)`
3. **Trust radius** (Important): Was hardcoded 10.0, now computed from bounds matching Java `BaseObjectiveFunction.getInitialTrustRegionRadius()`
4. **Flute missing Global optimizers** (Critical): Added `GLOBAL_HOLE` + `GLOBAL_HOLE_POSITION` to flute.rs (Java inherits from WhistleStudyModel)
5. **Progress/cancel support** (Important): Added `_with_progress` variants through full pipeline
6. **Geometry dimension assertion** (Minor): `set_merged_geometry` now asserts on length mismatch

### Golden fixture: DIRECT-01
- `DirectOptDriver.java`: Runs `GlobalHoleObjectiveFunction` + `GlobalHolePositionObjectiveFunction` on SamplePVC-Whistle through `ObjectiveFunctionOptimizer.optimizeObjectiveFunction()`
- GlobalHole: norm 15900 → 1899 (13 dims, 9074 evals)
- GlobalHolePosition: norm 15900 → 20832 (7 dims, 7815 evals — DIRECT found poor basin)
- 6 Rust parity tests: initial norm match, significant improvement, both stages used, correct geometry dimensions, geometry roundtrip, progress callback

**Test count**: 280 tests, all passing

---

## 2026-03-04 (cont): DIRECT-C Global Optimizer + Multi-Start Infrastructure

### New crate: `direct` (DIRECT-C global optimizer)

Clean-room Rust implementation of the DIRECT-C algorithm (DIviding RECTangles, Centred variant) for derivative-free global optimization over box constraints.

**Algorithm layers**:
1. **Base DIRECT** (Jones 1993): BTreeMap-sorted hyperrectangles, convex hull POH selection, trisection along longest sides
2. **DIRECT-1** (Gablonsky 2001): Single-side division for non-hypercubes, per-dimension potential tracking with half-decay
3. **DIRECT-C** (Patkau/WIDesigner): Variant selection strategies when standard POH stagnates — Large & Near (diameter × distance hull) and Low Value & Near (50 exponential distance bins × f-value hull)

**Key implementation details**:
- `RectKey` ordering: (diameter, f_value, serial) in BTreeMap for efficient convex hull queries
- Diameter rounded to f32 for grouping (matches Java/NLopt)
- Variant cycling pattern: `[2, 1, 1, 2, 1]` with stagnation detection
- 19 tests: Rosenbrock (2D, 5D), six-hump camel, Rastrigin, Styblinski-Tang, Goldstein-Price, sphere (1D/2D/5D), target value, callback, edge cases

### Multi-start infrastructure (`wid-optimize/src/multi_start.rs`)

- `random_start_points()` — xoshiro256** PRNG for reproducible sampling
- `grid_start_points()` — deterministic grid with `ceil(n^(1/d))` per dimension
- `multi_start_bobyqa()` / `multi_start_bobyqa_with_progress()` — run BOBYQA from N start points, keep best
- 7 tests: bounds, reproducibility, grid layout, improvement, progress, cancellation

### Two-stage pipeline (`wid-optimize/src/global_optimize.rs`)

Port of Java `ObjectiveFunctionOptimizer.runDirect()` + `runBobyqa()`:
- Stage 1: DIRECT-C global search (2× budget, convergence 7e-8, target value 0.001)
- Stage 2: BOBYQA local refinement from DIRECT-C's best point
- Returns whichever stage found better result
- Instrument-level wrappers: `optimize_global_holes_combined()` (40K evals), `optimize_global_holes_position()` (30K evals)
- 3 tests: Rosenbrock, six-hump camel, sphere 5D

### Session integration

- Whistle: registered `GlobalHolePositionObjectiveFunction` + `GlobalHoleObjectiveFunction`
- Reed: registered `GlobalHoleObjectiveFunction`
- Flute: registered `GlobalHolePositionObjectiveFunction` + `GlobalHoleObjectiveFunction`
- Dispatch in `optimize()` routes Global keys to DIRECT-C→BOBYQA pipeline with progress support
- Constraint templates delegate to parent (same geometry layout)

---

## 2026-03-04 (cont): M5.7 Reed Calibration + Optimization

### Reed calibration (alpha + beta)

Implemented `ReedCalibratorObjectiveFunction` port — 2D BOBYQA optimizing `[alpha, beta]` jointly using CentDeviationEvaluator (NOT FminmaxEvaluator like Whistle/Flute).

**Key difference from Whistle/Flute calibrators**: Reed uses `calculate_error_vector()` (cent deviation) while Whistle/Flute joint calibrators use `calculate_fminmax_error_vector()` (fmin+fmax combined).

**New files**:
- `wid-compile`: `get_alpha()`, `set_alpha()` — type-agnostic alpha dispatch (single_reed → double_reed → lip_reed)
- `wid-optimize/reed_calib.rs` — 2D BOBYQA calibrator with default bounds [0, 10] × [0, 10]
- `wid-session/reed.rs` — Reed optimizer registry (4 optimizers: calibrator + 3 hole optimizers)
- `golden-harness/ReedCalibDriver.java` — golden fixture generator

**Session dispatch**: 8 points in `wid-session/src/lib.rs` updated for `StudyKind::Reed`:
- `mod reed`, `available_optimizers()`, `can_optimize()`, `calibrate()`, `optimize()` (calibrator check + hole dispatch), `create_default_constraints()`, `create_blank_constraints()`

**CalibResult**: Added `initial_alpha`/`final_alpha` fields (all 7 existing constructors updated with `None`).

**Golden fixture**: SampleChanter.xml + A3-ClosedFingering.xml
- Initial: alpha=1.8, beta=0.09, norm=54.76
- Final: alpha=1.777, beta=0.095, norm=26.49

**Parity tests** (2 new, 242 total):
- `initial_norm_matches_golden` — within 1% of 54.76
- `reed_calibration_matches_golden` — final norm < 26.49 × 1.1, alpha/beta within 10%

---

## 2026-03-04 (cont): M5.5/M5.6 Parity Tests + Documentation Sweep

### Flute parity tests (14 new tests, 240 total)

Added golden-fixture-backed parity tests for all Flute calibrators and hole optimizers:

**Calibrator tests** (6 new):
- `airstream_length.rs`: 5 tests — initial AL extraction, roundtrip, fmax norm is zero for frequency-only tuning, calibration preserves zero norm, fife smoke test
- `flute_calib.rs`: 3 tests — initial fminmax norm matches golden (2649.61), joint calibration matches golden (norm < 1313 × 1.1, beta within 10%), fife smoke test
- `beta.rs`: 1 test — fmin norm is zero for frequency-only tuning (D4-Equal has no `<frequencyMin>` targets)

**Hole optimizer tests** (9 new, in `mod flute_tests`):
- `hole_size.rs`: 3 tests — initial norm, optimization golden (norm < 1081 × 2.0), fife smoke
- `hole_position.rs`: 3 tests — initial norm, optimization golden (norm < 1594 × 2.0), fife smoke
- `hole_combined.rs`: 3 tests — initial norm, optimization golden (norm < 1202 × 2.0), fife smoke

### Evaluator bug fix: FminmaxEvaluator frequency-only branch

**Root cause**: `calculate_fminmax_error_vector()` used `predicted_fmax` (resonance frequency from Im(Z)=0 crossing) for the frequency-only fallback, but Java's `FminmaxEvaluator` uses `predicted.getFrequency()` — the actual playing frequency from the LinearV tuner.

**Fix**: Changed frequency-only branch to use `predicted_frequency_linear_v()`, matching Java. Added `FPLAYING_WEIGHT = 1.0` constant. The fmax/fmin evaluators correctly return 0.0 for fingerings without explicit `frequencyMax`/`frequencyMin` targets (no fallback needed — matches golden norms of 0.0).

### Documentation sweep

Updated all project documentation to reflect M5.5/M5.6 completion:
- Created `wid-session/README.md` (new)
- Rewrote `wid-optimize/README.md` to cover all study models
- Updated `ARCHITECTURE.md` with Reed, updated test counts
- Updated `wid-eval/README.md`, `wid-acoustics/README.md`, `wid-compile/README.md`
- Updated `PORT_SPEC.md` milestones (M4 + M5.1–M5.6 marked complete)
- Updated `API_SHAPE.md` status, `FEATURE_MATRIX.md` Reed row

---

## 2026-03-04 (cont): M5.6 Reed Parity Bug Fix — finger_adjustment

**Root cause**: `CalculatorParams::REED` had `finger_adjustment: 0.0` but Java's `SimpleReedCalculator` uses `new DefaultHoleCalculator()` (default no-arg constructor) which sets `fingerAdjustment = DEFAULT_FINGER_ADJ = 0.010`.

The subtlety: Java's `DefaultHoleCalculator` has overloaded constructors with different defaults:
- `DefaultHoleCalculator()` → fingerAdjustment = **0.010**
- `DefaultHoleCalculator(holeSizeMult)` → fingerAdjustment = **0.0**

NAF uses the 1-arg constructor `DefaultHoleCalculator(0.9605)`, so fingerAdjustment = 0.0. We assumed Reed was similar, but Reed uses the no-arg constructor.

**Impact**: Each closed hole's shunt admittance was slightly wrong (`tf = r²/fingerAdj` was missing). Through 8 closed holes on SampleChanter, the cumulative error shifted the Im(Z)=0 crossing by ~0.3 Hz, producing ~2.8 cents error on the first fingering.

**Fix**: Changed `finger_adjustment` from `0.0` to `0.010` in `CalculatorParams::REED`. All 7 reed combos now within 0.012 cents. 226 tests passing.

**Debugging approach**: Wrote Java `ReedSVTraceDriver` to dump state vectors at each component, compared with Rust trace. First divergence appeared at the first closed hole (B4), confirming the hole TM calculation was the source.

**Lesson documented in `parity-notes.md`**: Always trace the exact Java constructor call for each calculator component — don't infer defaults from class name or documentation.

### Compilation fix: zero-length bore section at mouthpiece position

Fixed `process_position` in `wid-compile/src/lib.rs` to match Java's `addSection()` behavior: when a bore section has zero length (mouthpiece position coincides with a bore point), bump `right_bore_position` and the new bore point's position by `MINIMUM_CONE_LENGTH`. Without this, the stub section was incorrectly extracted to headspace, changing the component chain from 21 to 20 elements and producing a 0.012 cents residual error.

After fix: max Reed error dropped from 0.012 cents to **0.000011 cents** (~1e-5). Z-sample error dropped from ~1e-6 to **~1e-14** (machine epsilon). 226 tests all passing.

---

## 2026-03-04: M5.5 + M5.6 — Flute Calibration + Reed Evaluation

### M5.5: Flute Calibration + Optimization

Flute calibration and optimization pipeline complete. Two new calibrators and session dispatch for 6 total Flute optimizers (3 calibrators + 3 hole optimizers).

#### Calibrators

1. **AirstreamLengthObjectiveFunction** (1D Brent) — calibrates embouchure hole airstream length using FmaxEvaluator.
2. **FluteCalibrationObjectiveFunction** (2D BOBYQA) — jointly optimizes airstream length and beta using FminmaxEvaluator.
3. **BetaObjectiveFunction** (1D Brent) — reused from Whistle, calibrates beta using FminEvaluator.

#### Hole Optimizers

4. **HoleSizeObjectiveFunction** — N-dim BOBYQA for hole diameters (reused from Whistle)
5. **HolePositionObjectiveFunction** — (N+1)-dim BOBYQA for bore length + spacings (reused from Whistle)
6. **HoleObjectiveFunction** — (2N+1)-dim merged BOBYQA (reused from Whistle)

#### New files

- `wid-optimize/src/airstream_length.rs` — 1D Brent airstream length calibrator
- `wid-optimize/src/flute_calib.rs` — 2D BOBYQA joint calibrator (airstream + beta)
- `wid-session/src/flute.rs` — optimizer registry + constraint templates
- `wid-compile/src/lib.rs` — added `get_airstream_length()` / `set_airstream_length()`
- `CalibResult` extended with `initial_airstream_length` / `final_airstream_length` fields

#### Java golden harness (pending fixture generation — requires Java 17+)

- `FluteCalibDriver.java` — generates FL-CAL fixtures (3 calibrators)
- `FluteOptDriver.java` — generates FL-OPT fixtures (3 hole optimizers)

### M5.6: Reed Evaluation

Reed mouthpiece model and evaluation dispatch implemented. Reed instruments (single reed, double reed, lip reed) now compile and evaluate.

#### Reed mouthpiece model

Simple linear reactance model matching Java `SimpleReedMouthpieceCalculator`:
- `X = alpha × 1e-3 × freq + beta`
- Transfer matrix: `[[0+iX, z₀], [1, 0]]` (pressure-node boundary condition)
- Lip reeds negate beta sign
- Uses `SimpleInstrumentTuner` (standard reactance-zero search, NOT LinearV)

#### New/modified files

- `wid-acoustics/src/simple_reed.rs` — reed transfer matrix calculation
- `wid-compile/src/lib.rs` — added `MouthpieceType::SimpleReed { alpha, is_lip_reed }`, reed compilation for single/double/lip reed, `beta` field on `CompiledMouthpiece`
- `wid-eval/src/calculator_params.rs` — added `MouthpieceModel::SimpleReed`, `CalculatorParams::REED`
- `wid-eval/src/lib.rs` — reed dispatch in `calc_z()` and `calculate_error_vector()`
- `wid-eval/src/linear_v.rs` — added `SimpleReed` arm (unreachable — reed doesn't use LinearV)
- `wid-acoustics/src/simple_fipple.rs` — added `SimpleReed` arm to `calc_z_window()` (unreachable)

#### Reed headspace behavior (parity note)

Reed instruments (e.g., SampleChanter) can have headspace bore sections (bore points above mouthpiece position). In Java, the `SimpleReedMouthpieceCalculator` inherits the default `calcStateVector()` which just multiplies `calcTransferMatrix() * boreState` — headspace is extracted from the component chain during `updateComponents()` but is **never used** by the reed mouthpiece calculator. Our Rust code matches this behavior: headspace is extracted during compile and stored on `mouthpiece.headspace`, but the `SimpleReed` arm in `calc_z()` applies the reed TM directly without walking headspace. This is intentional parity.

#### Java golden harness (pending fixture generation)

- `ReedBulkEvalDriver.java` — 7 compatible reed combos (4 SampleChanter + 1 ReiswigChanter + 2 Didgeridoo)
- `ReedZSampleDriver.java` — Z-sample for SampleChanter + A3-ClosedFingering

### Test results

- 217 tests, all passing (no new test files yet — golden fixtures need Java 17+ to generate)
- All existing NAF, Whistle, and Flute tests pass with zero regressions
- Flute calibrator unit tests: airstream length extraction, roundtrip, norm reduction, joint calibration

## 2026-03-04: M5.4 — Whistle Calibration + Optimization

Complete Whistle calibration and optimization pipeline. All three calibrators (WindowHeight, Beta, joint) and all three hole optimizers (HoleSize, HolePosition, HoleCombined) match Java oracle golden fixtures.

### Calibrators

1. **WindowHeightObjectiveFunction** (1D Brent) — calibrates mouthpiece window height using FmaxEvaluator error vector. FeadogMk1: WH 0.0029 → 0.00246
2. **BetaObjectiveFunction** (1D Brent) — calibrates mouthpiece beta using FminEvaluator error vector. FeadogMk1: beta 0.522 → 0.510
3. **WhistleCalibrationObjectiveFunction** (2D BOBYQA) — joint calibration using FminmaxEvaluator (FMAX_WEIGHT=4.0, FMIN_WEIGHT=1.0). FeadogMk1: WH+beta jointly optimized

### Hole Optimizers

4. **HoleSizeObjectiveFunction** — N-dim BOBYQA for hole diameters. SamplePVC-Whistle: norm 15900 → 5661
5. **HolePositionObjectiveFunction** — (N+1)-dim BOBYQA for bore length + inter-hole spacings with PRESERVE_TAPER. Norm 15900 → 4604
6. **HoleObjectiveFunction** — (2N+1)-dim merged BOBYQA. Norm 15900 → 1899

### New evaluator infrastructure

- `calculate_fmax_error_vector()` — cents(target.frequencyMax, predicted fmax) per fingering
- `calculate_fmin_error_vector()` — cents(target.frequencyMin, predicted fmin) per fingering
- `calculate_fminmax_error_vector()` — RMS of weighted fmax + fmin errors

### Bug fix: HolePosition geometry ordering

`get_hole_geometry_position()` used `push()` which produced bottom-to-top spacing order, but Java uses indexed `geometry[i+1]` which produces top-to-bottom order. Fixed to use indexed assignment, matching Java's `HolePositionObjectiveFunction.getGeometryPoint()`.

### Session integration

- `wid-session/src/whistle.rs` — optimizer registry (6 optimizers), constraint template generation
- CalibResult extended with optional window_height/beta fields (supports NAF fipple + Whistle calibration)
- Dispatch for calibrate/optimize/create_constraints by StudyKind::Whistle

### Bobyqa crate name shadowing fix

Rust 2024 edition couldn't resolve `use bobyqa::X` inside a module also named `bobyqa`. Fixed by renaming the Cargo dependency: `bobyqa_impl = { package = "bobyqa", path = "../bobyqa" }`.

### Test results
- 217 total tests, all passing (up from 198)
- 16 new Whistle optimization tests (3 calibrator + 6 hole optimizer + 7 evaluator/geometry)
- Golden fixtures: 6 new (WH-CAL/3 + WH-OPT/3)

## 2026-03-02: M5.3 — Flute Evaluation Parity

Flute (transverse) evaluation matches Java oracle within 0.058 cents across 110 fingerings (8 combos). This was the smallest evaluation milestone because Flute reuses virtually everything from Whistle — same tuner (LinearV), same termination (unflanged), same hole calculator, identical `CalculatorParams`.

### Only real code change

The `calc_z_window` function in `simple_fipple.rs` needed an `EmbouchureHole` match arm. Same Xw/Rw formulas, different parameter extraction:
- Fipple: `eff_size = sqrt(windowLength * windowWidth)`, window_height with fallbacks
- EmbouchureHole: `eff_size = sqrt(min(width, airstreamLength) * length)`, height directly

Also added `EmbouchureHole` arm to `window_length()` in `linear_v.rs` (returns `airstreamLength` for flutes, `windowLength` for whistles — matching Java `Mouthpiece.getAirstreamLength()`).

### Golden harness: CentDeviationEvaluator index-shift bug

The fife-tuning XML has fingerings with only `frequencyMin`/`frequencyMax` (no `frequency`). Java's `CentDeviationEvaluator.calculateErrorVector()` has a subtle bug: when a fingering lacks `getFrequency()`, the `index` counter doesn't increment, shifting subsequent error values. The Flute golden harness computes cents per-fingering to avoid this.

### Test results
- Z-sample: 17 fingerings, max error = 0.000000 (exact match)
- Bulk eval: 110 fingerings across 8 combos (2 instruments × 4 compatible tunings), max diff = 0.058 cents
- Predicted freq: max relative error = 3.36e-5
- Oracle scope: 2 instruments (SamplePVC-Flute, fife), 6 tunings (4 compatible with 6-hole instruments)
- All existing 156+ tests pass (zero regressions)

## 2026-03-02: M5.2 — Whistle Evaluation Parity

Full Whistle evaluation pipeline: predicted frequencies and cent deviations match the Java oracle within 0.000002 cents across all 272 fingerings (16 instrument-tuning combos).

### Three fundamental differences from NAF

1. **Mouthpiece**: Simple fipple (empirical window impedance + headspace parallel combination) vs NAF's default fipple (transfer matrix with compliance headspace)
2. **Predicted frequency**: LinearV tuner (Strouhal number / velocity interpolation) vs NAF's simple tuner (reactance zero)
3. **Termination**: Unflanged end (already in M5.1) vs NAF's thick-flanged

### New files
- `wid-acoustics/src/simple_fipple.rs` — SimpleFippleMouthpieceCalculator port (window impedance Xw/Rw, headspace transmission model, parallel combination with bore SV)
- `wid-eval/src/linear_v.rs` — LinearVInstrumentTuner port (Strouhal model, blowing level tables, velocity interpolation, z-ratio root-finding, gain-aware findFmin)
- `golden-harness/.../WhistleBulkEvalDriver.java` — 2 instruments × 8 tunings = 16 combos
- `golden-harness/.../WhistleZSampleDriver.java` — Z-sample for SamplePVC-Whistle

### Key implementation details
- `MouthpieceModel` enum dispatches between DefaultFipple (NAF) and SimpleFipple (Whistle) in `calc_z`
- `gain_factor` computed at compile time from beta, windwayHeight, windowLength, windowWidth (Auvray, 2012)
- `find_fmin` includes gain check: steps down while `gain >= 1.0 && ratio < minRatio`, then returns `max(freqGain, freqRatio)` — matching Java PlayingRange.findFmin() exactly
- Brent minimization (golden-section + parabolic interpolation) added for finding Im(Z)/Re(Z) local minima
- `CalculatorParams::WHISTLE` = hole_size_mult=1.0, finger_adjustment=0.010, unflanged_end=true, SimpleFipple, blowing_level=5

### Test results
- Z-sample parity: 17 fingerings, max err near machine precision
- Bulk eval: 272 fingerings across 16 combos, max diff = 0.000002 cents
- Predicted freq: max relative error = 9.50e-10
- All existing NAF tests pass (zero regressions)
- **Total: 148 tests** across core crates (139 existing + 9 new Whistle tests)

---

## 2026-03-02: M5.1 — Study Model Infrastructure Refactor

Infrastructure-only refactor to parameterize NAF-specific code for multi-model support. No behavioral changes — NAF works identically before and after.

### New types
- Added `Whistle`, `Flute`, `Reed` variants to `StudyKind` enum
- Created `CalculatorParams` struct in `wid-eval` with per-study-model acoustic constants (hole_size_mult, finger_adjustment, unflanged_end)
- Added `TerminationType` enum (`ThickFlanged` / `Unflanged`) to `wid-acoustics`

### Parameterization
- Removed hardcoded `NAF_HOLE_SIZE_MULT` and `NAF_FINGER_ADJ` constants from `wid-eval`
- Added `calc_params: &CalculatorParams` parameter to: `calc_z`, `calc_z_samples`, `predicted_frequency`, `find_x_zero`, `calculate_error_vector`
- Propagated `calc_params` through `wid-optimize` (`calibrate_fipple`, `optimize_holes`, `optimize_holes_with_progress`)
- Added `calc_params` field to `StudySession`, resolved from `study_kind` in constructor

### Unflanged termination
- `calc_termination_sv` now accepts `TerminationType` parameter
- `Unflanged` path uses Silva 2008 Padé approximant (`tube::calc_z_load`) — already existed in codebase

### Session dispatch
- `available_optimizers()`, `can_optimize()`, `create_default_constraints()`, `create_blank_constraints()` now dispatch via `match self.study_kind`
- Non-NAF models return empty optimizer list / error for constraints (not yet implemented)

### WASM + Web UI
- `WasmSession::new()` accepts "NAF", "Whistle", "Flute", "Reed"
- Added `studyKind` signal and `switchStudyModel()` to session store
- Added study model dropdown in App.tsx header (non-NAF options disabled)

### Tests
- All 187 tests pass (184 existing + 3 new termination tests)
- New tests: `naf_params_match_old_constants`, `unflanged_produces_finite_result`, `unflanged_differs_from_thick_flanged`, `closed_end_ignores_termination_type`
- Zero TypeScript errors, clean Vite build

## 2026-03-02: Phase 4f — Bug Fixes + Polish

All fixes are frontend-only (no Rust/WASM changes). 184 tests still pass.

### Mouthpiece sync bug fix (CRITICAL)
- **Root cause**: `<div onFocusOut={sync} />` at line 166 of InstrumentEditor was a **sibling** of the mouthpiece grid, not a parent. An empty div can never receive focus, so focusout events from NumberFields never bubbled through it. All mouthpiece/fipple field edits were silently lost.
- **Fix**: Moved `onFocusOut={sync}` onto the `<section>` element wrapping the mouthpiece fields (matching how bore/hole tables use `<tbody onFocusOut={sync}>`). Deleted the dead `<div>`.

### Editor refresh after fipple calibration (CRITICAL)
- **Root cause**: `calibrate()` modifies the instrument in-place on the Rust side, but InstrumentEditor's `createEffect` only re-fetched when `props.docId` changed. After calibration, the editor showed stale fipple factor.
- **Fix**: Added `calibrationCount` signal to session store, incremented after successful calibration. InstrumentEditor watches `[props.docId, calibrationCount()]` — either changing triggers a re-fetch.

### refreshGating consistency
- Added `await refreshGating()` to `selectOptimizer()` and `selectConstraints()` (matching `selectInstrument`/`selectTuning` which already had it).
- Added `await refreshGating()` after hole optimization success to sync the Rust session's selection changes (new instrument_id) to the frontend.

### Auto-select on file open
- Opening an XML file now auto-selects the document in the sidebar via `selectInstrument`/`selectTuning`/`selectConstraints`. Previously users had to click the document after opening. Matches Java WIDesigner behavior.

### Worker error resilience
- Added `worker.onerror` handler in ComputeService. If the WASM module panics, all pending promises are rejected (instead of hanging forever) and state is cleaned up.

### Generic save/export for all doc types
- Replaced `saveInstrumentXml(docId)` with `saveDocXml(docId)` that looks up the document name from whichever doc list matches (instrument, tuning, or constraints).
- Added save/download button (&#x2913;) to each tab in the tab bar, next to the close button.
- Toolbar Save button now saves the active tab's document (any type) instead of only instruments.

### Stale error clearing + Sketch stub
- Added `setError(null)` at the start of all select functions so stale error banners clear on new selection.
- Added `onClick` to the Sketch button that logs "Sketch is not yet implemented (M5)" to the console panel (user feedback instead of dead button).

## 2026-03-02: Phase 4e — Optimization + Calibration UI

### Session store changes (`web/src/stores/session.ts`)
- Fixed `canOptimize` gating: fipple calibration no longer requires constraints
- Added `isFippleSelected`, `canCreateConstraints` memos
- Added `optimizing` / `optProgress` signals for live progress tracking
- Implemented `runOptimize()`: routes to `calibrate` (fipple) or `optimize` (hole) based on selection
  - Hole optimization: streams progress, adds new instrument to doc list, auto-selects + opens tab
  - Fipple calibration: sync command, modifies instrument in-place, logs before/after
  - Graceful cancellation handling (no error banner)
- Implemented `cancelOptimize()`: sends cancel signal to worker
- Implemented `createDefaultConstraints()` / `createBlankConstraints()`: creates constraints doc, auto-selects, opens editor tab

### OptimizeDialog component (`web/src/components/tools/OptimizeDialog.tsx`)
- Modal with two states: in-progress (spinner + live evaluations/norm) and result (before/after comparison)
- Type guard distinguishes CalibResult (fipple) from OptimizeResult (hole)
- Matches SettingsDialog overlay pattern

### StudyPanel wiring (`web/src/components/layout/StudyPanel.tsx`)
- Optimize button: dynamic label ("Calibrate" vs "Optimize"), disabled during active optimization
- Constraint creation: "+ Default" / "+ Blank" buttons, gated by `canCreateConstraints`
- Dialog lifecycle: opens on click, shows result on completion, closes on cancel

### Parity verification (browser, NAF-OPT-01 golden scenario)
- Initial norm: 1324815.0033 (golden: 1324815.0036) — within tolerance
- Final norm: 975.1391 (golden: 975.1391) — exact match
- Evaluations: 1750 vs golden 2018 — different BOBYQA path, same optimum

## 2026-03-02: M1 — Golden Harness + NAF Fixture Suite

### Gradle wrapper setup
- Generated Gradle 8.7 wrapper in `golden-harness/`
- Installed OpenJDK 17 and Gradle via Homebrew
- Verified `./gradlew build` compiles the empty project

### Java golden harness (5 classes)
- `Scenario.java` — Jackson POJO for scenario JSON format
- `OutputFormatter.java` — Deterministic JSON output with full f64 precision
- `ScenarioRunner.java` — Core orchestrator; wires NAFCalculator + SimpleInstrumentTuner + CentDeviationEvaluator exactly as NafStudyModel does
- `InstrumentOverrides.java` — Programmatic mutations for fipple factor / windway height null testing
- `GoldenHarnessMain.java` — CLI entry point with `--all` and per-scenario-id modes

Key wiring details:
- PhysicalParameters(72.0, TemperatureType.F) — matches NafStudyModel line 128
- NAFCalculator with DefaultHoleCalculator(0.9605) — matches NAFCalculator constructor
- CentDeviationEvaluator(calculator, tuner) — 2-arg constructor wires tuner from calculator state
- BoreLengthAdjustmentType.PRESERVE_BORE — matches NafStudyModel line 468

Added `--add-opens` JVM args to `build.gradle` for JAXB/Dozer compatibility on Java 17+.

### Scenario files (7 scenarios)
All scenario JSONs in `golden/scenarios/`:
- **NAF-FF-01**: Fipple scaling + null handling (4 eval variants + 2 zsample + internals dump)
- **NAF-FF-02**: Fipple calibration, 0-hole blank
- **NAF-FF-03**: Fipple calibration with holes
- **NAF-OPT-01**: Hole size+position optimization (BOBYQA)
- **NAF-OPT-02**: Weight=0 excludes a note from optimization
- **CONSTRAINTS-01**: Default constraints creation
- **CONSTRAINTS-02**: Blank constraints creation

### Support XMLs (3 files)
All in `golden/scenarios/support/`, derived from oracle originals:
- `NAF-FF-02_instrument_0hole.xml` — 0.75-bore starter with holes removed (all numeric values exact from oracle)
- `NAF-FF-02_tuning_0hole.xml` — single-note F#4 tuning for 0-hole instrument
- `NAF-OPT-02_tuning_weight0.xml` — F#4 chromatic tuning with G5(open) weight=0

### Fixture generation status
- NAF-FF-01: **generated and verified** — eval, zsample, and internals outputs all sane
- NAF-FF-02: **generated** — fipple calibration on 0-hole blank
- NAF-FF-03: **generated** — fipple calibration with holes
- NAF-OPT-01: **generated** — hole size + position optimization (BOBYQA)
- NAF-OPT-02: **generated** — weight=0 exclusion
- CONSTRAINTS-01: **generated** — default constraints creation
- CONSTRAINTS-02: **generated** — blank constraints creation

## 2026-03-02: M2 — NAF Evaluation Parity in Rust

### Workspace setup
- Rust workspace at `wid/` with 6 crates, edition 2024
- Shared deps: num-complex 0.4, quick-xml 0.37, serde 1, serde_json 1, approx 0.5

### Crates built (bottom-up)

**wid-math** (22 tests) — TransferMatrix (2x2 Complex64) + StateVector (P, U). Both `Copy`. Includes identity, multiply, impedance/admittance/reflectance, series/parallel, open/closed end constructors.

**wid-physics** (20 tests) — PhysicalParameters with full CIPM-2007 moist air model (temperature, pressure, humidity, CO2 → speed of sound, density, viscosity). SimplePhysicalParameters for fipple mouthpiece. All values validated against golden `internals_0.json` to 12+ digits.

**wid-types** (9 tests) — Serde structs for WIDesigner XML: InstrumentRaw, Tuning, Fingering, Note. XML namespace stripping (`ns2:` prefix). Parses oracle instrument and tuning files.

**wid-compile** (17 tests) — `compile(InstrumentRaw) → InstrumentCompiled`. Sorts bore points, extracts headspace, interleaves bore sections with holes, interpolates bore diameters. Component count (13) and headspace sections (1) match golden values.

**wid-acoustics** (0 unit tests, validated via wid-eval integration) — Tube (cylinder/cone TMs, radiation impedance), bore section TM, tonehole T-network (Lefebvre-Scavone 2012), thick-flanged termination, fipple mouthpiece (headspace ×4, scaled fipple factor, AIR_GAMMA=1.4018...).

**wid-eval** (8 tests) — Full impedance pipeline: `calc_z` walks components in reverse, `predicted_frequency` uses bracket search + Brent-Dekker root finding for Im(Z)=0 crossings. `calculate_error_vector` produces cents deviations.

### Parity results
- **Z-sample**: All 11 frequencies match golden within `abs_err ≤ 1.0 + 1e-6 × max(|expected|, |actual|)`
- **Evaluation**: All 15 fingerings within **0.5 cents** of golden oracle
- **76 tests total** across all crates, all passing

### Notable fix
Bracket search preference logic (`find_bracket`) needed to match Java's `PlayingRange.findBracket()` exactly — when the primary-direction bracket is outside `PreferredSolutionRatio`, the fallback direction is preferred unconditionally (not by distance comparison). Also uses `nearFreq²/bracket` as the search limit for the fallback, not `SearchBoundRatio`. This fixed fingering 14 (A5) which was finding the wrong Im(Z)=0 crossing.

## 2026-03-02: M3 — NAF Calibration + Optimization Parity

### Phase 1: Constraints types + XML parsing (wid-types)
- Added `Constraints`, `Constraint`, `ConstraintType` types in `wid-types/src/constraints.rs`
- XML deserialization with namespace stripping (reuses existing pattern)
- `lower_bounds()` / `upper_bounds()` preserve category-then-constraint order (ABI)
- Tested against CONSTRAINTS-01/02 golden fixtures

### Phase 2: Instrument mutation API (wid-compile)
- `get_hole_geometry_from_top()` — extract 13-element geometry vector (metres)
- `set_hole_geometry_from_top()` — apply geometry to InstrumentRaw
- `get_fipple_factor()` / `set_fipple_factor()` — read/write fipple factor
- Hole sorting by bore position (ascending), unit conversion (instrument units ↔ metres)
- Round-trip tested against NAF-OPT-01 golden initial geometry

### Phase 3: Brent minimizer (wid-optimize)
- New crate `wid-optimize` with `brent_min` module
- Port of Apache Commons Math `BrentOptimizer` (golden section + parabolic interpolation)
- 4 tests: quadratic, cosine, matching tolerances, start-at-boundary

### Phase 4: Fipple factor calibration (wid-optimize)
- `calibrate_fipple()` — 1D optimizer using Brent
- Uses only the lowest-frequency fingering (matching Java `getLowestNote`)
- NAF-FF-02 (0-hole): FF 0.75 → 0.266, norm 97743 → 0.0001
- NAF-FF-03 (6-hole): FF 0.75 → 0.274, norm 90010 → 0.0009
- Post-calibration eval within 1.0 cents on all fingerings

### Phase 5: BOBYQA optimizer + hole optimization

#### BOBYQA crate (standalone, open-source ready)
- New crate `crates/bobyqa/` — pure Rust, zero dependencies
- 1800 lines ported from Apache Commons Math 3.6.1 `BOBYQAOptimizer`
- Fortran GOTO labels → Rust `loop { match state { 20 | 60 | 90 | ... } }`
- State machine methods: `bobyqa`, `bobyqb`, `prelim`, `trsbox`, `altmov`, `update`
- 32 unit tests + 1 doc test:
  - 2D quadratics (bounded/unbounded), Rosenbrock 2D
  - ACM3 13D suite: sphere, cigar, tablet, cigtab, two_axes, elli, rosenbrock, ackley, rastrigin
  - DiffPow 6D, constrained Rosenbrock 13D
  - Powell's "points in square" (m=5 npt=16/21, m=10 npt=26/41)
  - Edge cases: tight bounds, minimum interp points, start at bound, asymmetric bounds
  - ACM3 parity: 4 exact (sphere=56, cigar=56, tablet=57, bounded_quadratic=27), 5 near-parity
- Java reference data generated from ACM3 via `BobyqaRef.java` + OpenJDK 17
- Comprehensive documentation: module docs, README.md, algorithm overview, usage examples

#### Hole optimization (wid-optimize)
- `optimize_holes()` — multi-variable optimization using BOBYQA
- Trust region: initial=10.0, stopping=1e-8, max_eval=20000+(n-1)*5000
- NAF-OPT-01 (all weight=1): norm 1324815 → 975, all 15 notes within ±16 cents
- NAF-OPT-02 (G5 weight=0): norm 1244615 → 964, fewer evaluations
- Geometry tolerance: all 13 elements within 5e-3 of golden

### Test count
- **139 tests** total across 8 crates (up from 76 in M2)
  - bobyqa: 32 + 1 doc test
  - wid-optimize: 20 (brent: 4, fipple: 6, hole_from_top: 7, lib: 3)
  - wid-eval: 8
  - wid-compile: 22
  - wid-types: 14
  - wid-math: 22
  - wid-physics: 20

## 2026-03-02: Expanded NAF Test Coverage (All Oracle XMLs)

### Bulk evaluation parity
- Created `NafBulkEvalDriver.java` to evaluate all 6 instruments × 6 tunings = 36 NAF combinations via the Java oracle
- All 36 combos evaluated successfully (540 total fingerings)
- Golden reference data committed to `golden/expected/NAF-BULK-EVAL/all_evals.json`

### Rust integration tests (wid-eval)
- 4 new integration tests in `wid/crates/wid-eval/tests/bulk_naf_eval.rs`
- All 540 fingerings match golden within **0.000003 cents** (max diff)
- Predicted frequency max relative error: **1.78e-9**
- Correctly handles null predictions for mismatched bore/tuning combos (e.g., tiny 0.5" bore with low B3 tuning)

### XML parsing coverage (wid-types)
- 3 new tests verify parsing of all 6 NAF instruments, all 6 NAF tunings, and all 16 NAF constraint XMLs
- Constraints tested across 8 objective function types: FippleFactor (0-hole, 6-hole), HoleFromTop (4 spacing variants), HoleGroupFromTop, NafHoleSize, SingleTaperHoleGroupFromTopHemiHead, SingleTaperHoleGroupFromTop, SingleTaperNoHoleGroupingFromTopHemiHead, SingleTaperNoHoleGroupingFromTop (5 spacing variants)

### Test count
- **146 tests** total (up from 139)
  - bobyqa: 32 + 1 doc test
  - wid-optimize: 20
  - wid-eval: 8 unit + 4 integration
  - wid-compile: 22
  - wid-types: 17 (up from 14)
  - wid-math: 22
  - wid-physics: 20

## 2026-03-02: M4 — Browser-Hosted MVP (NAF End-to-End)

### Phase 4a: wid-session crate
- Created `wid-session` crate with `StudySession` struct and JSON command dispatch
- Added `Serialize` derives to all wid-types structs (InstrumentRaw, Tuning, Constraints)
- Added `bobyqa_minimize_with_callback` to BOBYQA crate for progress/cancel support
- Threaded progress callback through wid-optimize's `optimize_holes()`
- Session API: open_xml, select_instrument/tuning/optimizer/constraints, evaluate_tuning,
  calibrate, optimize, export_xml, available_optimizers, get_params, get_selection
- Gating logic: canTune (instrument + tuning + hole count match), canOptimize (+ optimizer + constraints)
- Integration tests replaying golden scenarios through session API

### Phase 4b: WASM compilation + Web Worker + frontend

#### WASM crate
- Created `wid-wasm` crate (cdylib + rlib) with `WasmSession` struct
- JSON command dispatch: `execute(command_json)` for sync commands, `optimize(callback)` for async
- Pinned `wasm-bindgen="=0.2.100"` and `js-sys="=0.3.77"` to match installed wasm-bindgen-cli
  (v0.2.114 requires Rust 1.88, we have 1.86)
- Successful build: `cargo build --target wasm32-unknown-unknown --release -p wid-wasm`
- Generated JS glue via `wasm-bindgen --target web` (690KB WASM binary)

#### Web frontend
- Stack: Vite 6 + SolidJS 1.9 + Tailwind CSS v4
- `web/wasm` symlink (absolute path) → `wid/crates/wid-wasm/pkg/`
- Vite `@wasm` alias for WASM imports in worker context
- **compute-worker.ts**: Web Worker loading WASM, message-based command dispatch
- **ComputeService.ts**: Promise-wrapped worker API (init, run, optimize, cancel)
- **session.ts**: SolidJS reactive store synced from worker responses
- **App.tsx**: Study panel (instruments/tunings/optimizers/constraints lists),
  evaluation table with color-coded cents, console panel with physical parameters

#### End-to-end verification
- Opened instrument + tuning via drag-and-drop and file picker
- Study panel shows loaded documents, selection highlighting works
- Evaluation produces correct results matching golden fixtures
- Console output matches Java app format:
  `Properties of air at 20.00 C, 101.325 kPa, 45% humidity, 390 ppm CO2:`
  `Speed of sound is 343.787 m/s. Density is 1.1998 kg/m^3. Epsilon factor is 1.613e-03.`
- Color-coded cents: green (<5), amber (5-15), red (>15) — all working

#### Temperature default discrepancy (known, deferred)
The Java app's `OptimizationPreferences.DEFAULT_TEMPERATURE = 20` overrides the
`PhysicalParameters(72°F)` constructor default. This means the Java GUI always
starts at 20°C, while our golden harness (which bypasses preferences) uses 72°F.

Our session keeps 72°F to match golden fixtures and avoid test mismatches.
The web app currently shows 22.22°C in the console instead of Java's 20°C.
- Added 6 tests at 20°C to verify model correctness at the app-visible temperature
- Added 8 humidity variation tests (20% and 80% RH at 20°C) with monotonicity checks
- Added `set_params()` to `StudySession` for future use
- **TODO**: Add a preferences/settings layer (WASM or UI) that overrides to 20°C,
  mirroring how Java's `OptimizationPreferences` overrides the core default

### Test count
- **179 tests** total (up from 146)
  - bobyqa: 32 + 1 doc test
  - wid-optimize: 20
  - wid-eval: 8 unit + 4 integration
  - wid-compile: 22
  - wid-types: 17
  - wid-math: 22
  - wid-physics: 36 (up from 20: +6 at 20°C, +8 humidity variation, +2 monotonicity)
  - wid-session: 17

### Phase 4c: UI Shell + Settings + File Handling

- **Settings dialog** (gear icon in header): Temperature (°C) and Humidity (%)
  fields with Apply/Cancel. Calls `set_params` WASM command to update physical
  parameters. Applying 20°C produces values matching the Java app exactly.
- **Save button**: exports selected instrument as XML file download
- **Multi-file support**: both Open File button and drag-and-drop accept
  multiple XML files at once
- **Action buttons**: Sketch, Evaluate Tuning, Optimize — all gated by session
  state with explanatory tooltips when disabled. Sketch and Optimize are wired
  for gating but not yet functional (Phase 4e).
- **WASM commands added**: `set_params` (temperature + humidity override)
- **Re-exports**: `PhysicalParameters` and `TemperatureType` from wid-session
  for use by wid-wasm
- Removed unused `fileContent` signal from App.tsx
