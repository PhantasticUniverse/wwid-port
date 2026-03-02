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
- **M4**: Browser-hosted MVP (NAF end-to-end)
- **M5**: Full parity across all study models + tools

## Key Spec Documents

- `docs/PORT_SPEC.md` — definition of done, constraints, parity metrics
- `docs/FIXTURE_PLAN.md` — golden scenario suite design and tolerances
- `docs/API_SHAPE.md` — session-based compute contract
- `docs/FEATURE_MATRIX.md` — per-study-model optimizer/tool parity checklist
