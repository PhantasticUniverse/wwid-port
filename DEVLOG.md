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
