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
- NAF-FF-02 through CONSTRAINTS-02: pending

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
