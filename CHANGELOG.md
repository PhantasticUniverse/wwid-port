# Changelog

All notable changes to the WIDesigner port are documented here.

## [1.0.0] - 2026-03-09

Full parity with WIDesigner v2.6.0 across all four study models.

### Features
- **4 study models**: NAF, Whistle, Flute, Reed — evaluation, calibration, optimization
- **40+ optimizers**: hole position, hole size, hole groups, taper, bore diameter, merged, global (DIRECT-C + BOBYQA)
- **5 analysis tools**: Sketch, Compare, Supplementary Info, Graph Tuning (impedance pattern), Note Spectrum
- **Tuning wizard**: generate tunings from scales and temperaments
- **Browser-native**: runs entirely in-browser via WebAssembly, no server or Java needed
- **All tools in popup windows**: matching Java JFrame behavior

### Verification
- 454 tests, 57 golden fixture sets
- Evaluation parity: ≤0.058 cents across 994 fingerings
- 6 hostile parity audits, all clean

## [0.9.0] - 2026-03-06

Post-M5 polish, parity audits, and visual improvements.

### Added
- Visual parity: Graph Tuning markers, Note Spectrum gain coloring, Sketch engineering style
- UI/UX clarity: number formatting, Esc key on dialogs, tooltip coverage, toolbar reorganization
- All 6 tool dialogs converted to popup windows
- Sketch mouthpiece rendering (fipple, embouchure, reed)
- 6 hostile parity audits (30+ subsystems verified)

### Fixed
- ComputeService init hang
- evaluate_tuning NaN on missing frequency
- Popup closed guards
- Trust radius overrides
- Analysis tool frequency fallbacks

## [0.8.0] - 2026-03-04

M5 complete: all study models, optimizers, and analysis tools.

### Added
- Reed evaluation, calibration, and optimization (M5.6, M5.7)
- Flute calibration and optimization (M5.5)
- Whistle calibration and optimization (M5.4)
- DIRECT-C global optimizer and multi-start infrastructure
- NAF taper and grouped optimizers
- Bore optimizers (standalone, merged, global)
- Tuning wizard (scales, temperaments, pattern-based generation)
- 5 analysis tools with golden fixtures and parity tests
- Full frontend: all 4 study models, editors, optimization dialog

## [0.5.0] - 2026-03-02

M5.1–M5.3: study model infrastructure and multi-model evaluation.

### Added
- Study model infrastructure refactor (CalculatorParams, TerminationType, StudyKind)
- Whistle evaluation parity (0.000002 cents across 272 fingerings)
- Flute evaluation parity (0.058 cents across 110 fingerings)

## [0.4.0] - 2026-03-02

M4 complete: browser-hosted MVP (NAF end-to-end).

### Added
- wid-session crate (session orchestrator)
- wid-wasm crate (WASM bindings via Web Worker)
- SolidJS frontend: instrument/tuning/constraints editors
- Optimization and calibration UI
- Settings dialog, file handling, multi-file open

## [0.3.0] - 2026-03-02

M3 complete: NAF calibration and optimization parity.

### Added
- BOBYQA standalone crate
- Fipple factor calibration (1D Brent optimizer)
- Hole-from-top optimizer
- 139 tests passing

## [0.2.0] - 2026-03-02

M2 complete: NAF evaluation parity in Rust.

### Added
- Full acoustic pipeline (impedance, frequency prediction, cents deviation)
- 76 tests, all 15 NAF fingerings within 0.5 cents of Java oracle

## [0.1.0] - 2026-03-02

M0–M1: project setup and golden fixture infrastructure.

### Added
- Oracle + golden harness workflow
- Golden fixture scenarios and expected outputs
- Rust workspace skeleton (11 crates)
