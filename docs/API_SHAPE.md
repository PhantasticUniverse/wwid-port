# API_SHAPE.md — Session-Based Compute Contract

## Purpose

> **Status**: Core API implemented across all four study models (NAF, Whistle, Flute, Reed). Session lifecycle, document I/O, selection, evaluation, optimization, calibration, and constraints creation are all working end-to-end in the browser. DIRECT-C global optimization and multi-start infrastructure complete. 280 tests passing. Remaining: spectrum/sketch/compare tools, NAF taper optimizers, bore optimizers.

Define a stable `StudySession` API that mirrors baseline StudyModel behavior:

* Selection-driven tools
* Gating rules (what's available depends on what's selected)
* Outputs as structured data + XML artifacts
* Cancellation + progress for long-running operations

Used by:

* **Browser worker host** — WASM bindings dispatch JSON commands to `StudySession`
* **Tests** — headless session driving golden fixture parity tests
* **Optional CLI** — same API, different frontend

---

## Conceptual State Model

```
Session
  study_kind: NAF | Whistle | Flute | Reed
  docs: { Instrument[], Tuning[], Constraints[] }
  selection:
    instrumentId?
    tuningId?
    optimizerKey?
    constraintsId?
  params:
    temperature (°F or °C)
    humidity (%)
  calc_params:
    hole_size_mult, finger_adjustment
    termination_type, mouthpiece_model
    blowing_level (Whistle/Flute only)
```

### Document Lifecycle

```
1. Open       →  open_xml(xml_string) parses and stores a document
2. Select     →  select_instrument/tuning/optimizer/constraints
3. Evaluate   →  evaluate_tuning() returns per-fingering results
4. Calibrate  →  calibrate() adjusts mouthpiece params in-place
5. Optimize   →  optimize() produces new instrument with adjusted geometry
6. Export     →  export_xml(doc_id) serializes back to WIDesigner XML
```

### Gating (enabled operations)

| Predicate | Requirements |
|-----------|-------------|
| `can_tune()` | Instrument + Tuning selected, matching hole counts |
| `can_calibrate()` | `can_tune()` + selected optimizer is a calibrator type |
| `can_optimize()` | `can_tune()` + optimizer + constraints selected, constraint dimensions match optimizer |

---

## Commands

### Session lifecycle

| Command | Returns | Description |
|---------|---------|-------------|
| `createSession(studyKind)` | `sessionId` | Initialize session for a study model |
| `resetSession()` | — | Clear all documents and selection |
| `getSessionState()` | snapshot | Documents, selection, enabled tools, params |

### Document I/O

| Command | Returns | Description |
|---------|---------|-------------|
| `openXml(xmlString)` | `{ docId, docKind, metadata }` | Parse XML, auto-detect type, store |
| `exportXml(docId)` | `xmlString` | Serialize document to WIDesigner XML |
| `listDocs(docKind?)` | `[{docId, name, metadata}]` | List stored documents by type |
| `deleteDoc(docId)` | — | Remove a document |

**Example — open and select an instrument:**
```json
// Request
{"command": "open_xml", "xml": "<Instrument xmlns=\"http://...\">\n  ..."}

// Response
{"doc_id": 1, "doc_kind": "Instrument", "name": "SamplePVC-Whistle"}

// Then select it
{"command": "select_instrument", "doc_id": 1}
```

### Selection

| Command | Description |
|---------|-------------|
| `selectInstrument(docId)` | Set active instrument |
| `selectTuning(docId)` | Set active tuning |
| `selectOptimizer(optimizerKey)` | Set active optimizer/calibrator |
| `selectConstraints(docId)` | Set active constraints |
| `clearSelection(kind)` | Clear one selection slot |

### Constraints creation

| Command | Returns | Description |
|---------|---------|-------------|
| `createDefaultConstraints(optimizerKey)` | `constraintsDocId` | Generate template with default bounds |
| `createBlankConstraints(optimizerKey)` | `constraintsDocId` | Generate template with wide bounds |

**Constraints structure** (what the bounds arrays look like):

For a 6-hole Whistle `HoleObjectiveFunction`, the constraints contain 13 entries:
- 7 position constraints: bore end position + 6 inter-hole spacings (metres)
- 6 size constraints: hole diameters (metres)

```json
{
  "name": "Default",
  "objective_function_name": "HoleObjectiveFunction",
  "number_of_holes": 6,
  "constraint_list": [
    {"display_name": "Bore length", "lower_bound": 0.2, "upper_bound": 0.7},
    {"display_name": "Hole 1 spacing", "lower_bound": 0.012, "upper_bound": 0.04},
    ...
    {"display_name": "Hole 1 diameter", "lower_bound": 0.004, "upper_bound": 0.0091},
    ...
  ]
}
```

**Ordering is ABI**: the objective function reads lower/upper bound arrays positionally — position bounds first, then size bounds. Reordering breaks optimization.

### Options

| Command | Description |
|---------|-------------|
| `setPhysicalParams(temp, humidity)` | Override temperature and humidity |

---

## Tool Endpoints

### Evaluate tuning

```json
// Request
{"command": "evaluate_tuning"}

// Response
{
  "rows": [
    {"note": "F#4", "target_freq": 370.0, "predicted_freq": 369.8, "cents": -0.94, "weight": 1},
    ...
  ],
  "net_error": 15900.0,
  "mean_deviation": 42.3
}
```

### Calibrate

```json
// Request
{"command": "calibrate"}

// Response (NAF fipple example)
{
  "initial_fipple_factor": 0.75,
  "final_fipple_factor": 0.274,
  "initial_norm": 90010.0,
  "final_norm": 0.0009
}
```

### Optimize (async with progress)

```json
// Started via optimize() WASM call — not a JSON command
// Progress callback receives:
{"evaluations": 1500, "best_value": 975.14}

// Final result:
{
  "new_instrument_id": 3,
  "initial_norm": 1324815.0,
  "final_norm": 975.14,
  "evaluations": 1750
}
```

### Available optimizers

```json
// Request
{"command": "available_optimizers"}

// Response (Whistle example)
[
  {"key": "WindowHeightObjectiveFunction", "display_name": "Window height calibrator"},
  {"key": "BetaObjectiveFunction", "display_name": "Beta calibrator"},
  {"key": "WhistleCalibrationObjectiveFunction", "display_name": "Whistle calibration"},
  {"key": "HoleSizeObjectiveFunction", "display_name": "Hole size only"},
  {"key": "HolePositionObjectiveFunction", "display_name": "Hole position only"},
  {"key": "HoleObjectiveFunction", "display_name": "Hole position & size"},
  {"key": "GlobalHolePositionObjectiveFunction", "display_name": "Hole spacing (global)"},
  {"key": "GlobalHoleObjectiveFunction", "display_name": "Hole size+spacing (global)"}
]
```

---

## Optimizer Strategies

| Strategy | Optimizer Key Prefix | Algorithm | Budget |
|----------|---------------------|-----------|--------|
| Local | `Hole*`, `HoleSize*`, `HolePosition*` | BOBYQA | 20K + 5K per dim |
| Global | `Global*` | DIRECT-C → BOBYQA | 2× budget (DIRECT) + 1× (BOBYQA) |
| Calibrator | `*CalibrationObjectiveFunction`, `*ObjectiveFunction` (fipple/beta/etc) | 1D Brent or 2D BOBYQA | varies |

Global optimizers use a two-stage pipeline:
1. **DIRECT-C** global search (convergence 7e-8, target value 0.001) finds a good basin
2. **BOBYQA** local refinement converges precisely from DIRECT-C's best point
3. The better of the two results is kept

---

## Errors (structured)

| Error | Cause |
|-------|-------|
| `HoleCountMismatch` | Instrument and tuning have different hole counts |
| `InvalidXml(kind, details)` | XML parsing or validation failure |
| `MissingSelection(kind)` | Required document not selected |
| `UnsupportedObjectiveForStudy` | Optimizer not available for current study model |
| `OptimizationCancelled` | User cancelled a running optimization |
| `NumericFailure(...)` | Root-finding or impedance computation failure |

---

## Stability Rules

* `docId` values are stable within a session (monotonically increasing integers)
* Outputs are deterministic under fixed params/selection
* Constraints ordering is treated as ABI: the objective function parameterization depends on the positional layout of the lower/upper bound arrays
