# wwid-port

A browser-runnable port of **[Woodwind Instrument Designer (WIDesigner)](https://github.com/edwardkort/WWIDesigner)** v2.6.0 — full end-user feature parity across all four study models (NAF, Whistle, Flute, Reed), running entirely in the browser via Rust → WebAssembly.

## Why?

WIDesigner is a powerful Java Swing desktop app for designing and optimizing woodwind instruments — NAFs (Native American flutes), tin whistles, transverse flutes, and reed instruments. It uses numerical optimization (BOBYQA, DIRECT-C) to find hole positions and sizes that produce accurate tuning.

The original requires Java, a JVM, and a desktop OS. This port brings the same capabilities to any modern browser with WebAssembly support, with no install, no server, and no Java dependency. All computation runs locally in a Web Worker.

## How it works

```
              ┌─────────────┐     ┌──────────────┐     ┌──────────────┐
  User XML →  │  wid-types   │ ──→ │  wid-compile  │ ──→ │  wid-eval    │ ──→ results
              │  (parse XML) │     │  (component   │     │  (impedance  │
              └─────────────┘     │   chain)      │     │   pipeline)  │
                                  └──────────────┘     └──────┬───────┘
                                                              │
                                  ┌──────────────┐     ┌──────┴───────┐
                                  │  wid-session  │ ←── │ wid-optimize │
                                  │  (orchestrate)│     │  (BOBYQA,    │
                                  └──────┬───────┘     │   DIRECT-C)  │
                                         │             └──────────────┘
                                  ┌──────┴───────┐
                                  │   wid-wasm   │ ──→ Web Worker ──→ Browser UI
                                  │  (bindings)  │
                                  └──────────────┘
```

1. **Parse** — Load WIDesigner-format XML (instruments, tunings, constraints)
2. **Compile** — Convert raw geometry to an acoustic component chain
3. **Evaluate** — Walk the chain computing impedance, find playing frequencies, measure cents deviation
4. **Optimize** — Iteratively adjust geometry to minimize tuning error (BOBYQA for local, DIRECT-C for global)
5. **Display** — SolidJS frontend with evaluation tables, editors, progress tracking

## How we ensure parity

We use an **oracle + golden fixtures** workflow:

1. The official Java release package (v2.6.0) is the **oracle** — source of truth
2. A small Java CLI ("golden harness") generates **golden fixtures**: predicted frequencies, impedance samples, optimization outcomes
3. The port must match these fixtures within defined tolerances (≤ 0.5 cents for evaluation, ≤ 1.0 cents for optimization)
4. **449 tests** validate parity across all four study models

Fipple factor behavior is **load-bearing** and has dedicated fixtures to prevent accidental drift.

## Repo layout

```
docs/                     Planning and specification documents
  PORT_SPEC.md              Definition of "done" + parity rules
  FIXTURE_PLAN.md           Golden scenario suite design
  API_SHAPE.md              Session-based API contract
  FEATURE_MATRIX.md         Per-study-model parity checklist

WWIDesigner-2.6.0-src/    Upstream Java source snapshot (read-only reference)

oracle/                   Extracted official v2.6.0 release (gitignored)
golden-harness/           Java CLI to generate fixtures from the oracle
golden/                   Committed scenarios + expected oracle outputs

wid/                      The port (Rust workspace)
  crates/bobyqa/            Standalone BOBYQA optimizer
  crates/direct/            Standalone DIRECT-C global optimizer
  crates/wid-math/          TransferMatrix and StateVector types
  crates/wid-physics/       Physical parameters (air model)
  crates/wid-types/         XML domain model (parse/serialize)
  crates/wid-compile/       Raw → compiled instrument pipeline
  crates/wid-acoustics/     Acoustic element transfer matrices
  crates/wid-eval/          Impedance evaluation + frequency prediction
  crates/wid-optimize/      Calibration + optimization infrastructure
  crates/wid-session/       Session orchestrator (selection + dispatch)
  crates/wid-wasm/          WASM bindings (Web Worker interface)

web/                      SolidJS + Vite + Tailwind frontend
tools/                    Helper scripts (fetch oracle, etc.)
```

## Quick start

### Prerequisites

- **Rust** 1.86+ (edition 2024)
- **Java 17+** (only for golden fixture generation; not needed for running the port)
- **Node.js** 18+ (for the web frontend)

### Run the port tests

```bash
cd wid
cargo test        # All 449 tests
```

### Generate golden fixtures (optional — fixtures are committed)

```bash
./tools/fetch-oracle.sh                    # Download WIDesigner v2.6.0 release
cd golden-harness
JAVA_HOME=/opt/homebrew/opt/openjdk@17 \
  ./gradlew run --args="--all"             # Generate all fixtures
```

### Build and run the web app

```bash
# Build WASM
cd wid
cargo build --target wasm32-unknown-unknown --release -p wid-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm \
  --out-dir ../web/wasm --target web

# Run dev server
cd ../web
npm install
npx vite          # Opens at http://localhost:5173
```

Requires `wasm-bindgen-cli` 0.2.100 (pinned for Rust 1.86 compatibility).

### Browser requirements

Any browser with WebAssembly and Web Worker support: Chrome 57+, Firefox 52+, Safari 11+, Edge 16+.

## Development workflow

```
1. Write/modify Java driver    →  golden-harness/src/.../FooDriver.java
2. Generate golden fixture     →  ./gradlew run --args="FOO-01"
3. Write Rust implementation   →  wid/crates/.../foo.rs
4. Write parity test           →  compare against golden/expected/FOO-01/
5. cargo test                  →  verify parity within tolerances
6. Rebuild WASM + test in UI   →  verify end-to-end in browser
```

## Documentation

| Document | Purpose |
|----------|---------|
| [**User Guide**](docs/user-guide/index.md) | **End-user documentation: getting started, study models, tools, optimizers, reference** |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | How to develop, test, and extend the port |
| [`CHANGELOG.md`](CHANGELOG.md) | Version history and release notes |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | Crate dependency graph, data flow, extension guide |
| [`docs/PORT_SPEC.md`](docs/PORT_SPEC.md) | Definition of done, parity metrics, milestones |
| [`docs/FIXTURE_PLAN.md`](docs/FIXTURE_PLAN.md) | Golden fixture suite design and tolerances |
| [`docs/API_SHAPE.md`](docs/API_SHAPE.md) | Session-based API contract |
| [`docs/FEATURE_MATRIX.md`](docs/FEATURE_MATRIX.md) | Per-study-model feature parity checklist |
| [`wid/TESTING.md`](wid/TESTING.md) | Testing guide: running tests, adding fixtures |
| [`golden-harness/README.md`](golden-harness/README.md) | Golden fixture generation from Java oracle |
| [`web/README.md`](web/README.md) | Frontend architecture and development |
| [`parity-notes.md`](parity-notes.md) | Debugging notes and parity gotchas |
| [`DEVLOG.md`](DEVLOG.md) | Detailed development log |

## Current status

All milestones complete (M0–M5).

| Study Model | Evaluation | Calibration | Optimization | Global Opt |
|-------------|:----------:|:-----------:|:------------:|:----------:|
| NAF         | Complete   | Complete    | Complete     | —          |
| Whistle     | Complete   | Complete    | Complete     | Complete   |
| Flute       | Complete   | Complete    | Complete     | Complete   |
| Reed        | Complete   | Complete    | Complete     | Complete   |

449 tests, 57 golden fixture sets, 5 analysis tools, tuning wizard — all passing.

## License

Port code: Apache-2.0. See [LICENSE](LICENSE) and [NOTICE](NOTICE) for details.

Upstream WIDesigner: GPL v3 (source snapshot included for read-only reference only; not compiled into the port).
