import { Show, For, createSignal } from "solid-js";
import { sessionStore } from "../../stores/session";
import OptimizeDialog from "../tools/OptimizeDialog";
import type { OptimizeResult, CalibResult } from "../../types/session";

export default function StudyPanel() {
  const [showOptDialog, setShowOptDialog] = createSignal(false);
  const [optResult, setOptResult] = createSignal<OptimizeResult | CalibResult | null>(null);

  async function handleOptimize() {
    setOptResult(null);
    setShowOptDialog(true);
    const result = await sessionStore.runOptimize();
    if (result) {
      setOptResult(result);
    } else {
      // Cancelled or errored — close dialog
      setShowOptDialog(false);
    }
  }

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
          <For each={sessionStore.optimizers()}>
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
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              onClick={() => sessionStore.createDefaultConstraints()}
            >
              + Default
            </button>
            <button
              class="flex-1 px-2 py-1 rounded text-xs font-medium transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              onClick={() => sessionStore.createBlankConstraints()}
            >
              + Blank
            </button>
          </div>
        </Show>

        {/* Action buttons */}
        <div
          class="mt-auto flex flex-col gap-2 pt-4 border-t"
          style={{ "border-color": "var(--color-border)" }}
        >
          <button
            class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
            style={{ background: "var(--color-accent)", color: "white" }}
            disabled={!sessionStore.canSketch()}
            onClick={() => sessionStore.log("Sketch is not yet implemented (M5)")}
            title={
              !sessionStore.canSketch() ? "Select an instrument first" : "Show instrument sketch"
            }
          >
            Sketch
          </button>
          <button
            class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
            style={{ background: "var(--color-accent)", color: "white" }}
            disabled={!sessionStore.canTune()}
            onClick={() => sessionStore.evaluateTuning()}
            title={
              !sessionStore.canTune()
                ? "Select an instrument and matching tuning first"
                : "Evaluate current tuning"
            }
          >
            Evaluate Tuning
          </button>
          <button
            class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
            style={{ background: "var(--color-accent)", color: "white" }}
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
        </div>
      </Show>

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
