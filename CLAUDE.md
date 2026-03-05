# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This repo ports **WIDesigner v2.6.0** (a Java Swing desktop app for designing woodwind instruments) to a browser-runnable implementation using **Rust (WASM) + a web frontend**. The goal is full end-user feature parity across all four study models: NAF, Whistle, Flute (transverse), and Reed.

## Commands

### Oracle setup
```bash
./tools/fetch-oracle.sh          # Download + extract WIDesigner v2.6.0 release package to oracle/v2.6.0/
```

### Golden harness (Java, generates fixtures from the oracle)
```bash
cd golden-harness
./gradlew run --args="<scenario-id>"   # Run a specific golden fixture scenario
./gradlew run -PmainClass=com.widgolden.NafBulkEvalDriver  # Run bulk NAF eval (all 36 combos)
./gradlew build                        # Build the harness
```
Requires Java 17+ (configured via Gradle toolchain).

### Port (Rust)
```bash
cd wid
cargo build                      # Build the Rust core
cargo test                       # Run all port tests against golden fixtures
cargo test <test_name>           # Run a single test
```

### WASM build
```bash
cd wid
cargo build --target wasm32-unknown-unknown --release -p wid-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm --out-dir ../web/wasm --target web
```
Requires wasm-bindgen-cli 0.2.100 (pinned; 0.2.114 needs Rust 1.88).

### Web frontend
```bash
cd web
npm install                      # Install dependencies (first time)
npx vite                         # Dev server (default port 5173)
npx vite build                   # Production build to web/dist/
```
Stack: Vite 6 + SolidJS 1.9 + Tailwind CSS v4. The `web/wasm` symlink points to `wid/crates/wid-wasm/pkg/`.

## Architecture

### Oracle + Golden Fixtures Workflow

Parity is enforced through a three-layer system:

1. **Oracle** (`oracle/v2.6.0/`) — the official WIDesigner release JAR + sample files. Downloaded via `tools/fetch-oracle.sh`, gitignored. This is the source of truth.
2. **Golden harness** (`golden-harness/`) — a Java CLI that links against the oracle JARs and runs scenarios, producing JSON fixture outputs. Entry point: `com.widgolden.GoldenHarnessMain`.
3. **Golden fixtures** (`golden/`) — committed scenario inputs (`scenarios/`) and expected oracle outputs (`expected/`). The port's tests compare against these.

### Port Structure (`wid/`)

The port lives in `wid/crates/` as a Rust workspace. Key architectural decisions:

- **`InstrumentRaw`** = direct XML domain model (what users load/save)
- **`InstrumentCompiled`** = output of `compile(InstrumentRaw)` — component chain, termination, headspace, ordering
- All acoustics operate on the compiled representation. This explicit compile step prevents the baseline's implicit "forgot to call updateComponents" bugs.
- Heavy compute runs in a **Web Worker** (off main thread), with progress streaming and cancellation support.
- **`wid-session`** = session orchestrator (StudySession struct). Owns docs, selection, physical params. JSON command dispatch.
- **`wid-wasm`** = thin WASM bindings over wid-session. `execute(json)` for sync commands, `optimize(callback)` for async.
- **Web frontend** (`web/`) = SolidJS + Vite + Tailwind. ComputeService wraps a Web Worker that loads WASM.

### Upstream Reference (`WWIDesigner-2.6.0-src/`)

Read-only Java source snapshot for tracing behavior. Key packages under `src/main/com/wwidesigner/`:
- `geometry/` — instrument geometry model + bore/hole calculations (`geometry/calculation/`)
- `modelling/` — acoustic modelling (impedance, transfer matrices)
- `optimization/` — objective functions, BOBYQA, DIRECT-C, multi-start (`optimization/multistart/`)
- `math/` — numeric solvers (Brent, complex numbers, state vectors)
- `note/` — tuning, scales, temperaments, fingerings, wizard components (`note/wizard/`)
- `gui/` — Swing UI (not being ported; study model orchestration logic lives here)

### Session-Based API (`docs/API_SHAPE.md`)

The port exposes a `StudySession` API (used by worker host, tests, and optional CLI):
- Selection-driven: Instrument + Tuning + Optimizer + Constraints define available operations
- Gating: `canTune` requires instrument + tuning + matching hole counts; `canOptimize` adds optimizer + constraints
- All tool outputs are structured data (not UI concerns)

## Critical Parity Rules

### Fipple Factor (NAF) — Load-Bearing

Fipple factor behavior has dedicated fixtures and must match baseline exactly:
- Null handling for `fippleFactor` and `windwayHeight`
- Scaling by windway height
- Calibration semantics: uses only the lowest note, 1D optimizer (Brent), recompile after parameter change
- No "improvements" to fipple behavior until full parity is protected by tests

### Constraints Ordering is ABI

Constraints XML lower/upper bound arrays must match baseline ordering exactly — the objective function parameterization depends on this layout.

### Tolerances
- Evaluation: **<= 0.5 cents** per fingering
- Optimization: **<= 1.0 cents** per weighted note, or objective norm <= oracle + epsilon
- Z-samples: `abs_err <= A + R * max(|expected|, |actual|)` (avoid resonance roots where Im(Z) ~ 0)

## Milestones

- **M0**: Repo setup + specs ✓
- **M1**: Golden harness + fixture suite v0 (NAF + fipple protected) ✓
- **M2**: NAF evaluation parity in Rust ✓
- **M3**: NAF calibration + optimization parity ✓
- **M4**: Browser-hosted MVP (NAF end-to-end) ✓
  - Phase 4a: wid-session crate ✓
  - Phase 4b: WASM + Web Worker + frontend ✓
  - Phase 4c: UI shell + settings + file handling ✓
  - Phase 4d: Editors (instrument/tuning/constraints) ✓
  - Phase 4e: Optimization + calibration UI ✓
  - Phase 4f: Bug fixes + polish ✓
- **M5**: Full parity across all study models + tools ✓
  - M5.1: Study model infrastructure refactor ✓
  - M5.2: Whistle evaluation parity ✓
  - M5.3: Flute evaluation parity ✓
  - M5.4: Whistle calibration + optimization parity ✓
  - M5.5: Flute calibration + optimization ✓
  - M5.6: Reed evaluation model ✓
  - M5.7: Reed calibration + optimization ✓
  - 445 tests, 4 study models, 56 golden fixture sets, 5 analysis tools, tuning wizard

## Temperature Default

The Rust core and golden fixtures use 72°F (22.22°C). The Java GUI overrides this to 20°C via `OptimizationPreferences.DEFAULT_TEMPERATURE`. Our web app has a Settings dialog where users can change temperature (defaults show on first open as 22.22°C from the session). See `parity-notes.md` for full analysis.

## Key Spec Documents

- `docs/PORT_SPEC.md` — definition of done, constraints, parity metrics
- `docs/FIXTURE_PLAN.md` — golden scenario suite design and tolerances
- `docs/API_SHAPE.md` — session-based compute contract
- `docs/FEATURE_MATRIX.md` — per-study-model optimizer/tool parity checklist
