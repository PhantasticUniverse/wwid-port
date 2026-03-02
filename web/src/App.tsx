import { createSignal, onMount, Show, For } from "solid-js";
import { sessionStore } from "./stores/session";

export default function App() {
  const [fileContent, setFileContent] = createSignal<string | null>(null);

  onMount(() => {
    sessionStore.init();
  });

  async function handleFileOpen(event: Event) {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    const xml = await file.text();
    setFileContent(xml);
    await sessionStore.openXml(xml);
    input.value = "";
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    const file = event.dataTransfer?.files[0];
    if (!file) return;
    const xml = await file.text();
    setFileContent(xml);
    await sessionStore.openXml(xml);
  }

  function handleDragOver(event: DragEvent) {
    event.preventDefault();
  }

  return (
    <div
      class="min-h-screen flex flex-col"
      onDrop={handleDrop}
      onDragOver={handleDragOver}
    >
      {/* Top bar */}
      <header class="flex items-center justify-between px-4 py-2 border-b"
        style={{ "background": "var(--color-surface)", "border-color": "var(--color-border)" }}>
        <h1 class="text-lg font-semibold tracking-tight">WIDesigner</h1>
        <div class="flex items-center gap-3">
          <label class="px-3 py-1.5 rounded text-sm font-medium cursor-pointer transition-colors"
            style={{ "background": "var(--color-accent)", "color": "white" }}>
            Open File
            <input type="file" accept=".xml" class="hidden" onChange={handleFileOpen} />
          </label>
        </div>
      </header>

      {/* Main content */}
      <div class="flex flex-1 overflow-hidden">
        {/* Study Panel */}
        <aside class="w-56 border-r overflow-y-auto p-3 flex flex-col gap-4"
          style={{ "background": "var(--color-surface)", "border-color": "var(--color-border)" }}>

          <Show when={!sessionStore.ready()} fallback={null}>
            <div class="text-sm" style={{ "color": "var(--color-text-muted)" }}>
              Loading WASM...
            </div>
          </Show>

          <Show when={sessionStore.ready()}>
            {/* Instruments */}
            <section>
              <h2 class="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ "color": "var(--color-text-muted)" }}>Instruments</h2>
              <Show when={sessionStore.instruments.length === 0}>
                <p class="text-xs" style={{ "color": "var(--color-text-muted)" }}>No instruments loaded</p>
              </Show>
              <For each={sessionStore.instruments}>
                {(doc) => (
                  <button
                    class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
                    classList={{
                      "font-semibold": sessionStore.selection()?.instrument_id === doc.doc_id,
                    }}
                    style={{
                      "background": sessionStore.selection()?.instrument_id === doc.doc_id
                        ? "var(--color-accent)" : "transparent",
                      "color": sessionStore.selection()?.instrument_id === doc.doc_id
                        ? "white" : "var(--color-text)",
                    }}
                    onClick={() => sessionStore.selectInstrument(doc.doc_id)}
                  >
                    {doc.name}
                  </button>
                )}
              </For>
            </section>

            {/* Tunings */}
            <section>
              <h2 class="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ "color": "var(--color-text-muted)" }}>Tunings</h2>
              <Show when={sessionStore.tunings.length === 0}>
                <p class="text-xs" style={{ "color": "var(--color-text-muted)" }}>No tunings loaded</p>
              </Show>
              <For each={sessionStore.tunings}>
                {(doc) => (
                  <button
                    class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
                    classList={{
                      "font-semibold": sessionStore.selection()?.tuning_id === doc.doc_id,
                    }}
                    style={{
                      "background": sessionStore.selection()?.tuning_id === doc.doc_id
                        ? "var(--color-accent)" : "transparent",
                      "color": sessionStore.selection()?.tuning_id === doc.doc_id
                        ? "white" : "var(--color-text)",
                    }}
                    onClick={() => sessionStore.selectTuning(doc.doc_id)}
                  >
                    {doc.name}
                  </button>
                )}
              </For>
            </section>

            {/* Optimizers */}
            <section>
              <h2 class="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ "color": "var(--color-text-muted)" }}>Optimizers</h2>
              <For each={sessionStore.optimizers()}>
                {(opt) => (
                  <button
                    class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
                    classList={{
                      "font-semibold": sessionStore.selection()?.optimizer_key === opt.key,
                    }}
                    style={{
                      "background": sessionStore.selection()?.optimizer_key === opt.key
                        ? "var(--color-accent)" : "transparent",
                      "color": sessionStore.selection()?.optimizer_key === opt.key
                        ? "white" : "var(--color-text)",
                    }}
                    onClick={() => sessionStore.selectOptimizer(opt.key)}
                  >
                    {opt.display_name}
                  </button>
                )}
              </For>
            </section>

            {/* Constraints */}
            <section>
              <h2 class="text-xs font-semibold uppercase tracking-wider mb-2"
                style={{ "color": "var(--color-text-muted)" }}>Constraints</h2>
              <Show when={sessionStore.constraints.length === 0}>
                <p class="text-xs" style={{ "color": "var(--color-text-muted)" }}>No constraints loaded</p>
              </Show>
              <For each={sessionStore.constraints}>
                {(doc) => (
                  <button
                    class="w-full text-left px-2 py-1 rounded text-sm transition-colors"
                    classList={{
                      "font-semibold": sessionStore.selection()?.constraints_id === doc.doc_id,
                    }}
                    style={{
                      "background": sessionStore.selection()?.constraints_id === doc.doc_id
                        ? "var(--color-accent)" : "transparent",
                      "color": sessionStore.selection()?.constraints_id === doc.doc_id
                        ? "white" : "var(--color-text)",
                    }}
                    onClick={() => sessionStore.selectConstraints(doc.doc_id)}
                  >
                    {doc.name}
                  </button>
                )}
              </For>
            </section>

            {/* Action buttons */}
            <div class="mt-auto flex flex-col gap-2 pt-4 border-t"
              style={{ "border-color": "var(--color-border)" }}>
              <button
                class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
                style={{ "background": "var(--color-accent)", "color": "white" }}
                disabled={!sessionStore.canTune()}
                onClick={() => sessionStore.evaluateTuning()}
                title={!sessionStore.canTune() ? "Select an instrument and matching tuning first" : ""}
              >
                Evaluate Tuning
              </button>
            </div>
          </Show>
        </aside>

        {/* Workspace */}
        <main class="flex-1 overflow-auto p-4">
          <Show when={sessionStore.error()}>
            <div class="mb-4 p-3 rounded text-sm"
              style={{ "background": "rgba(239,68,68,0.1)", "color": "var(--color-error)", "border": "1px solid var(--color-error)" }}>
              {sessionStore.error()}
            </div>
          </Show>

          <Show when={sessionStore.lastEval()}>
            {(evalResult) => (
              <div>
                <h2 class="text-lg font-semibold mb-3">Evaluation Results</h2>
                <table class="w-full text-sm border-collapse">
                  <thead>
                    <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                      <th class="text-left py-2 px-3">Note</th>
                      <th class="text-right py-2 px-3">Target (Hz)</th>
                      <th class="text-right py-2 px-3">Predicted (Hz)</th>
                      <th class="text-right py-2 px-3">Deviation (cents)</th>
                      <th class="text-right py-2 px-3">Weight</th>
                    </tr>
                  </thead>
                  <tbody>
                    <For each={evalResult().rows}>
                      {(row) => {
                        const absCents = Math.abs(row.cents);
                        const color =
                          absCents < 5
                            ? "var(--color-success)"
                            : absCents < 15
                              ? "var(--color-warning)"
                              : "var(--color-error)";
                        return (
                          <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                            <td class="py-1.5 px-3">{row.note}</td>
                            <td class="text-right py-1.5 px-3">{row.target_freq.toFixed(2)}</td>
                            <td class="text-right py-1.5 px-3">{row.predicted_freq.toFixed(2)}</td>
                            <td class="text-right py-1.5 px-3 font-mono" style={{ color }}>
                              {row.cents >= 0 ? "+" : ""}
                              {row.cents.toFixed(2)}
                            </td>
                            <td class="text-right py-1.5 px-3">{row.weight}</td>
                          </tr>
                        );
                      }}
                    </For>
                  </tbody>
                  <tfoot>
                    <tr class="font-semibold" style={{ "border-top": "2px solid var(--color-border)" }}>
                      <td class="py-2 px-3" colSpan={3}>Net Error</td>
                      <td class="text-right py-2 px-3 font-mono">
                        {evalResult().net_error >= 0 ? "+" : ""}
                        {evalResult().net_error.toFixed(2)} cents
                      </td>
                      <td />
                    </tr>
                    <tr class="font-semibold">
                      <td class="py-2 px-3" colSpan={3}>Mean Deviation</td>
                      <td class="text-right py-2 px-3 font-mono">
                        {evalResult().mean_deviation.toFixed(2)} cents
                      </td>
                      <td />
                    </tr>
                  </tfoot>
                </table>
              </div>
            )}
          </Show>

          <Show when={!sessionStore.lastEval() && sessionStore.ready()}>
            <div class="flex flex-col items-center justify-center h-full opacity-40">
              <p class="text-lg mb-2">Drop XML files here to get started</p>
              <p class="text-sm">or use the Open File button</p>
            </div>
          </Show>
        </main>
      </div>

      {/* Console */}
      <div class="border-t overflow-y-auto"
        style={{
          "background": "var(--color-surface-alt)",
          "border-color": "var(--color-border)",
          "height": "120px",
          "min-height": "80px",
        }}>
        <div class="px-3 py-1 text-xs font-semibold uppercase tracking-wider"
          style={{ "color": "var(--color-text-muted)", "border-bottom": "1px solid var(--color-border)" }}>
          Console
        </div>
        <div class="px-3 py-1 font-mono text-xs leading-relaxed"
          style={{ "color": "var(--color-text-muted)" }}>
          <For each={sessionStore.consoleLogs}>
            {(line) => <div>{line}</div>}
          </For>
        </div>
      </div>
    </div>
  );
}
