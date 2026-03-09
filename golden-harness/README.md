# Golden Harness

Java CLI that generates golden fixture outputs from the official WIDesigner v2.6.0 release JARs. These fixtures are the source of truth for verifying parity of the Rust port.

## Prerequisites

- **Java 17+** (on macOS: `brew install openjdk@17`)
- **Oracle downloaded**: run `./tools/fetch-oracle.sh` from the repo root to download and extract the WIDesigner v2.6.0 release to `oracle/v2.6.0/`

On macOS, set `JAVA_HOME` if needed:
```bash
export JAVA_HOME=/opt/homebrew/opt/openjdk@17
```

## Commands

### Run a specific scenario

```bash
cd golden-harness
./gradlew run --args="SCENARIO-ID"
```

The scenario ID maps to a JSON file in `golden/scenarios/` (e.g., `NAF-FF-01`). Output is written to `golden/expected/SCENARIO-ID/`.

### Run all scenarios

```bash
./gradlew run --args="--all"
```

### Run a specific driver class

```bash
./gradlew run -PmainClass=com.widgolden.NafBulkEvalDriver
```

### Build without running

```bash
./gradlew build
```

## Driver classes

| Class | Purpose |
|-------|---------|
| `GoldenHarnessMain` | Entry point — dispatches scenario IDs to appropriate drivers |
| `NafBulkEvalDriver` | Evaluates all 36 NAF instrument/tuning combos |
| `WhistleBulkEvalDriver` | Evaluates all Whistle instrument/tuning combos |
| `FluteBulkEvalDriver` | Evaluates all Flute instrument/tuning combos |
| `ReedBulkEvalDriver` | Evaluates all Reed instrument/tuning combos |
| `WhistleCalibDriver` | Whistle calibration scenarios |
| `FluteCalibDriver` | Flute calibration scenarios |
| `ReedCalibDriver` | Reed calibration scenarios |
| `WhistleOptDriver` | Whistle optimization scenarios |
| `FluteOptDriver` | Flute optimization scenarios |
| `GenericOptDriver` | Generic optimizer scenarios (any study model) |
| `BoreOptDriver` | Bore optimizer scenarios |
| `DirectOptDriver` | DIRECT-C global optimizer scenarios |
| `WhistleZSampleDriver` | Whistle impedance sample scenarios |
| `FluteZSampleDriver` | Flute impedance sample scenarios |
| `ReedZSampleDriver` | Reed impedance sample scenarios |
| `ToolsDriver` | Analysis tool output scenarios (sketch, compare, supplementary, graph, spectrum) |
| `WizardDriver` | Tuning wizard scenarios (scale generation, tuning from pattern) |

Supporting classes:
- `Scenario` — scenario definition (instrument, tuning, optimizer, physical params)
- `ScenarioRunner` — orchestrates running a scenario through the oracle
- `OutputFormatter` — formats oracle output as JSON fixtures
- `InstrumentOverrides` — applies parameter overrides to oracle instruments

## How to add a new scenario

1. **Create the scenario JSON** in `golden/scenarios/`:
   ```json
   {
     "instrumentFile": "path/relative/to/oracle/samples",
     "tuningFile": "path/relative/to/oracle/samples",
     "optimizerKey": "HoleFromTop",
     "constraintsFile": "optional/constraints/path"
   }
   ```

2. **Add the scenario ID** to `GoldenHarnessMain` dispatch (or use an existing driver).

3. **Run it** to generate the expected output:
   ```bash
   ./gradlew run --args="MY-NEW-SCENARIO"
   ```

4. **Commit** the scenario JSON and expected output directory.

5. **Write a Rust parity test** in `wid/tests/` that loads the golden fixture and compares against the port's output.
