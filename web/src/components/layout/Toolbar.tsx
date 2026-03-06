import { Show, createSignal, createMemo } from "solid-js";
import { sessionStore } from "../../stores/session";
import OptimizeDialog from "../tools/OptimizeDialog";
import CompareDialog from "../tools/CompareDialog";
import GraphTuningDialog from "../tools/GraphTuningDialog";
import NoteSpectrumDialog from "../tools/NoteSpectrumDialog";
import WizardDialog from "../tools/WizardDialog";
import { openSketchPopup } from "../tools/SketchPopup";
import { openSupplementaryPopup } from "../tools/SupplementaryPopup";
import { getUseDirect } from "./SettingsDialog";
import type { OptimizeResult, CalibResult, TuningResult } from "../../types/session";

export default function Toolbar() {
  const [showOptDialog, setShowOptDialog] = createSignal(false);
  const [optResult, setOptResult] = createSignal<OptimizeResult | CalibResult | null>(null);
  const [showCompare, setShowCompare] = createSignal(false);
  const [showGraph, setShowGraph] = createSignal(false);
  const [showSpectrum, setShowSpectrum] = createSignal(false);
  const [showWizard, setShowWizard] = createSignal(false);

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
    const result = await sessionStore.evaluateTuning();
    if (result) setLastEval(result);
  }

  async function handleSupplementary() {
    const result = await sessionStore.supplementaryInfo();
    if (result) {
      const instName =
        sessionStore.instruments.find((d) => d.doc_id === sessionStore.selection()?.instrument_id)?.name ?? "Instrument";
      openSupplementaryPopup(result as any, instName);
    }
  }

  async function handleSpectrum() {
    let evalData = lastEval();
    if (!evalData) {
      evalData = await sessionStore.evaluateTuning();
      if (evalData) setLastEval(evalData);
    }
    if (evalData) setShowSpectrum(true);
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
            disabled={!sessionStore.canSketch()}
            onClick={async () => {
              const data = await sessionStore.sketchInstrument();
              if (data) openSketchPopup(data as any);
            }}
            title="Show instrument cross-section diagram"
          >
            Sketch
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!canCompare()}
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
            disabled={!sessionStore.canTune()}
            onClick={handleEvaluate}
            title={!sessionStore.canTune() ? "Select instrument and tuning first" : "Calculate tuning deviations for all fingerings"}
          >
            Evaluate
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canTune()}
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
            disabled={!sessionStore.canTune()}
            onClick={() => setShowGraph(true)}
            title="Plot impedance playing ranges"
          >
            Graph
          </button>
          <button
            class={btn}
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
            disabled={!sessionStore.canTune()}
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
            disabled={!sessionStore.canOptimize() || sessionStore.optimizing()}
            onClick={handleOptimize}
            title={
              !sessionStore.canOptimize()
                ? sessionStore.isFippleSelected()
                  ? "Select instrument and tuning to calibrate"
                  : "Select instrument, tuning, optimizer, and constraints"
                : sessionStore.isFippleSelected()
                  ? "Calibrate mouthpiece parameters"
                  : "Run hole position/size optimization"
            }
          >
            {sessionStore.isFippleSelected() ? "Calibrate" : "Optimize"}
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
        isFipple={sessionStore.isFippleSelected()}
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
        <CompareDialog onClose={() => setShowCompare(false)} />
      </Show>

      <Show when={showGraph()}>
        <GraphTuningDialog onClose={() => setShowGraph(false)} />
      </Show>

      <Show when={showSpectrum() && lastEval()}>
        <NoteSpectrumDialog
          onClose={() => setShowSpectrum(false)}
          notes={lastEval()!.rows.map((r) => ({ note: r.note, target_freq: r.target_freq }))}
        />
      </Show>

      <Show when={showWizard()}>
        <WizardDialog onClose={() => setShowWizard(false)} />
      </Show>
    </>
  );
}
