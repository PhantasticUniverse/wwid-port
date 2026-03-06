import { Show, For, createSignal, onMount, onCleanup } from "solid-js";
import { sessionStore } from "../../stores/session";
import { getSpectrumMult } from "../layout/SettingsDialog";
import { Chart, LineController, LineElement, PointElement, LinearScale, Legend, Tooltip } from "chart.js";
import type { ChartConfiguration } from "chart.js";

Chart.register(LineController, LineElement, PointElement, LinearScale, Legend, Tooltip);

interface SpectrumPoint {
  freq: number;
  impedance_ratio: number;
  loop_gain: number;
}

interface NoteSpectrumResult {
  note_name: string;
  target_freq: number;
  points: SpectrumPoint[];
}

interface EvalRow {
  note: string;
  target_freq: number;
}

export default function NoteSpectrumDialog(props: {
  onClose: () => void;
  /** Pre-evaluated fingering notes (for the dropdown) */
  notes: EvalRow[];
}) {
  const [selectedIdx, setSelectedIdx] = createSignal(0);
  const [data, setData] = createSignal<NoteSpectrumResult | null>(null);
  const [loading, setLoading] = createSignal(false);
  let canvasRef: HTMLCanvasElement | undefined;
  let chartInstance: Chart | undefined;

  async function loadSpectrum(idx: number) {
    setLoading(true);
    chartInstance?.destroy();
    chartInstance = undefined;
    const result = await sessionStore.noteSpectrum(idx, getSpectrumMult());
    if (result) {
      setData(result as NoteSpectrumResult);
    }
    setLoading(false);
  }

  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") props.onClose(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
    if (props.notes.length > 0) loadSpectrum(0);
  });

  onCleanup(() => {
    chartInstance?.destroy();
  });

  function initChart(canvas: HTMLCanvasElement) {
    canvasRef = canvas;
    const d = data();
    if (!d || !canvasRef) return;
    chartInstance?.destroy();

    // Split gain data into playable (>=1) and damped (<1) segments
    // We create two datasets with NaN gaps so each only draws its portion
    const gainPlayable = d.points.map((p) => ({
      x: p.freq,
      y: p.loop_gain >= 1.0 ? p.loop_gain : NaN,
    }));
    const gainDamped = d.points.map((p) => ({
      x: p.freq,
      y: p.loop_gain < 1.0 ? p.loop_gain : NaN,
    }));

    // Find x range for the gain=1 reference line
    const freqMin = d.points.length > 0 ? d.points[0].freq : 0;
    const freqMax = d.points.length > 0 ? d.points[d.points.length - 1].freq : 1000;

    const config: ChartConfiguration = {
      type: "line",
      data: {
        datasets: [
          {
            label: "Im(Z)/Re(Z)",
            data: d.points.map((p) => ({ x: p.freq, y: p.impedance_ratio })),
            borderColor: "#9ca3af",
            backgroundColor: "transparent",
            borderWidth: 1.5,
            pointRadius: 0,
            yAxisID: "y",
          },
          {
            label: "Gain (playable)",
            data: gainPlayable,
            borderColor: "#22c55e",
            backgroundColor: "transparent",
            borderWidth: 2,
            pointRadius: 0,
            yAxisID: "y1",
            spanGaps: false,
          },
          {
            label: "Gain (damped)",
            data: gainDamped,
            borderColor: "#ef4444",
            backgroundColor: "transparent",
            borderWidth: 2,
            pointRadius: 0,
            yAxisID: "y1",
            spanGaps: false,
          },
          {
            label: "",
            data: [
              { x: freqMin, y: 1.0 },
              { x: freqMax, y: 1.0 },
            ],
            borderColor: "#6b728080",
            backgroundColor: "transparent",
            borderWidth: 1,
            borderDash: [6, 3],
            pointRadius: 0,
            yAxisID: "y1",
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        interaction: { mode: "nearest", intersect: false },
        scales: {
          x: {
            type: "linear",
            title: { display: true, text: "Frequency (Hz)", color: "#8b8fa3" },
            ticks: { color: "#8b8fa3" },
            grid: { color: "#1a1d27" },
          },
          y: {
            type: "linear",
            position: "left",
            title: { display: true, text: "Im(Z)/Re(Z)", color: "#9ca3af" },
            ticks: { color: "#9ca3af" },
            grid: { color: "#1a1d27" },
          },
          y1: {
            type: "linear",
            position: "right",
            title: { display: true, text: "Loop Gain", color: "#8b8fa3" },
            ticks: { color: "#8b8fa3" },
            grid: { drawOnChartArea: false },
          },
        },
        plugins: {
          legend: {
            position: "top",
            labels: {
              color: "#e4e6ef",
              boxWidth: 12,
              font: { size: 11 },
              filter: (item) => item.text !== "",
            },
          },
          tooltip: {
            callbacks: {
              label: (ctx) => `${ctx.dataset.label}: ${(ctx.parsed.y).toFixed(4)} @ ${(ctx.parsed.x).toFixed(1)} Hz`,
            },
          },
        },
      },
    };

    chartInstance = new Chart(canvasRef, config);
  }

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
          width: "860px",
          "max-height": "90vh",
        }}
      >
        <h2 class="text-lg font-semibold mb-3">Note Spectrum</h2>

        {/* Fingering selector */}
        <div class="flex items-center gap-3 mb-4">
          <label class="text-sm" style={{ color: "var(--color-text-muted)" }}>Fingering:</label>
          <select
            class="px-2 py-1 rounded text-sm border"
            style={{ background: "var(--color-surface-alt)", color: "var(--color-text)", "border-color": "var(--color-border)" }}
            value={selectedIdx()}
            onChange={(e) => {
              const idx = parseInt(e.currentTarget.value);
              setSelectedIdx(idx);
              loadSpectrum(idx);
            }}
          >
            <For each={props.notes}>
              {(note, i) => (
                <option value={i()}>
                  {note.note} ({note.target_freq.toFixed(1)} Hz)
                </option>
              )}
            </For>
          </select>
          <Show when={data()}>
            {(d) => (
              <span class="text-xs" style={{ color: "var(--color-text-muted)" }}>
                {d().note_name} — target {d().target_freq.toFixed(1)} Hz
              </span>
            )}
          </Show>
        </div>

        <Show when={loading()}>
          <div class="text-sm py-8 text-center" style={{ color: "var(--color-text-muted)" }}>
            Computing spectrum...
          </div>
        </Show>

        <Show when={!loading() && data()}>
          <div style={{ height: "420px" }}>
            <canvas ref={(el) => { canvasRef = el; requestAnimationFrame(() => initChart(el)); }} />
          </div>
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
