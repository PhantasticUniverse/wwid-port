# Testing Guide

## Running tests

```bash
cd wid

# All tests (457)
cargo test

# Tests matching a pattern
cargo test naf_
cargo test whistle_eval
cargo test optimization

# A specific integration test file
cargo test --test bulk_naf_eval

# With output (for debugging)
cargo test test_name -- --nocapture
```

## Test tiers

### Unit tests (inline `#[test]`)

Located alongside the code they test, inside `#[cfg(test)] mod tests { ... }` blocks. These test individual functions in isolation.

Examples:
- `wid-math/src/lib.rs` — transfer matrix multiplication
- `wid-physics/src/lib.rs` — speed of sound calculation
- `wid-compile/src/lib.rs` — instrument compilation
- `wid-optimize/src/lib.rs` — norm calculation

### Integration tests (golden fixture parity)

Located in `wid/tests/`. These load golden fixtures from `golden/expected/` and compare the port's output against the Java oracle's output.

Examples:
- `tests/bulk_naf_eval.rs` — 36 NAF instrument/tuning combos (540 fingerings)
- `tests/whistle_eval.rs` — 16 Whistle combos (272 fingerings)
- `tests/flute_eval.rs` — 8 Flute combos (110 fingerings)
- `tests/reed_eval.rs` — 7 Reed combos (72 fingerings)
- `tests/naf_fipple.rs` — fipple factor calibration (load-bearing)
- `tests/optimization.rs` — optimizer parity across all study models

### Session tests

In `wid-session/src/lib.rs` — 60+ tests exercising the full session API: document loading, selection, gating, evaluation, optimization, calibration, analysis tools, tuning wizard.

## Golden fixture structure

```
golden/
├── scenarios/          Input scenario definitions (JSON)
│   ├── NAF-FF-01.json
│   ├── WH-BORE-01.json
│   └── ...
└── expected/           Oracle outputs (committed)
    ├── NAF-BULK-EVAL/
    │   └── results.json
    ├── NAF-FF-01/
    │   └── results.json
    └── ...
```

Each scenario JSON defines the inputs (instrument file, tuning file, optimizer, constraints). The expected directory contains the oracle's output for that scenario.

## Tolerance definitions

| Test type | Tolerance | Rationale |
|-----------|-----------|-----------|
| Evaluation | ≤ 0.5 cents per fingering | Accounts for floating-point path differences |
| Optimization | ≤ 1.0 cents per weighted note, or norm ≤ oracle + epsilon | BOBYQA can converge to different local minima |
| Z-samples | `abs_err ≤ A + R × max(\|expected\|, \|actual\|)` | Avoids false failures at resonance roots where Im(Z) ≈ 0 |

## How to add a new golden fixture

### End-to-end walkthrough

1. **Create a scenario definition** in `golden/scenarios/`:

   ```json
   {
     "instrumentFile": "samples/NAF/A4 NAF.xml",
     "tuningFile": "samples/NAF/A4 NAF Tuning.xml"
   }
   ```

2. **Generate the oracle output** (requires Java 17+ and oracle downloaded):

   ```bash
   cd golden-harness
   JAVA_HOME=/opt/homebrew/opt/openjdk@17 \
     ./gradlew run --args="MY-SCENARIO"
   ```

   This writes to `golden/expected/MY-SCENARIO/`.

3. **Write a Rust parity test**:

   ```rust
   #[test]
   fn my_scenario_matches_oracle() {
       let expected: MyExpected = load_golden("MY-SCENARIO");
       let session = setup_session_from_scenario("MY-SCENARIO");
       let actual = session.evaluate_tuning().unwrap();

       for (exp, act) in expected.notes.iter().zip(actual.rows.iter()) {
           let cents_err = (act.cents - exp.cents).abs();
           assert!(cents_err <= 0.5, "cents deviation {} for {}", cents_err, exp.note);
       }
   }
   ```

4. **Run and verify**: `cargo test my_scenario`

5. **Commit**: scenario JSON + expected output + test code.

## Special cases

### BOBYQA chaotic sensitivity

BOBYQA's Hessian estimation amplifies tiny evaluation differences (~1e-9) by ~1000×, potentially causing divergent trajectories on multimodal landscapes. For sensitive optimizers, test evaluation parity and norm reduction rather than exact trajectory match.

### Fipple factor (load-bearing)

Fipple factor behavior has dedicated fixtures (`NAF-FF-01`, `NAF-FF-02`, `NAF-FF-03`). Any change to fipple handling must preserve these fixtures exactly.
