# Contributing to WIDesigner Port

The port from Java WIDesigner v2.6.0 is complete. This document covers working on the living Rust/WASM codebase going forward.

## Getting started

### Prerequisites

- **Rust** 1.86+ (edition 2024)
- **Node.js** 18+
- **wasm-bindgen-cli** 0.2.100 (`cargo install wasm-bindgen-cli@0.2.100`)
- **Java 17+** (only if regenerating golden fixtures)

### Quick setup

```bash
git clone <repo-url>
cd wwid-port

# Run all tests
cd wid && cargo test

# Build WASM
cargo build --target wasm32-unknown-unknown --release -p wid-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm \
  --out-dir crates/wid-wasm/pkg --target web

# Start dev server
cd ../web
npm install
npx vite
```

Or use the justfile: `just build` (full pipeline) or `just dev` (WASM + dev server).

## Development workflow

### Rust core

```bash
cd wid
cargo test              # All 449 tests
cargo test naf_         # Tests matching a pattern
cargo test --test bulk_naf_eval  # A specific integration test
```

### Web frontend

```bash
cd web
npx vite                # Dev server at http://localhost:5173
npx vite build          # Production build to web/dist/
```

### WASM rebuild

After changing any Rust crate, rebuild WASM before testing in the browser:

```bash
cd wid
cargo build --target wasm32-unknown-unknown --release -p wid-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm \
  --out-dir crates/wid-wasm/pkg --target web
```

## Adding features

### New study model

1. Add a `StudyKind` variant in `wid-session/src/lib.rs`
2. Create a module under `wid-session/src/` (e.g., `bagpipe.rs`)
3. Implement `available_optimizers()`, `default_physical_params()`, calibration dispatch
4. Wire up the session dispatch (`match study_kind { ... }`)
5. Add constraint creation in `create_default_constraints()`
6. Add golden fixtures and parity tests
7. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full extension pattern

### New optimizer

1. Implement the objective function in `wid-optimize/src/`
2. Add it to the study model's `available_optimizers()` list
3. Wire up dispatch in the session's `optimize()` method
4. Add a golden fixture via the Java harness (see [`golden-harness/README.md`](golden-harness/README.md))
5. Write a parity test comparing against the golden output

### New analysis tool

1. Implement the computation in `wid-session` (returning structured data)
2. Add WASM dispatch in `wid-wasm/src/lib.rs`
3. Create a popup component in `web/src/components/tools/`
4. Wire up the button in `StudyPanel.tsx`

### New UI component

The frontend uses SolidJS with reactive stores. Key patterns:
- Session state lives in `web/src/stores/session.ts`
- Compute runs in a Web Worker via `ComputeService`
- Tool results open in popup windows (matching Java JFrame behavior)

## Testing

See [`wid/TESTING.md`](wid/TESTING.md) for the full testing guide.

- **Unit tests**: inline `#[test]` in each module
- **Integration tests**: golden fixture parity tests in `wid/tests/`
- **Tolerances**: ≤0.5 cents (evaluation), ≤1.0 cents (optimization)

## Code style

- Rust edition **2024** — use modern syntax
- No `unsafe` code
- Prefer `Result` over `panic!` at API boundaries
- Keep crate boundaries clean — no circular dependencies
- No unnecessary dependencies

## Commit style

- Write detailed commit messages explaining the "why"
- Update `DEVLOG.md` for significant changes
- One logical change per commit

## Golden harness (reference)

The Java oracle harness (`golden-harness/`) exists for generating parity fixtures from the original WIDesigner JARs. Most contributors won't need it unless adding new acoustic models or optimizers. See [`golden-harness/README.md`](golden-harness/README.md).

## Project structure

```
wid/                    Rust workspace (11 crates)
web/                    SolidJS + Vite + Tailwind frontend
golden/                 Committed golden fixture data
golden-harness/         Java CLI for generating fixtures
docs/                   Architecture and spec documents
tools/                  Helper scripts
```

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for the full crate dependency graph and data flow.
