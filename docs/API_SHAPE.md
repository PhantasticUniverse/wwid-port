# API_SHAPE.md

## Purpose

> **Status**: Core API implemented across all four study models (NAF, Whistle, Flute, Reed). Session lifecycle, document I/O, selection, evaluation, optimization, calibration, and constraints creation are all working end-to-end in the browser. 240 tests passing. Graph/spectrum/sketch/compare tools and advanced optimization modes (DIRECT-C, multi-start) are remaining M5 work.

Define a stable “StudySession” API that mirrors baseline StudyModel behavior:

* selection-driven tools
* gating rules
* outputs as structured data + XML artifacts
* cancellation + progress

Used by:

* browser worker host
* tests (headless)
* optional CLI

---

## Conceptual State Model

```
Session
  docs: { Instrument[], Tuning[], Constraints[], WizardComponents[] }
  selection:
    instrumentId?
    tuningId?
    optimizerKey?
    constraintsId? (optional; required for some NAF flows)
  options:
    units
    physical params
    blowing level
    spectrum multiplier
  multiStart:
    enabled?
    starts?
    seed?
    twoStage?
    firstStageEvaluatorKey?
```

### Gating (enabled operations)

* `canTune` = instrument && tuning && holeCountsMatch
* `canOptimize` = canTune && optimizerKey && (constraints attached if required by the selected optimizer/study)

---

## Commands (High-level)

### Session lifecycle

* `createSession(studyKind) -> sessionId`
* `resetSession() -> void`
* `getSessionState() -> snapshot (docs + selection + enabled tools)`

### Document I/O

* `openXml(xmlString) -> { docId, docKind, metadata }`
* `exportXml(docId) -> xmlString`
* `saveXml(docId, xmlString) -> validates and replaces doc`
* `listDocs(docKind?) -> [{docId, name, metadata}]`
* `deleteDoc(docId)`

### Additional doc kinds (wizard components)

Treat reusable tuning-wizard components as documents:

* `SymbolList`
* `Temperament`
* `ScaleIntervals` / `ScaleWithIntervals`
* `ScaleWithFrequencies`
* `FingeringPattern`
* `Tuning` (final)

`openXml/exportXml/saveXml` support these kinds.

### Selection

* `selectInstrument(docId)`
* `selectTuning(docId)`
* `selectOptimizer(optimizerKey)`
* `selectConstraints(docId)`  (for editing/viewing)
* `attachConstraintsToOptimizer(optimizerKey, docId)`
* `clearSelection(kind)`

### Constraints creation

* `createDefaultConstraints(optimizerKey) -> constraintsDocId`
* `createBlankConstraints(optimizerKey) -> constraintsDocId`
* `exportConstraintsBounds(constraintsDocId) -> { lower[], upper[], metadata }`

### Options

* `setUnits(lengthType)`
* `setPhysicalParams(temp, humidity, pressure?, co2?)`
* `setBlowingLevel(0..10)` (whistle/flute)
* `setSpectrumMultiplier(mult)`

---

## Tool Endpoints (Outputs)

### Evaluate tuning (table)

* `calculateTuning() -> TuningTableResult`

  * rows: `{ noteName, fingering, targetHz?, predictedHz?, cents?, minHz?, maxHz?, weight }`
  * summary: `{ meanAbsCents, rmsCents, maxAbsCents, noteCountUsed }`

### Graph tuning (curve samples)

* `graphTuning(fingeringIndices?, freqRange?) -> GraphCurves`

  * curves: `{ fingeringId, points:[{freqHz, y}] }`
  * metadata: axis labels, markers

### Note spectrum

* `noteSpectrum(fingeringIndex, freqRange?) -> SpectrumResult`

  * points: `{freqHz, impedanceRatio?, gain?, ...}`
  * resonance markers

### Supplementary info

* `supplementaryInfo() -> SupplementaryTable`

  * rows with derived quantities per note

### Sketch instrument (numeric geometry export)

* `sketchData() -> SketchGeometry`

  * bore polyline points
  * hole circles (pos, dia)
  * mouthpiece marker
  * termination marker

### Compare instruments

* `compareInstruments(instrumentA, instrumentB, tuningId?) -> CompareResult`

  * geometry deltas
  * tuning deltas

---

## Optimization / Calibration API

### Start optimization

* `optimizeStart({
    objectiveKey?,
    optimizerTypeOverride?,
    seed?,
    multiStart?: {
      enabled: boolean,
      starts?: number,
      seed?: number,
      twoStage?: boolean,
      firstStageEvaluatorKey?: string
    }
  }) -> runId`

### Progress events

* `optimizeStatus(runId) -> { state, evaluationsDone, tuningsDone, bestNormSoFar, elapsedMs, phase? }`

### Cancel

* `optimizeCancel(runId) -> void`

### Finish result

* `optimizeResult(runId) -> OptimizeResult`

  * status: `ok | cancelled_partial | cancelled | error`
  * `{ initialNorm, finalNorm, residualRatio, evaluationsDone, tuningsDone, elapsedMs }`
  * output: `{ newInstrumentDocId? , instrumentXml? }`

---

## Errors (structured)

* HoleCountMismatch
* InvalidXml(kind, details)
* MissingSelection(kind)
* UnsupportedObjectiveForStudy
* OptimizationCancelled
* NumericFailure(NoPlayingRange, RootNotFound, etc.)

---

## Stability Rules

* docId and runId are stable within a session
* outputs are deterministic under fixed seed/options
* constraints ordering is treated as ABI: objective parameterization must match constraints layout
