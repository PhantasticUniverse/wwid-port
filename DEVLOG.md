# Development Log

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

#### Temperature default discrepancy (documented, not yet resolved)
The Java app's `OptimizationPreferences.DEFAULT_TEMPERATURE = 20` overrides the
`PhysicalParameters(72°F)` constructor default. This means the Java GUI always
starts at 20°C, while our golden harness (which bypasses preferences) uses 72°F.
- Our WASM session defaults to 20°C (matching what Java users see)
- Golden fixtures remain at 72°F (matching the core engine default)
- Added 6 tests at 20°C to verify model correctness at the app-visible temperature
- Added 8 humidity variation tests (20% and 80% RH at 20°C) with physical monotonicity checks
- **TODO**: Make temperature configurable via settings UI (Phase 4d/4e)
- See `parity-notes.md` for full analysis

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
