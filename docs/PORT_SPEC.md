# PORT_SPEC.md

## Purpose

Port Woodwind Instrument Designer (WIDesigner) to a browser-runnable implementation while preserving full end-user capability and (as close as practical) identical behavior to the reference build.

## Baseline / Oracle

- **Oracle behavior**: WIDesigner **v2.6.0** release package.
- **Source reference**: v2.6.0 source snapshot kept in-repo for reading/traceability.

## Definition of Done

The port is “done” when:

1. **Capability parity**
  - A user can do **everything** they can do in WIDesigner v2.6.0 across all study models:
    - NAF, Whistle, Flute (transverse), Reed
    - Evaluate (tables/graphs), Calibrate, Optimize, Compare, Sketch, Save/Load XML, Cancel runs
2. **Functional parity (or better)**
  - Under the same inputs (instrument/tuning/options/constraints), the port produces results that are:
    - **Functionally identical**: within defined tolerances for evaluation and optimization, and
    - **Constraint-identical**: respects constraints exactly, and
    - **Deterministic/seeded**: deterministic where baseline is deterministic; seeded where baseline uses randomness.
  - “Better” is allowed only if it is consistent, stable (repeatable), and does not violate invariants.
3. **Browser parity**
  - Runs fully **in-browser** (WASM) without requiring a server.
  - Heavy compute runs off the main thread (worker), with progress + cancel.

## Non-Goals (for parity milestone)

- Pixel-perfect UI replication
- New features (e.g., instrument-from-scratch wizard) unless explicitly added after parity
- Accounts/cloud sync (files remain local)

---

## Core Architecture Principles

### 1) Preserve the “Study Session” contract

All operations occur through a selection-driven session state machine:

- Selected Instrument + Tuning (+ Optimizer + Constraints) define what tools can run.
- Hole-count mismatch blocks tuning/optimization.
- Optimize produces a **new Instrument variant** (non-destructive) unless explicitly chosen otherwise.

### 2) Explicit geometry compilation step

Baseline behavior relies on a derived/compiled geometry representation.

In the port:

- `InstrumentRaw` = direct XML domain model
- `InstrumentCompiled` = output of `compile(InstrumentRaw)` (component chain, termination, headspace, ordering)
- All acoustics operate on compiled representation
- Objective evaluations mutate raw → compile → evaluate

This makes it structurally impossible to “forget updateComponents”.

### 3) Golden-fixture driven parity

We do not “approximate.” We reproduce baseline by:

- Generating **golden fixtures** from the oracle
- Matching those fixtures with layered tests
- Only then extending scope

---

## Constraints (applies to all study models)

Baseline supports constraints files and “Create default/blank constraints” generally.

- NAF makes constraints selection explicit in the Study tree.
- Other study models use defaults unless the user opens/creates constraints for the selected optimizer.

**Port rule:** constraints must be supported for **all** study models and attached to the currently selected optimizer.

---

## Load-Bearing Behavior: Fipple Factor (NAF)

### Must match baseline exactly

The following invariants are non-negotiable:

- Handling of `fippleFactor == null` matches baseline
- Scaling by windway height matches baseline
- Calibration objective semantics match baseline:
  - uses only the lowest note
  - 1D optimizer path
  - rebuild/compile after applying parameter change

### Guardrails

- Dedicated fixture suite for fipple factor
- No “improvements” to fipple behavior until full parity is achieved and protected

---

## Functional Parity Metrics

### A) Evaluation parity (predicted notes)

For each golden scenario:

- Predicted frequencies match within tolerance (measured in cents).
- If the study model provides min/max prediction (whistle/flute), those match too.

Initial target tolerances (refine after observing oracle stability):

- **≤ 0.5 cents** per fingering for evaluation

### B) Optimization parity

Optimization outcomes are equivalent if:

- All constraints are satisfied (hard requirement).
- Weighted notes meet tolerance:
  - **≤ 1.0 cents** difference for weighted notes, OR objective norm is **≤ oracle + epsilon**.

“Better” outcomes are allowed if:

- constraints are satisfied exactly,
- improvements are stable and repeatable,
- load-bearing invariants (esp. fipple suite) still pass.

### C) Determinism / randomness

- Deterministic runs must be reproducible: same inputs → same outputs.
- Multi-start/randomized strategies must use a fixed seed in dev/test harnesses.

---

## Multi-start & Two-stage Optimization Parity

Baseline includes multi-start optimization modes and a two-stage variant. The port must support:

- Multi-start enabled/disabled
- Deterministic seeding for reproducible development/testing
- Two-stage multi-start:
  - first-stage evaluator ranks/filters candidates
  - final optimization uses original evaluator
- Phase boundaries for reporting (global start point selection + refinement)

---

## Study-model-specific semantic validation

In addition to generic XML checks, enforce baseline semantic rules, e.g.:

- Reed study: mouthpiece position must be at the uppermost bore point position.

---

## Feature Parity Checklist

### Study Models

- NAF
- Whistle
- Flute (transverse)
- Reed

### Tools

- Calculate tuning (table)
- Graph tuning
- Graph note spectrum
- Supplementary info table
- Sketch instrument (visually equivalent)
- Compare instruments

### Optimization / Calibration

- Calibrator workflows per study model
- Optimizers per study model (including grouped-hole where applicable)
- Cancel + progress reporting
- Multi-start + two-stage modes

### File I/O / UX

- Load/save Instrument XML
- Load/save Tuning XML (including measured/min/max variants)
- Load/save Constraints XML
- Drag & drop open for instrument/tuning/constraints XML
- Tuning wizard equivalent (produces valid tuning XML)

---

## Browser Runtime Requirements

- Runs fully locally in-browser (WASM)
- Heavy computations run off main thread (worker)
- Progress streaming: evaluations/tunings counts + best-so-far objective
- Cancellation: responsive and returns partial result where baseline does

---

## Testing Strategy (Layered)

1. XML round-trip and semantic validation
2. Geometry compilation parity (component ordering/termination/headspace)
3. Impedance samples Z(f) parity
4. Predicted note parity (root finding)
5. Evaluator parity (cents + weights)
6. Objective parity (weighted SSE)
7. Optimization parity (end-to-end)
8. Tool outputs parity via numeric sampling (graphs/tables)

---

## Milestones

- [x] M0: Repo setup + specs + fixture plan + API shape
- [x] M1: Java golden harness + fixture suite v0 (NAF + fipple protected)
- [x] M2: NAF evaluation parity in Rust (76 tests, all 15 fingerings within 0.5 cents)
- [x] M3: NAF calibration + optimization parity (139 tests, fipple cal + BOBYQA hole optimization)
- [x] Post-M3: Expanded coverage to all NAF oracle XMLs (146 tests, 36 combos × 15 fingerings = 540 total)
- [x] M4: Browser-hosted MVP (NAF end-to-end)
- [x] M5: Full parity across all study models + tools (457 tests, 57 golden fixture sets, 5 analysis tools, tuning wizard)
  - [x] M5.1: Study model infrastructure refactor
  - [x] M5.2: Whistle evaluation parity (16 combos, 272 fingerings, 0.000002 cents)
  - [x] M5.3: Flute evaluation parity (8 combos, 110 fingerings, 0.058 cents)
  - [x] M5.4: Whistle calibration + optimization parity (3 calibrators + 3 hole optimizers)
  - [x] M5.5: Flute calibration + optimization (3 calibrators + 3 hole optimizers, golden-backed parity tests)
  - [x] M5.6: Reed evaluation model (7 combos, 72 fingerings, 0.012 cents)
  - [x] M5.7: Reed calibration + optimization (2D BOBYQA alpha/beta calibrator, 3 hole optimizers)

---

## Risks & Mitigations

- Floating point drift → use cents-based tolerances + Z-samples for anchoring
- Solver differences → prefer faithful solver implementations; validate with fixtures
- Performance in WASM → worker + throttled progress events + allocation control
- Hidden coupling in geometry compilation → explicit compile step + fixtures

