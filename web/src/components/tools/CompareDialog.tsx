import { Show, For, createSignal, onMount, onCleanup } from "solid-js";
import { sessionStore } from "../../stores/session";
import { openComparePopup } from "./ComparePopup";

interface CompareRow {
  category: string;
  field: string;
  old_value: number | null;
  new_value: number | null;
  difference: number | null;
  percent_change: number | null;
}

interface CompareResult {
  old_name: string;
  new_name: string;
  rows: CompareRow[];
}

export default function CompareDialog(props: { onClose: () => void }) {
  const instruments = () => sessionStore.instruments;

  // Pre-select: if 2+ instruments, pick first two (most recent optimization = last two)
  const defaultOld = () => instruments().length >= 2 ? instruments()[instruments().length - 2].doc_id : instruments()[0]?.doc_id ?? -1;
  const defaultNew = () => instruments().length >= 2 ? instruments()[instruments().length - 1].doc_id : instruments()[0]?.doc_id ?? -1;

  const [oldId, setOldId] = createSignal(defaultOld());
  const [newId, setNewId] = createSignal(defaultNew());
  const [loading, setLoading] = createSignal(false);

  async function runCompare() {
    if (oldId() < 0 || newId() < 0 || oldId() === newId()) return;
    setLoading(true);
    const r = await sessionStore.compareInstruments(oldId(), newId());
    setLoading(false);
    if (r) {
      openComparePopup(r as CompareResult);
      props.onClose();
    }
  }

  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") props.onClose(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  return (
    <div
      class="fixed inset-0 flex items-center justify-center"
      style={{ background: "rgba(0,0,0,0.4)", "z-index": "50" }}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <div
        class="rounded-lg shadow-lg p-6"
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          width: "720px",
          "max-height": "90vh",
          "overflow-y": "auto",
        }}
      >
        <h2 class="text-lg font-semibold mb-3">Compare Instruments</h2>

        {/* Instrument selectors */}
        <div class="flex gap-4 mb-4">
          <div class="flex-1">
            <label class="text-xs mb-1 block" style={{ color: "var(--color-text-muted)" }}>Old (baseline)</label>
            <select
              class="w-full px-2 py-1 rounded text-sm border"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", "border-color": "var(--color-border)" }}
              value={oldId()}
              onChange={(e) => setOldId(parseInt(e.currentTarget.value))}
            >
              <For each={instruments()}>
                {(inst) => <option value={inst.doc_id}>{inst.name}</option>}
              </For>
            </select>
          </div>
          <div class="flex-1">
            <label class="text-xs mb-1 block" style={{ color: "var(--color-text-muted)" }}>New (modified)</label>
            <select
              class="w-full px-2 py-1 rounded text-sm border"
              style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", "border-color": "var(--color-border)" }}
              value={newId()}
              onChange={(e) => setNewId(parseInt(e.currentTarget.value))}
            >
              <For each={instruments()}>
                {(inst) => <option value={inst.doc_id}>{inst.name}</option>}
              </For>
            </select>
          </div>
          <div class="flex items-end">
            <button
              class="px-3 py-1 rounded text-sm font-medium"
              style={{ background: "var(--color-accent)", color: "white" }}
              disabled={loading() || oldId() === newId()}
              onClick={runCompare}
            >
              Compare
            </button>
          </div>
        </div>

        <Show when={loading()}>
          <div class="text-sm py-4" style={{ color: "var(--color-text-muted)" }}>Comparing...</div>
        </Show>

        <div class="flex justify-end mt-4">
          <button
            class="px-4 py-1.5 rounded text-sm font-medium"
            style={{ background: "var(--color-accent)", color: "white" }}
            onClick={props.onClose}
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

