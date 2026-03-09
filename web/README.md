# WIDesigner Web Frontend

Browser-based UI for the WIDesigner port, built with SolidJS and backed by Rust/WASM computation.

## Stack

- **SolidJS 1.9** — reactive UI framework
- **Vite 6** — build tool and dev server
- **Tailwind CSS v4** — utility-first styling
- **Chart.js** — impedance and spectrum charts
- **Rust/WASM** — acoustic computation via Web Worker

## Architecture

```
App.tsx                         Root component
├── layout/
│   ├── Toolbar.tsx             Top bar: file actions, study model selector
│   ├── StudyPanel.tsx          Right sidebar: tool buttons, optimizer selector
│   ├── Workspace.tsx           Tab-based document editor area
│   └── ConsolePanel.tsx        Bottom: log output
│   └── SettingsDialog.tsx      Settings modal (temperature, humidity, etc.)
├── editors/
│   ├── InstrumentEditor.tsx    Bore points, holes, mouthpiece params
│   ├── TuningEditor.tsx        Note table with fingering patterns
│   └── ConstraintsEditor.tsx   Lower/upper bounds editor
├── tools/
│   ├── EvalPopup.ts            Evaluation results (popup window)
│   ├── SketchPopup.ts          Instrument sketch (popup window)
│   ├── ComparePopup.ts         Compare two instruments (popup window)
│   ├── SupplementaryPopup.ts   Supplementary info table (popup window)
│   ├── GraphTuningPopup.ts     Impedance pattern chart (popup window)
│   ├── NoteSpectrumPopup.ts    Note spectrum chart (popup window)
│   ├── OptimizeDialog.tsx      Optimization/calibration modal
│   ├── CompareDialog.tsx       Compare instrument selector
│   └── WizardDialog.tsx        Tuning wizard modal
├── shared/
│   └── NumberField.tsx         Numeric input component
├── services/
│   └── ComputeService.ts       Web Worker lifecycle manager
├── worker/
│   └── compute-worker.ts       Web Worker: loads WASM, dispatches commands
└── stores/
    └── session.ts              Reactive session state (SolidJS store)
```

## WASM integration

The `web/wasm` symlink points to `wid/crates/wid-wasm/pkg/`, where `wasm-bindgen` outputs the compiled WASM module and JS glue.

**Data flow:**
1. Frontend calls `sessionStore.evaluateTuning()` (or similar)
2. `ComputeService` posts a JSON message to the Web Worker
3. Worker calls `wid_wasm.execute(json)` → Rust `StudySession` processes it
4. Result JSON is posted back to the main thread
5. SolidJS store updates, UI re-renders

For heavy compute (optimization), the worker uses `wid_wasm.optimize(callback)` with progress streaming.

## Popup windows

All 6 tool dialogs (Evaluate, Sketch, Compare, Supplementary, Graph Tuning, Note Spectrum) open in separate browser windows, matching Java WIDesigner's JFrame behavior. Chart.js renders into popup canvases via same-origin JavaScript context sharing.

## Development

```bash
npm install          # Install dependencies (first time)
npx vite             # Dev server at http://localhost:5173
npx vite build       # Production build to web/dist/
```

### Rebuilding WASM

After changing Rust code, rebuild before testing:

```bash
cd ../wid
cargo build --target wasm32-unknown-unknown --release -p wid-wasm
wasm-bindgen target/wasm32-unknown-unknown/release/wid_wasm.wasm \
  --out-dir crates/wid-wasm/pkg --target web
```
