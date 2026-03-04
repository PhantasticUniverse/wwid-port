# wid-session

Session orchestrator for the WIDesigner port. Owns documents, selection state, physical parameters, and dispatches evaluation, calibration, and optimization operations across all four study models.

## Concepts

The session is the top-level entry point for all instrument design operations. It follows a **selection-driven** pattern:

1. **Open** documents (instrument XML, tuning XML, constraints XML)
2. **Select** which instrument, tuning, optimizer, and constraints to use
3. **Gate** — available operations depend on what's selected (e.g., can't optimize without constraints)
4. **Execute** — evaluate tuning, calibrate mouthpiece, or optimize hole geometry
5. **Export** — serialize the result back to WIDesigner-compatible XML

This mirrors how Java's `StudyModel` classes work, but makes the state machine explicit rather than implicit.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `StudySession` | Top-level struct; owns `DocStore`, `Selection`, and `PhysicalParameters` |
| `open_xml()` | Parse XML string, auto-detect type (Instrument/Tuning/Constraints), store |
| `export_xml()` | Serialize a stored document back to WIDesigner-compatible XML |
| `select_instrument()` / `select_tuning()` / `select_optimizer()` / `select_constraints()` | Set selection by `DocId` or key |
| `can_tune()` / `can_optimize()` / `can_calibrate()` | Gating predicates |
| `evaluate_tuning()` | Run evaluation pipeline, return per-fingering results |
| `calibrate()` | Dispatch to study-model-specific calibrator |
| `optimize()` | Run BOBYQA or DIRECT-C→BOBYQA with progress callback |
| `create_default_constraints()` / `create_blank_constraints()` | Generate constraint templates |
| `list_docs()` | List stored documents filtered by `DocKind` |
| `available_optimizers()` | List optimizer/calibrator options for current study model |

## Study Model Registry

Each `StudyKind` defines its own calibrators and hole optimizers:

| Study Model | Calibrators | Hole Optimizers | Global Optimizers |
|-------------|-------------|-----------------|-------------------|
| **NAF** | Fipple factor (1D Brent) | Hole from top | — |
| **Whistle** | Window height, Beta, Joint WH+Beta | Hole size, Hole position, Hole combined | Global hole position, Global hole |
| **Flute** | Airstream length, Beta, Joint AL+Beta | Hole size, Hole position, Hole combined | Global hole position, Global hole |
| **Reed** | Alpha+Beta (2D BOBYQA) | Hole size, Hole position, Hole combined | Global hole |

## Calibration vs Optimization

These are fundamentally different operations:

- **Calibrators** adjust mouthpiece parameters (fipple factor, window height, airstream length, alpha, beta) to minimize error at existing hole positions. They **modify the instrument in-place** — the existing document is updated.

- **Hole optimizers** adjust hole geometry (diameters, positions, or both) within constraint bounds. They **produce a new instrument document** — the original is preserved and a new document is added to the session.

## Optimizer Strategy Dispatch

The session automatically selects the right optimization strategy based on the optimizer key:

| Key Pattern | Strategy | Pipeline |
|-------------|----------|----------|
| `FippleFactorObjectiveFunction` | 1D calibration | Brent minimizer |
| `*CalibrationObjectiveFunction` | 2D calibration | BOBYQA |
| `WindowHeight*`, `Beta*`, `AirstreamLength*` | 1D calibration | Brent |
| `Hole*`, `HoleSize*`, `HolePosition*` | Local optimization | BOBYQA |
| `Global*` | Global optimization | DIRECT-C → BOBYQA |

## Result Types

| Type | Fields | Produced by |
|------|--------|-------------|
| `CalibResult` | fipple/window/airstream/alpha/beta, initial/final norm | `calibrate()` |
| `OptimizeResult` | new_instrument_id, initial/final norm, evaluations | `optimize()` |
| `TuningResult` | rows: `Vec<EvalRow>`, net_error, mean_deviation | `evaluate_tuning()` |
| `EvalRow` | note, target_freq, predicted_freq, cents, weight | Per-fingering detail |

## Gating Logic

| Predicate | Requirements |
|-----------|-------------|
| `can_tune()` | Instrument + Tuning selected, matching hole counts |
| `can_optimize()` | `can_tune()` + Optimizer + Constraints selected, constraint dimensions match |
| `can_calibrate()` | `can_tune()` + Optimizer is a calibrator type (no constraints needed) |

**Common gating issues**:
- Opening a 6-hole instrument with an 8-hole tuning → `can_tune() = false` (hole count mismatch)
- Selecting a hole optimizer without constraints → `can_optimize() = false`
- Selecting a calibrator → `can_calibrate() = true`, `can_optimize() = false` (calibrators don't use constraints)

## Module Structure

| Module | Description |
|--------|-------------|
| `lib.rs` | `StudySession` struct, dispatch logic, XML I/O |
| `naf.rs` | NAF optimizer registry + constraint templates |
| `whistle.rs` | Whistle optimizer registry + constraint templates |
| `flute.rs` | Flute optimizer registry + constraint templates |
| `reed.rs` | Reed optimizer registry + constraint templates |
| `doc_store.rs` | `DocStore`, `StoredDoc`, `DocContent` — document storage |
| `types.rs` | `DocId`, `DocKind`, `StudyKind`, result types, `SessionError` |

## Dependencies

- `wid-compile` — instrument compilation + geometry mutation
- `wid-eval` — impedance evaluation pipeline
- `wid-optimize` — calibrators, hole optimizers, global optimizers
- `wid-types` — XML domain model types
- `wid-physics` — physical parameters
- `bobyqa` — BOBYQA optimizer (progress callback types)

## Tests

22 tests covering session lifecycle, document I/O, selection management, gating predicates, evaluation, constraint generation, XML round-trips, and document mutation.
