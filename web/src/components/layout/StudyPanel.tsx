import { Show, For, createMemo } from "solid-js";
import { sessionStore } from "../../stores/session";
import { getUseDirect } from "./SettingsDialog";

export default function StudyPanel() {
  // Filter Global optimizers when DIRECT is disabled in settings
  const filteredOptimizers = createMemo(() => {
    const opts = sessionStore.optimizers();
    if (getUseDirect()) return opts;
    return opts.filter((o) => !o.key.startsWith("Global"));
  });

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
                title={opt.display_name}
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
              title="Create constraints with default bounds for selected optimizer"
            >
              + Default
            </button>
            <button
              class="flex-1 px-2 py-1 rounded text-xs font-medium transition-colors"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", border: "1px solid var(--color-border)" }}
              onClick={() => sessionStore.createBlankConstraints()}
              title="Create constraints with empty bounds for selected optimizer"
            >
              + Blank
            </button>
          </div>
        </Show>
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
            title={doc.name}
          >
            {doc.name}
          </button>
        )}
      </For>
    </section>
  );
}
