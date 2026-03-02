import { createSignal, onMount, Show, For } from "solid-js";
import { sessionStore } from "./stores/session";

export default function App() {
  const [showSettings, setShowSettings] = createSignal(false);
  const [settingsTemp, setSettingsTemp] = createSignal(20.0);
  const [settingsHumidity, setSettingsHumidity] = createSignal(45.0);

  onMount(() => {
    sessionStore.init();
  });

  async function handleFileOpen(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (!files) return;
    for (const file of Array.from(files)) {
      const xml = await file.text();
      await sessionStore.openXml(xml);
    }
    input.value = "";
  }

  async function handleDrop(event: DragEvent) {
    event.preventDefault();
    const files = event.dataTransfer?.files;
    if (!files) return;
    for (const file of Array.from(files)) {
      if (file.name.endsWith(".xml")) {
        const xml = await file.text();
        await sessionStore.openXml(xml);
      }
    }
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
            <input type="file" accept=".xml" multiple class="hidden" onChange={handleFileOpen} />
          </label>
          <button
            class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
            style={{ "background": "var(--color-accent)", "color": "white" }}
            disabled={!sessionStore.selection()?.instrument_id}
            onClick={() => {
              const id = sessionStore.selection()?.instrument_id;
              if (id != null) sessionStore.saveInstrumentXml(id);
            }}
            title={!sessionStore.selection()?.instrument_id ? "Select an instrument first" : "Save selected instrument as XML"}
          >
            Save
          </button>
          <button
            class="px-2 py-1.5 rounded text-sm transition-colors"
            style={{ "color": "var(--color-text-muted)" }}
            onClick={() => {
              // Sync settings fields with current params before opening
              const p = sessionStore.params();
              if (p) {
                setSettingsTemp(p.temperature);
                setSettingsHumidity(p.humidity);
              }
              setShowSettings(true);
            }}
            title="Settings"
          >
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 4.754a3.246 3.246 0 1 0 0 6.492 3.246 3.246 0 0 0 0-6.492zM5.754 8a2.246 2.246 0 1 1 4.492 0 2.246 2.246 0 0 1-4.492 0z"/>
              <path d="M9.796 1.343c-.527-1.79-3.065-1.79-3.592 0l-.094.319a.873.873 0 0 1-1.255.52l-.292-.16c-1.64-.892-3.433.902-2.54 2.541l.159.292a.873.873 0 0 1-.52 1.255l-.319.094c-1.79.527-1.79 3.065 0 3.592l.319.094a.873.873 0 0 1 .52 1.255l-.16.292c-.892 1.64.901 3.434 2.541 2.54l.292-.159a.873.873 0 0 1 1.255.52l.094.319c.527 1.79 3.065 1.79 3.592 0l.094-.319a.873.873 0 0 1 1.255-.52l.292.16c1.64.893 3.434-.902 2.54-2.541l-.159-.292a.873.873 0 0 1 .52-1.255l.319-.094c1.79-.527 1.79-3.065 0-3.592l-.319-.094a.873.873 0 0 1-.52-1.255l.16-.292c.893-1.64-.902-3.433-2.541-2.54l-.292.159a.873.873 0 0 1-1.255-.52l-.094-.319zm-2.633.283c.246-.835 1.428-.835 1.674 0l.094.319a1.873 1.873 0 0 0 2.693 1.115l.291-.16c.764-.415 1.6.42 1.184 1.185l-.159.292a1.873 1.873 0 0 0 1.116 2.692l.318.094c.835.246.835 1.428 0 1.674l-.319.094a1.873 1.873 0 0 0-1.115 2.693l.16.291c.415.764-.421 1.6-1.185 1.184l-.291-.159a1.873 1.873 0 0 0-2.693 1.116l-.094.318c-.246.835-1.428.835-1.674 0l-.094-.319a1.873 1.873 0 0 0-2.692-1.115l-.292.16c-.764.415-1.6-.421-1.184-1.185l.159-.291A1.873 1.873 0 0 0 1.945 8.93l-.319-.094c-.835-.246-.835-1.428 0-1.674l.319-.094A1.873 1.873 0 0 0 3.06 4.377l-.16-.292c-.415-.764.42-1.6 1.185-1.184l.292.159a1.873 1.873 0 0 0 2.692-1.116l.094-.318z"/>
            </svg>
          </button>
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
                disabled={!sessionStore.canSketch()}
                title={!sessionStore.canSketch() ? "Select an instrument first" : "Show instrument sketch"}
              >
                Sketch
              </button>
              <button
                class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
                style={{ "background": "var(--color-accent)", "color": "white" }}
                disabled={!sessionStore.canTune()}
                onClick={() => sessionStore.evaluateTuning()}
                title={!sessionStore.canTune() ? "Select an instrument and matching tuning first" : "Evaluate current tuning"}
              >
                Evaluate Tuning
              </button>
              <button
                class="px-3 py-1.5 rounded text-sm font-medium transition-colors disabled:opacity-40"
                style={{ "background": "var(--color-accent)", "color": "white" }}
                disabled={!sessionStore.canOptimize()}
                title={!sessionStore.canOptimize() ? "Select instrument, tuning, optimizer, and constraints" : "Run optimization"}
              >
                Optimize
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

      {/* Settings Dialog */}
      <Show when={showSettings()}>
        <div class="fixed inset-0 flex items-center justify-center"
          style={{ "background": "rgba(0,0,0,0.4)", "z-index": "50" }}
          onClick={(e) => { if (e.target === e.currentTarget) setShowSettings(false); }}>
          <div class="rounded-lg shadow-lg p-6 w-96"
            style={{ "background": "var(--color-surface)", "border": "1px solid var(--color-border)" }}>
            <h2 class="text-lg font-semibold mb-4">Settings</h2>

            <div class="flex flex-col gap-4">
              {/* Temperature */}
              <div class="flex items-center justify-between">
                <label class="text-sm" style={{ "color": "var(--color-text)" }}>
                  Temperature, C:
                </label>
                <input
                  type="number"
                  step="0.1"
                  class="w-24 px-2 py-1 rounded text-sm text-right"
                  style={{
                    "background": "var(--color-surface-alt)",
                    "border": "1px solid var(--color-border)",
                    "color": "var(--color-text)",
                  }}
                  value={settingsTemp()}
                  onInput={(e) => setSettingsTemp(parseFloat(e.currentTarget.value) || 0)}
                />
              </div>

              {/* Humidity */}
              <div class="flex items-center justify-between">
                <label class="text-sm" style={{ "color": "var(--color-text)" }}>
                  Relative Humidity, %:
                </label>
                <input
                  type="number"
                  step="1"
                  min="0"
                  max="100"
                  class="w-24 px-2 py-1 rounded text-sm text-right"
                  style={{
                    "background": "var(--color-surface-alt)",
                    "border": "1px solid var(--color-border)",
                    "color": "var(--color-text)",
                  }}
                  value={settingsHumidity()}
                  onInput={(e) => setSettingsHumidity(parseFloat(e.currentTarget.value) || 0)}
                />
              </div>
            </div>

            {/* Buttons */}
            <div class="flex justify-end gap-2 mt-6">
              <button
                class="px-3 py-1.5 rounded text-sm"
                style={{ "color": "var(--color-text-muted)" }}
                onClick={() => setShowSettings(false)}
              >
                Cancel
              </button>
              <button
                class="px-3 py-1.5 rounded text-sm font-medium"
                style={{ "background": "var(--color-accent)", "color": "white" }}
                onClick={async () => {
                  await sessionStore.updateParams(settingsTemp(), settingsHumidity());
                  setShowSettings(false);
                }}
              >
                Apply
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}
