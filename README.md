# wwid-port

A browser-runnable port of **Woodwind Instrument Designer (WIDesigner)** with full end-user feature parity.

## Goal

Recreate everything users can do in the original WIDesigner (v2.6.0) — across all study models (NAF, Whistle, Flute, Reed) — but runnable locally in a web browser (WASM + worker), without a server.

“Done” means:
- Feature parity with WIDesigner v2.6.0
- Functionally identical results (or better), under the same inputs and constraints
- Deterministic/seeded behavior where applicable
- In-browser execution with progress + cancel for long optimizations

## How we ensure parity

We use an **oracle + golden fixtures** workflow:

1) The official Java release package (v2.6.0) is treated as the **oracle**
2) A small Java CLI (“golden harness”) generates **golden fixtures**:
   - predicted notes / cents deviations
   - impedance samples
   - optimization outcomes (norms + output XML)
3) The port must match these fixtures within defined tolerances.

Fipple factor behavior is treated as **load-bearing** and has dedicated fixtures to prevent accidental drift.

## Repo layout

- `docs/` — planning/spec docs  
  - `PORT_SPEC.md` — definition of “done” + constraints + parity rules  
  - `FIXTURE_PLAN.md` — golden scenario suite  
  - `API_SHAPE.md` — session-based compute contract  
  - `FEATURE_MATRIX.md` — parity checklist

- `WWIDesigner-2.6.0-src/` — upstream source snapshot (read-only reference)

- `oracle/` — extracted official v2.6.0 release package (ignored by git)
- `golden-harness/` — Java CLI to generate fixtures from the oracle
- `golden/` — committed scenarios + expected oracle outputs
- `wid/` — the actual port (Rust core + WASM + web app)
- `tools/` — helper scripts (fetch oracle, run harness, etc.)

## Quick start

1) Fetch oracle release:
   ```bash
   ./tools/fetch-oracle.sh
   ```

2) Generate golden outputs:
   ```bash
   cd golden-harness
   ./gradlew run --args="<scenario-id>"   # or --args="--all"
   ```

3) Run port tests against fixtures:
   ```bash
   cd wid
   cargo test
   ```

## Baseline

See `BASELINE.md` for the pinned reference build and how fixtures are generated.