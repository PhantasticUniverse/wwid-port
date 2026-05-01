import { Show, createSignal, createMemo } from "solid-js";
import { sessionStore } from "../../stores/session";
import OptimizeDialog from "../tools/OptimizeDialog";
import CompareDialog from "../tools/CompareDialog";
import WizardDialog from "../tools/WizardDialog";
import { openSketchPopup } from "../tools/SketchPopup";
import { openSupplementaryPopup } from "../tools/SupplementaryPopup";
import { openGraphTuningPopup } from "../tools/GraphTuningPopup";
import { openNoteSpectrumPopup } from "../tools/NoteSpectrumPopup";
import { openEvalPopup } from "../tools/EvalPopup";
import ToolDockDialog, {
  type GraphTuningResult,
  type NoteSpectrumResult,
  type SupplementaryResult,
  type ToolDockContent,
} from "../tools/ToolDockDialog";
import type { SketchData } from "../tools/SketchPopup";
import { getOutputMode } from "./SettingsDialog";
import type { OptimizeResult, CalibResult, TuningResult } from "../../types/session";

export default function Toolbar() {
  const [showOptDialog, setShowOptDialog] = createSignal(false);
  const [optResult, setOptResult] = createSignal<OptimizeResult | CalibResult | null>(null);
  const [showCompare, setShowCompare] = createSignal(false);
  const [showWizard, setShowWizard] = createSignal(false);
  const [toolDock, setToolDock] = createSignal<ToolDockContent | null>(null);

  const [lastEval, setLastEval] = createSignal<TuningResult | null>(null);

  const canCompare = createMemo(() => sessionStore.instruments.length >= 2);

  async function handleOptimize() {
    setOptResult(null);
    setShowOptDialog(true);
    const result = await sessionStore.runOptimize();
    if (result) {
      setOptResult(result);
    } else {
      setShowOptDialog(false);
    }
  }

  async function handleEvaluate() {
    const result = await sessionStore.evaluateTuning(false);
    if (result) {
      setLastEval(result);
      if (getOutputMode() === "dock") {
        const instName =
          sessionStore.instruments.find((d) => d.doc_id === sessionStore.selection()?.instrument_id)?.name ?? "Instrument";
        setToolDock({ kind: "eval", result, instrumentName: instName });
      } else {
        const instName =
          sessionStore.instruments.find((d) => d.doc_id === sessionStore.selection()?.instrument_id)?.name ?? "Instrument";
        // If the browser blocks Java-style popups, fall back to the in-app dock.
        const opened = openEvalPopup(result, instName);
        if (!opened) setToolDock({ kind: "eval", result, instrumentName: instName });
      }
    }
  }

  async function handleSupplementary() {
    const result = await sessionStore.supplementaryInfo();
    if (result) {
      const instName =
        sessionStore.instruments.find((d) => d.doc_id === sessionStore.selection()?.instrument_id)?.name ?? "Instrument";
      const supplementary = result as SupplementaryResult;
      if (getOutputMode() === "dock") {
        setToolDock({ kind: "supplementary", result: supplementary, instrumentName: instName });
      } else {
        const opened = openSupplementaryPopup(supplementary, instName);
        if (!opened) setToolDock({ kind: "supplementary", result: supplementary, instrumentName: instName });
      }
    }
  }

  async function handleGraph() {
    const result = await sessionStore.graphTuning();
    if (result) {
      const graph = result as GraphTuningResult;
      if (getOutputMode() === "dock") {
        setToolDock({ kind: "graph", result: graph });
      } else {
        const opened = openGraphTuningPopup(graph);
        if (!opened) setToolDock({ kind: "graph", result: graph });
      }
    }
  }

  async function handleSpectrum() {
    let evalData = lastEval();
    if (!evalData) {
      evalData = await sessionStore.evaluateTuning(false);
      if (evalData) setLastEval(evalData);
    }
    if (evalData && evalData.rows.length > 0) {
      if (getOutputMode() === "dock") {
        const spectrum = await sessionStore.noteSpectrum(0);
        if (spectrum) setToolDock({ kind: "spectrum", result: spectrum as NoteSpectrumResult });
      } else {
        const opened = openNoteSpectrumPopup(evalData.rows.map((r) => ({ note: r.note, target_freq: r.target_freq })));
        if (!opened) {
          const spectrum = await sessionStore.noteSpectrum(0);
          if (spectrum) setToolDock({ kind: "spectrum", result: spectrum as NoteSpectrumResult });
        }
      }
    }
  }

  const btn = "px-3 py-1 rounded text-xs font-medium transition-colors disabled:opacity-40 whitespace-nowrap";

  return (
    <>
      <Show when={sessionStore.ready()}>
        <div
          class="flex items-center gap-1 px-3 py-1.5 border-b overflow-x-auto"
          style={{
            background: "var(--color-surface)",
            "border-color": "var(--color-border)",
            "min-height": "36px",
          }}
        >
          {/* Instrument tools */}
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canSketch() || sessionStore.loading()}
            onClick={async () => {
              const data = await sessionStore.sketchInstrument();
              if (data) {
                const sketch = data as SketchData;
                if (getOutputMode() === "dock") {
                  setToolDock({ kind: "sketch", result: sketch });
                } else {
                  const opened = openSketchPopup(sketch);
                  if (!opened) setToolDock({ kind: "sketch", result: sketch });
                }
              }
            }}
            title="Show instrument cross-section diagram"
          >
            Sketch
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!canCompare() || sessionStore.loading()}
            onClick={() => setShowCompare(true)}
            title={!canCompare() ? "Load at least 2 instruments to compare" : "Compare two instruments side by side"}
          >
            Compare
          </button>

          <div class="w-px h-5 mx-1" style={{ background: "var(--color-border)" }} />

          {/* Tuning analysis */}
          <button
            class={btn}
            style={{ background: "var(--color-accent)", color: "white" }}
            disabled={!sessionStore.canTune() || sessionStore.loading()}
            onClick={handleEvaluate}
            title={!sessionStore.canTune() ? "Select instrument and tuning first" : "Calculate tuning deviations for all fingerings"}
          >
            Evaluate
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canTune() || sessionStore.loading()}
            onClick={handleSupplementary}
            title="Show supplementary acoustic information"
          >
            Supplementary
          </button>

          <div class="w-px h-5 mx-1" style={{ background: "var(--color-border)" }} />

          {/* Impedance curves */}
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canTune() || sessionStore.loading()}
            onClick={handleGraph}
            title="Plot impedance playing ranges"
          >
            Graph
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canTune() || sessionStore.loading()}
            onClick={handleSpectrum}
            title="Show note impedance spectrum"
          >
            Spectrum
          </button>

          <div class="w-px h-5 mx-1" style={{ background: "var(--color-border)" }} />

          {/* Optimization */}
          <button
            class={btn}
            style={{ background: "var(--color-accent)", color: "white" }}
            disabled={!sessionStore.canOptimize() || sessionStore.optimizing() || sessionStore.loading()}
            onClick={handleOptimize}
            title={
              !sessionStore.canOptimize()
                ? sessionStore.isCalibratorSelected()
                  ? "Select instrument and tuning to calibrate"
                  : "Select instrument, tuning, optimizer, and constraints"
                : sessionStore.isCalibratorSelected()
                  ? "Calibrate mouthpiece parameters"
                  : "Run hole position/size optimization"
            }
          >
            {sessionStore.isCalibratorSelected() ? "Calibrate" : "Optimize"}
          </button>

          {/* Wizard */}
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            onClick={() => setShowWizard(true)}
            title="Generate tuning from temperament and scale"
          >
            Wizard
          </button>
        </div>
      </Show>

      {/* Dialogs */}
      <OptimizeDialog
        open={showOptDialog()}
        isCalibration={sessionStore.isCalibratorSelected()}
        progress={sessionStore.optProgress()}
        result={optResult()}
        onCancel={() => {
          sessionStore.cancelOptimize();
          setShowOptDialog(false);
        }}
        onClose={() => {
          setShowOptDialog(false);
          setOptResult(null);
        }}
      />

      <Show when={showCompare()}>
        <CompareDialog
          onClose={() => setShowCompare(false)}
          onDock={(result) => setToolDock({ kind: "compare", result })}
        />
      </Show>

      <Show when={showWizard()}>
        <WizardDialog onClose={() => setShowWizard(false)} />
      </Show>

      <Show when={toolDock()}>
        {(content) => (
          <ToolDockDialog content={content()} onClose={() => setToolDock(null)} />
        )}
      </Show>
    </>
  );
}
