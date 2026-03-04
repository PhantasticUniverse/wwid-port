# wid-session

Session orchestrator for the WIDesigner port. Owns documents, selection state, physical parameters, and dispatches evaluation, calibration, and optimization operations across all four study models.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `StudySession` | Top-level session struct; owns `DocStore`, `Selection`, and `PhysicalParameters` |
| `open_xml()` | Parse XML string, auto-detect type (Instrument/Tuning/Constraints), store and return `OpenResult` |
| `export_xml()` | Serialize a stored document back to WIDesigner-compatible XML |
| `select_instrument()` / `select_tuning()` / `select_optimizer()` / `select_constraints()` | Set current selection by `DocId` or key |
| `can_tune()` / `can_optimize()` / `can_calibrate()` | Gating predicates based on current selection |
| `evaluate_tuning()` | Run evaluation pipeline, return `TuningResult` with per-fingering rows |
| `calibrate()` | Dispatch to study-model-specific calibrator, return `CalibResult` |
| `optimize()` | Run BOBYQA optimization with progress callback, return `OptimizeResult` |
| `create_default_constraints()` / `create_blank_constraints()` | Generate constraint templates for selected optimizer |
| `list_docs()` | List stored documents filtered by `DocKind` |
| `available_optimizers()` | List optimizer/calibrator options for current study model |

## Study Model Registry

Each `StudyKind` defines its own calibrators and hole optimizers:

| Study Model | Calibrators | Hole Optimizers |
|-------------|-------------|-----------------|
| **NAF** | Fipple factor (1D Brent) | Hole from top, Hole group from top, Hole size |
| **Whistle** | Window height (1D Brent), Beta (1D Brent), Joint WH+Beta (2D BOBYQA) | Hole size, Hole position, Hole combined |
| **Flute** | Airstream length (1D Brent), Beta (1D Brent), Joint AL+Beta (2D BOBYQA) | Hole size, Hole position, Hole combined |
| **Reed** | *(not yet implemented)* | *(not yet implemented)* |

## Calibration vs Optimization

- **Calibrators** adjust mouthpiece parameters (fipple factor, window height, airstream length, beta) to minimize error at existing hole positions. They modify the instrument in place.
- **Hole optimizers** adjust hole geometry (diameters, positions, or both) within constraint bounds. They produce a new instrument document.

## Result Types

| Type | Fields | Used by |
|------|--------|---------|
| `CalibResult` | `fipple_factor`, `window_height`, `airstream_length`, `beta`, `initial_norm`, `final_norm` | `calibrate()` |
| `OptimizeResult` | `new_instrument_id`, `initial_norm`, `final_norm`, `evaluations` | `optimize()` |
| `TuningResult` | `rows: Vec<EvalRow>`, `net_error`, `mean_deviation` | `evaluate_tuning()` |
| `EvalRow` | `note`, `target_freq`, `predicted_freq`, `cents`, `weight` | Per-fingering detail |

## Gating Logic

| Predicate | Requirements |
|-----------|-------------|
| `can_tune()` | Instrument + Tuning selected, matching hole counts |
| `can_optimize()` | `can_tune()` + Optimizer + Constraints selected, constraint dimensions match |
| `can_calibrate()` | `can_tune()` + Optimizer is a calibrator type |

## Module Structure

| Module | Description |
|--------|-------------|
| `lib.rs` | `StudySession` struct, dispatch logic, XML I/O |
| `naf.rs` | NAF optimizer registry + constraint templates |
| `whistle.rs` | Whistle optimizer registry + constraint templates |
| `flute.rs` | Flute optimizer registry + constraint templates |
| `doc_store.rs` | `DocStore`, `StoredDoc`, `DocContent` — document storage |
| `types.rs` | `DocId`, `DocKind`, `StudyKind`, result types, `SessionError` |

## Dependencies

- `wid-compile` — instrument compilation + geometry mutation
- `wid-eval` — impedance evaluation pipeline
- `wid-optimize` — calibrators and hole optimizers
- `wid-types` — XML domain model types
- `wid-physics` — physical parameters
- `bobyqa` — BOBYQA optimizer (progress callback types)

## Tests

22 tests covering session lifecycle, document I/O, selection management, gating predicates, evaluation, constraint generation, XML round-trips, and document mutation.
