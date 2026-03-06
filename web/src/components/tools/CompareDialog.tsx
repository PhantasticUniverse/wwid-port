import { Show, For, createSignal, onMount, onCleanup } from "solid-js";
import { sessionStore } from "../../stores/session";

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
  const [result, setResult] = createSignal<CompareResult | null>(null);
  const [loading, setLoading] = createSignal(false);

  async function runCompare() {
    if (oldId() < 0 || newId() < 0 || oldId() === newId()) return;
    setLoading(true);
    const r = await sessionStore.compareInstruments(oldId(), newId());
    if (r) setResult(r as CompareResult);
    setLoading(false);
  }

  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") props.onClose(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
    // Auto-compare on mount if both are pre-selected
    if (oldId() >= 0 && newId() >= 0 && oldId() !== newId()) {
      runCompare();
    }
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

        {/* Results table */}
        <Show when={loading()}>
          <div class="text-sm py-4" style={{ color: "var(--color-text-muted)" }}>Comparing...</div>
        </Show>
        <Show when={result()}>
          {(r) => <CompareTable result={r()} />}
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

function CompareTable(props: { result: CompareResult }) {
  const r = props.result;

  // Group rows by category
  let lastCategory = "";

  return (
    <div style={{ "max-height": "400px", "overflow-y": "auto" }}>
      <table class="w-full text-sm" style={{ "border-collapse": "collapse" }}>
        <thead>
          <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
            <th class="text-left px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>Category</th>
            <th class="text-left px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>Field</th>
            <th class="text-right px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>{r.old_name}</th>
            <th class="text-right px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>{r.new_name}</th>
            <th class="text-right px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>Diff</th>
            <th class="text-right px-2 py-1 text-xs font-semibold" style={{ color: "var(--color-text-muted)" }}>%</th>
          </tr>
        </thead>
        <tbody>
          <For each={r.rows}>
            {(row) => {
              const showCategory = row.category !== lastCategory;
              lastCategory = row.category;
              const pctColor = row.percent_change != null
                ? row.percent_change > 0 ? "#22c55e" : row.percent_change < 0 ? "#ef4444" : ""
                : "";

              return (
                <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                  <td class="px-2 py-1" style={{ color: "var(--color-text-muted)" }}>
                    {showCategory ? row.category : ""}
                  </td>
                  <td class="px-2 py-1">{row.field}</td>
                  <td class="px-2 py-1 text-right" style={{ "font-family": "monospace" }}>
                    {row.old_value != null ? fmtNum(row.old_value) : "—"}
                  </td>
                  <td class="px-2 py-1 text-right" style={{ "font-family": "monospace" }}>
                    {row.new_value != null ? fmtNum(row.new_value) : "—"}
                  </td>
                  <td class="px-2 py-1 text-right" style={{ "font-family": "monospace" }}>
                    {row.difference != null ? fmtDiff(row.difference) : "—"}
                  </td>
                  <td class="px-2 py-1 text-right" style={{ "font-family": "monospace", color: pctColor }}>
                    {row.percent_change != null ? `${row.percent_change >= 0 ? "+" : ""}${row.percent_change.toFixed(2)}%` : "—"}
                  </td>
                </tr>
              );
            }}
          </For>
        </tbody>
      </table>
    </div>
  );
}

function fmtNum(v: number): string {
  return Math.abs(v) < 0.001 ? v.toExponential(4) : v.toFixed(4);
}

function fmtDiff(v: number): string {
  const prefix = v >= 0 ? "+" : "";
  return prefix + fmtNum(v);
}
