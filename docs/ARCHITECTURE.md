# Architecture

This document describes the internal architecture of the WIDesigner port for developers extending or maintaining the system.

## Crate dependency graph

```
wid-wasm ─── wid-session ─── wid-optimize ─── wid-eval ─── wid-acoustics ─── wid-math
                │                  │              │              │
                │                  ├── bobyqa     │              └── wid-physics
                │                  └── direct     │
                │                                 │
                └── wid-types                     └── wid-compile ─── wid-types
                                                                        │
                                                                   wid-physics
```

### Crate responsibilities

| Crate | Purpose |
|-------|---------|
| `wid-math` | TransferMatrix (2x2 complex), StateVector, numeric primitives |
| `wid-physics` | Air properties: speed of sound, density, viscosity (CIPM-2007) |
| `wid-types` | XML domain model: `InstrumentRaw`, `Tuning`, `Constraints` + serde |
| `wid-compile` | `compile(InstrumentRaw) → InstrumentCompiled` — component chain, headspace, ordering |
| `wid-acoustics` | Transfer matrices for bore sections, toneholes, termination, mouthpiece models |
| `wid-eval` | Impedance calculation, frequency prediction (Im(Z)=0 root finding), cents deviation |
| `wid-optimize` | Objective functions, calibration, BOBYQA/DIRECT dispatch, norm calculation |
| `bobyqa` | Standalone BOBYQA (Bound Optimization BY Quadratic Approximation) |
| `direct` | Standalone DIRECT-C (DIviding RECTangles) global optimizer |
| `wid-session` | Session orchestrator: document store, selection, gating, command dispatch |
| `wid-wasm` | Thin WASM bindings: `execute(json) → json` for sync, `optimize(callback)` for async |

## Data flow

```
                     ┌─────────────────────────────────────────────────────┐
                     │                   wid-session                       │
                     │  StudySession { docs, selection, params }           │
                     └──────────┬──────────────────────────────────────────┘
                                │
              ┌─────────────────┼─────────────────┐
              ▼                 ▼                  ▼
        open_xml()        evaluate_tuning()    optimize()
              │                 │                  │
              ▼                 ▼                  ▼
     ┌────────────┐    ┌───────────────┐   ┌──────────────────┐
     │  wid-types  │    │  wid-compile   │   │   wid-optimize    │
     │  parse XML  │───▶│  compile()     │──▶│  objective fn     │
     └────────────┘    └───────┬───────┘   │  BOBYQA/DIRECT    │
                               │           └────────┬─────────┘
                               ▼                    │
                       InstrumentCompiled            │
                               │                    │
                               ▼                    │
                        ┌──────────────┐            │
                        │   wid-eval    │◀───────────┘
                        │  calc_z()     │  (called per evaluation)
                        │  predict_f()  │
                        └──────────────┘
```

### Key types

- **`InstrumentRaw`** — direct XML representation (what users load/save). Units as specified by `length_type`.
- **`InstrumentCompiled`** — output of `compile()`. Component chain in metres, ready for acoustics. Contains: mouthpiece, bore sections interleaved with toneholes, termination.
- **`Tuning`** — note names, target frequencies, fingering patterns, optimization weights.
- **`Constraints`** — lower/upper bound arrays defining the optimizer search space. Array ordering is ABI (must match Java exactly).
- **`StudySession`** — owns all state. JSON command dispatch via `execute()`.

## Session orchestrator pattern

`wid-session::StudySession` is the central API:

1. **Document store** — open/close/get/set instrument, tuning, constraints XML
2. **Selection** — which instrument, tuning, optimizer, constraints are active
3. **Gating** — `can_tune()`, `can_optimize()`, `can_sketch()` based on selection
4. **Dispatch** — `evaluate_tuning()`, `optimize()`, `calibrate()`, analysis tools
5. **Study-model polymorphism** — `StudyKind` enum routes to model-specific modules

### WASM integration

`wid-wasm` is a thin JSON wrapper:
- `execute(json: &str) → String` — synchronous commands (open, evaluate, sketch, etc.)
- `optimize(callback: js_sys::Function)` — async with progress streaming
- The web frontend runs WASM in a Web Worker (off main thread)
- `ComputeService` in the frontend manages the worker lifecycle

## How to add a 5th study model

1. **Add `StudyKind` variant**: `wid-session/src/lib.rs` — add to the enum
2. **Create module**: `wid-session/src/bagpipe.rs` (or similar)
3. **Implement required functions**:
   - `available_optimizers() → Vec<(key, display_name)>`
   - `default_physical_params() → PhysicalParams`
   - `create_default_constraints(instrument, tuning, optimizer_key) → Constraints`
   - Calibration dispatch (if applicable)
   - Optimizer dispatch
4. **Wire up session dispatch**: add `StudyKind::Bagpipe => ...` arms in `evaluate_tuning()`, `optimize()`, `calibrate()`, `create_default_constraints()`, etc.
5. **Add acoustic support**: if the new model needs a new mouthpiece type, add it to `wid-acoustics/src/mouthpiece.rs` and `wid-compile`
6. **Add golden fixtures**: create a Java driver in `golden-harness/`, generate fixtures, write parity tests
7. **Update frontend**: add the study kind to the selector dropdown in `StudyPanel.tsx`

## Load-bearing interfaces

### JSON command names (WASM ABI)

The 43 commands dispatched via `wid-wasm::execute()` use camelCase naming (e.g., `evaluateTuning`, `openXml`, `getInstrumentXml`). Frontend code in `session.ts` must match exactly.

### Constraints ordering

Constraint lower/upper bound arrays encode optimizer parameters in a specific order. This ordering is ABI — the objective function reads parameters by index. Changing the order breaks optimization.

### InstrumentCompiled layout

The component chain in `InstrumentCompiled` must maintain ascending position order with bore sections interleaved between toneholes. The acoustic pipeline walks this chain sequentially.

## Key design decisions

| Decision | Rationale |
|----------|-----------|
| Explicit `compile()` step | Prevents Java's "forgot to call updateComponents" bugs. All acoustics operate on compiled representation only |
| Worker-based compute | Heavy optimization runs off main thread. Progress streaming via callback, cancellation via shared flag |
| Golden fixture parity testing | The oracle (Java v2.6.0) is source of truth. Tests compare against committed fixture outputs, not the Java code itself |
| Session owns all state | Single point of truth for documents, selection, and parameters. No distributed state bugs |
| Popup windows for tools | Matches Java's JFrame behavior. Each tool opens in its own browser window |
