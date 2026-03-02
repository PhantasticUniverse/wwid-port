import { createSignal, createEffect, Show, For, on } from "solid-js";
import { createStore, reconcile } from "solid-js/store";
import { sessionStore } from "../../stores/session";
import type { TuningData } from "../../types/documents";

const EMPTY_TUNING: TuningData = {
  name: "",
  numberOfHoles: 0,
  fingering: [],
};

export default function TuningEditor(props: { docId: number }) {
  const [data, setData] = createStore<TuningData>({ ...EMPTY_TUNING });
  const [loaded, setLoaded] = createSignal(false);

  createEffect(
    on(
      () => props.docId,
      async (docId) => {
        setLoaded(false);
        const tuning = await sessionStore.getTuning(docId);
        setData(reconcile(tuning));
        setLoaded(true);
      }
    )
  );

  async function sync() {
    await sessionStore.setTuning(props.docId, structuredClone(data) as TuningData);
  }

  async function toggleHole(fingeringIdx: number, holeIdx: number) {
    const current = data.fingering[fingeringIdx].openHole[holeIdx];
    setData("fingering", fingeringIdx, "openHole", holeIdx, !current);
    await sync();
  }

  return (
    <Show when={loaded()} fallback={<p class="text-sm opacity-50">Loading...</p>}>
      <div class="flex flex-col gap-5 max-w-4xl">
        {/* Header */}
        <section class="flex flex-col gap-2">
          <div class="flex items-center gap-3">
            <label class="text-xs w-24" style={{ color: "var(--color-text-muted)" }}>
              Name
            </label>
            <input
              class="flex-1 px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.name}
              onInput={(e) => setData("name", e.currentTarget.value)}
              onBlur={sync}
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-24" style={{ color: "var(--color-text-muted)" }}>
              Comment
            </label>
            <input
              class="flex-1 px-2 py-1 rounded text-sm"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={data.comment ?? ""}
              onInput={(e) => setData("comment", e.currentTarget.value || undefined)}
              onBlur={sync}
            />
          </div>
          <div class="flex items-center gap-3">
            <label class="text-xs w-24" style={{ color: "var(--color-text-muted)" }}>
              # Holes
            </label>
            <span class="text-sm tabular-nums">{data.numberOfHoles}</span>
          </div>
        </section>

        {/* Fingering table */}
        <section>
          <h3
            class="text-xs font-semibold uppercase tracking-wider mb-2"
            style={{ color: "var(--color-text-muted)" }}
          >
            Fingerings
          </h3>
          <div class="overflow-x-auto">
            <table class="w-full text-xs border-collapse">
              <thead>
                <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                  <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Note
                  </th>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Frequency
                  </th>
                  {/* Hole columns — numbered from N down to 1 (matching Java display) */}
                  <For each={holeHeaders(data.numberOfHoles)}>
                    {(h) => (
                      <th
                        class="text-center py-1 px-1"
                        style={{ color: "var(--color-text-muted)", "min-width": "24px" }}
                      >
                        {h}
                      </th>
                    )}
                  </For>
                  <th class="text-right py-1 px-2" style={{ color: "var(--color-text-muted)" }}>
                    Weight
                  </th>
                </tr>
              </thead>
              <tbody>
                <For each={data.fingering}>
                  {(fing, fi) => (
                    <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                      <td class="py-1 px-2">{fing.note.name}</td>
                      <td class="py-1 px-2 text-right tabular-nums">
                        {fing.note.frequency?.toFixed(2) ?? "—"}
                      </td>
                      {/* Hole circles — display order: hole N, N-1, ..., 1 */}
                      <For each={[...fing.openHole].reverse()}>
                        {(isOpen, hi) => {
                          // Reverse index back to original
                          const realIdx = () => fing.openHole.length - 1 - hi();
                          return (
                            <td class="text-center py-1 px-1">
                              <button
                                class="w-5 h-5 rounded-full border-2 transition-colors cursor-pointer"
                                style={{
                                  "border-color": isOpen
                                    ? "var(--color-accent)"
                                    : "var(--color-text-muted)",
                                  background: isOpen ? "transparent" : "var(--color-text-muted)",
                                }}
                                title={isOpen ? "Open (click to close)" : "Closed (click to open)"}
                                onClick={() => toggleHole(fi(), realIdx())}
                              />
                            </td>
                          );
                        }}
                      </For>
                      <td class="py-1 px-2">
                        <input
                          type="number"
                          step="1"
                          class="w-12 px-1 py-0.5 rounded text-xs text-right tabular-nums"
                          style={{
                            background: "var(--color-surface-alt)",
                            border: "1px solid var(--color-border)",
                            color: "var(--color-text)",
                          }}
                          value={fing.optimizationWeight ?? 1}
                          onInput={(e) => {
                            const v = parseInt(e.currentTarget.value);
                            if (!isNaN(v))
                              setData("fingering", fi(), "optimizationWeight", v);
                          }}
                          onBlur={sync}
                        />
                      </td>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>
          </div>
        </section>
      </div>
    </Show>
  );
}

/** Generate hole header labels: N, N-1, ..., 1 */
function holeHeaders(n: number): string[] {
  const headers: string[] = [];
  for (let i = n; i >= 1; i--) {
    headers.push(String(i));
  }
  return headers;
}
