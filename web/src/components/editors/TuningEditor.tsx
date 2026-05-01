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
                  <th class="text-left py-1 px-2" style={{ color: "var(--color-text-muted)", "min-width": "190px" }}>
                    Fingering
                    <div class="text-[10px] font-normal normal-case tracking-normal">
                      mouthpiece → foot
                    </div>
                  </th>
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
                      <td class="py-1 px-2">
                        <NafFingeringChart
                          holes={fing.openHole}
                          onToggle={(holeIdx) => toggleHole(fi(), holeIdx)}
                        />
                      </td>
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

function NafFingeringChart(props: {
  holes: boolean[];
  onToggle: (holeIdx: number) => void;
}) {
  const width = 174;
  const height = 30;
  const boreY = 15;
  const startX = 38;
  const endX = 158;
  const spacing = props.holes.length > 1 ? (endX - startX) / (props.holes.length - 1) : 0;

  return (
    <svg
      width={width}
      height={height}
      viewBox={`0 0 ${width} ${height}`}
      role="img"
      aria-label={`NAF fingering ${props.holes.map((open) => (open ? "open" : "closed")).join(", ")}`}
    >
      <path
        d="M6 15 L24 6 L162 6 Q169 6 169 15 Q169 24 162 24 L24 24 Z"
        fill="var(--color-field)"
        stroke="var(--color-text)"
        stroke-width="1.4"
      />
      <path
        d="M22 15 L162 15"
        stroke="var(--color-border-strong)"
        stroke-width="1"
        opacity="0.55"
      />
      <For each={props.holes}>
        {(isOpen, idx) => {
          const x = () => startX + idx() * spacing;
          return (
            <g
              role="button"
              tabindex="0"
              aria-label={`Hole ${idx() + 1}: ${isOpen ? "open" : "closed"}`}
              onClick={() => props.onToggle(idx())}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  props.onToggle(idx());
                }
              }}
              style={{ cursor: "pointer" }}
            >
              <circle cx={x()} cy={boreY} r="8" fill="transparent" />
              <circle
                cx={x()}
                cy={boreY}
                r="4.8"
                fill={isOpen ? "var(--color-field)" : "var(--color-text)"}
                stroke="var(--color-text)"
                stroke-width="1.2"
              />
            </g>
          );
        }}
      </For>
    </svg>
  );
}
