import { Show, For, createSignal, createMemo } from "solid-js";
import { sessionStore } from "../../stores/session";
import OptimizeDialog from "../tools/OptimizeDialog";
import SketchDialog from "../tools/SketchDialog";
import CompareDialog from "../tools/CompareDialog";
import GraphTuningDialog from "../tools/GraphTuningDialog";
import NoteSpectrumDialog from "../tools/NoteSpectrumDialog";
import WizardDialog from "../tools/WizardDialog";
import { openSupplementaryPopup } from "../tools/SupplementaryPopup";
import { getUseDirect } from "./SettingsDialog";
import type { OptimizeResult, CalibResult, TuningResult } from "../../types/session";

export default function StudyPanel() {
  const [showOptDialog, setShowOptDialog] = createSignal(false);
  const [optResult, setOptResult] = createSignal<OptimizeResult | CalibResult | null>(null);
  const [showSketch, setShowSketch] = createSignal(false);
  const [showCompare, setShowCompare] = createSignal(false);
  const [showGraph, setShowGraph] = createSignal(false);
  const [showSpectrum, setShowSpectrum] = createSignal(false);
  const [showWizard, setShowWizard] = createSignal(false);

  // Cache last eval result for spectrum fingering list
  const [lastEval, setLastEval] = createSignal<TuningResult | null>(null);

  const canCompare = createMemo(() => sessionStore.instruments.length >= 2);

  // Filter Global optimizers when DIRECT is disabled in settings
  const filteredOptimizers = createMemo(() => {
    const opts = sessionStore.optimizers();
    if (getUseDirect()) return opts;
    return opts.filter((o) => !o.key.startsWith("Global"));
  });

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
    // Need eval data for fingering list; run eval first if we don't have it cached
    let evalData = lastEval();
    if (!evalData) {
      evalData = await sessionStore.evaluateTuning();
      if (evalData) setLastEval(evalData);
    }
    if (evalData) setShowSpectrum(true);
  }

  // Shared button style
  const btnClass = "px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40";
  const btnPrimary = { background: "var(--color-accent)", color: "white" };
  const btnSecondary = { background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" };

  return (
    <aside
      class="w-56 border-r overflow-y-auto p-3 flex flex-col gap-4"
      style={{ background: "var(--color-surface)", "border-color": "var(--color-border)" }}
    >
      <Show when={!sessionStore.ready()} fallback={null}>
        <div class="text-sm" style={{ color: "var(--color-text-muted)" }}>
          Loading WASM...
        </div>
      </Show>

      <Show when={sessionStore.ready()}>
        {/* Instruments */}
        <DocSection
          title="Instruments"
          items={sessionStore.instruments}
          selectedId={sessionStore.selection()?.instrument_id ?? null}
          onSelect={(id) => sessionStore.selectInstrument(id)}
          onDoubleClick={(id, name) => sessionStore.openTab(id, "Instrument", name)}
          emptyText="No instruments loaded"
        />

        {/* Tunings */}
        <DocSection
          title="Tunings"
          items={sessionStore.tunings}
          selectedId={sessionStore.selection()?.tuning_id ?? null}
          onSelect={(id) => sessionStore.selectTuning(id)}
          onDoubleClick={(id, name) => sessionStore.openTab(id, "Tuning", name)}
          emptyText="No tunings loaded"
        />

        {/* Optimizers */}
        <section>
          <h2
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Optimizers
          </h2>
          <For each={filteredOptimizers()}>
            {(opt) => (
              <button
                class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
                classList={{ "font-semibold": sessionStore.selection()?.optimizer_key === opt.key }}
                style={{
                  background:
                    sessionStore.selection()?.optimizer_key === opt.key
                      ? "var(--color-accent)"
                      : "transparent",
                  color:
                    sessionStore.selection()?.optimizer_key === opt.key
                      ? "white"
                      : "var(--color-text)",
                }}
                onClick={() => sessionStore.selectOptimizer(opt.key)}
              >
                {opt.display_name}
              </button>
            )}
          </For>
        </section>

        {/* Constraints */}
        <DocSection
          title="Constraints"
          items={sessionStore.constraints}
          selectedId={sessionStore.selection()?.constraints_id ?? null}
          onSelect={(id) => sessionStore.selectConstraints(id)}
          onDoubleClick={(id, name) => sessionStore.openTab(id, "Constraints", name)}
          emptyText="No constraints loaded"
        />

        {/* Constraint creation buttons */}
        <Show when={sessionStore.canCreateConstraints()}>
          <div class="flex gap-2">
            <button
              class="flex-1 px-2 py-1 rounded text-xs font-medium transition-colors"
              style={btnSecondary}
              onClick={() => sessionStore.createDefaultConstraints()}
            >
              + Default
            </button>
            <button
              class="flex-1 px-2 py-1 rounded text-xs font-medium transition-colors"
              style={btnSecondary}
              onClick={() => sessionStore.createBlankConstraints()}
            >
              + Blank
            </button>
          </div>
        </Show>

        {/* ── Action buttons ─────────────────────────── */}
        <div
          class="mt-auto flex flex-col gap-2 pt-4 border-t"
          style={{ "border-color": "var(--color-border)" }}
        >
          {/* Instrument tools */}
          <div class="flex gap-2">
            <button
              class={`flex-1 ${btnClass}`}
              style={btnPrimary}
              disabled={!sessionStore.canSketch()}
              onClick={() => setShowSketch(true)}
              title={!sessionStore.canSketch() ? "Select an instrument first" : "Show instrument sketch"}
            >
              Sketch
            </button>
            <button
              class={`flex-1 ${btnClass}`}
              style={btnPrimary}
              disabled={!canCompare()}
              onClick={() => setShowCompare(true)}
              title={!canCompare() ? "Load at least 2 instruments" : "Compare two instruments"}
            >
              Compare
            </button>
          </div>

          {/* Tuning analysis */}
          <button
            class={btnClass}
            style={btnPrimary}
            disabled={!sessionStore.canTune()}
            onClick={handleEvaluate}
            title={!sessionStore.canTune() ? "Select an instrument and matching tuning first" : "Evaluate current tuning"}
          >
            Evaluate Tuning
          </button>

          <div class="flex gap-2">
            <button
              class={`flex-1 ${btnClass}`}
              style={btnSecondary}
              disabled={!sessionStore.canTune()}
              onClick={handleSupplementary}
              title="Supplementary acoustic info"
            >
              Supplementary
            </button>
          </div>

          {/* Impedance curves */}
          <div class="flex gap-2">
            <button
              class={`flex-1 ${btnClass}`}
              style={btnSecondary}
              disabled={!sessionStore.canTune()}
              onClick={() => setShowGraph(true)}
              title="Graph tuning playing ranges"
            >
              Graph
            </button>
            <button
              class={`flex-1 ${btnClass}`}
              style={btnSecondary}
              disabled={!sessionStore.canTune()}
              onClick={handleSpectrum}
              title="Note impedance spectrum"
            >
              Spectrum
            </button>
          </div>

          {/* Optimization */}
          <button
            class={btnClass}
            style={btnPrimary}
            disabled={!sessionStore.canOptimize() || sessionStore.optimizing()}
            onClick={handleOptimize}
            title={
              !sessionStore.canOptimize()
                ? sessionStore.isFippleSelected()
                  ? "Select instrument and tuning to calibrate"
                  : "Select instrument, tuning, optimizer, and constraints"
                : sessionStore.isFippleSelected()
                  ? "Calibrate fipple factor"
                  : "Run hole optimization"
            }
          >
            {sessionStore.isFippleSelected() ? "Calibrate" : "Optimize"}
          </button>

          {/* Wizard */}
          <button
            class={btnClass}
            style={btnSecondary}
            onClick={() => setShowWizard(true)}
            title="Generate tuning from temperament + scale"
          >
            Tuning Wizard
          </button>
        </div>
      </Show>

      {/* ── Dialogs ──────────────────────────── */}
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

      <Show when={showSketch()}>
        <SketchDialog onClose={() => setShowSketch(false)} />
      </Show>

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
    </aside>
  );
}

/** Reusable document list section with single-click select + double-click open tab. */
function DocSection(props: {
  title: string;
  items: { doc_id: number; name: string }[];
  selectedId: number | null;
  onSelect: (id: number) => void;
  onDoubleClick: (id: number, name: string) => void;
  emptyText: string;
}) {
  return (
    <section>
      <h2
        class="text-xs font-semibold uppercase tracking-wider mb-2"
        style={{ color: "var(--color-text-muted)" }}
      >
        {props.title}
      </h2>
      <Show when={props.items.length === 0}>
        <p class="text-xs" style={{ color: "var(--color-text-muted)" }}>
          {props.emptyText}
        </p>
      </Show>
      <For each={props.items}>
        {(doc) => (
          <button
            class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
            classList={{ "font-semibold": props.selectedId === doc.doc_id }}
            style={{
              background:
                props.selectedId === doc.doc_id ? "var(--color-accent)" : "transparent",
              color: props.selectedId === doc.doc_id ? "white" : "var(--color-text)",
            }}
            onClick={() => props.onSelect(doc.doc_id)}
            onDblClick={() => props.onDoubleClick(doc.doc_id, doc.name)}
          >
            {doc.name}
          </button>
        )}
      </For>
    </section>
  );
}
