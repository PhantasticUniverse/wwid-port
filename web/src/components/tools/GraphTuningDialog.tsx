import { Show, createSignal, onMount, onCleanup } from "solid-js";
import { sessionStore } from "../../stores/session";
import { Chart, LineController, LineElement, PointElement, LinearScale, Legend, Tooltip } from "chart.js";

Chart.register(LineController, LineElement, PointElement, LinearScale, Legend, Tooltip);

interface TuningCurve {
  note_name: string;
  target_freq: number;
  predicted_freq: number;
  freq_min?: number;
  freq_max?: number;
  points: [number, number][];
}

interface GraphTuningResult {
  curves: TuningCurve[];
}

// Distinct colors for up to 20 series
const COLORS = [
  "#3b82f6", "#ef4444", "#22c55e", "#f59e0b", "#a855f7",
  "#06b6d4", "#ec4899", "#84cc16", "#f97316", "#6366f1",
  "#14b8a6", "#e11d48", "#65a30d", "#d946ef", "#0ea5e9",
  "#fb923c", "#8b5cf6", "#10b981", "#f43f5e", "#facc15",
];

export default function GraphTuningDialog(props: { onClose: () => void }) {
  const [data, setData] = createSignal<GraphTuningResult | null>(null);
  const [loading, setLoading] = createSignal(true);
  let canvasRef: HTMLCanvasElement | undefined;
  let chartInstance: Chart | undefined;

  onMount(async () => {
    const result = await sessionStore.graphTuning();
    if (result) {
      setData(result as GraphTuningResult);
    }
    setLoading(false);
  });

  onCleanup(() => {
    chartInstance?.destroy();
  });

  function initChart(canvas: HTMLCanvasElement) {
    canvasRef = canvas;
    const d = data();
    if (!d || !canvasRef) return;

    const datasets = d.curves.map((curve, i) => ({
      label: curve.note_name,
      data: curve.points.map(([freq, ratio]) => ({ x: freq, y: ratio })),
      borderColor: COLORS[i % COLORS.length],
      backgroundColor: "transparent",
      borderWidth: 1.5,
      pointRadius: 0,
      tension: 0.1,
    }));

    chartInstance = new Chart(canvasRef, {
      type: "line",
      data: { datasets },
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
            title: { display: true, text: "Im(Z) / Re(Z)", color: "#8b8fa3" },
            ticks: { color: "#8b8fa3" },
            grid: { color: "#1a1d27" },
          },
        },
        plugins: {
          legend: {
            position: "right",
            labels: { color: "#e4e6ef", boxWidth: 12, font: { size: 11 } },
          },
          tooltip: {
            callbacks: {
              label: (ctx) => `${ctx.dataset.label}: ${(ctx.parsed.y).toFixed(3)} @ ${(ctx.parsed.x).toFixed(1)} Hz`,
            },
          },
        },
      },
    });
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
        <h2 class="text-lg font-semibold mb-3">Graph Tuning — Playing Ranges</h2>

        <Show when={loading()}>
          <div class="text-sm py-8 text-center" style={{ color: "var(--color-text-muted)" }}>
            Computing playing ranges...
          </div>
        </Show>

        <Show when={!loading() && !data()}>
          <div class="text-sm py-4" style={{ color: "var(--color-text-muted)" }}>
            Failed to compute graph tuning data.
          </div>
        </Show>

        <Show when={!loading() && data()}>
          <div style={{ height: "450px" }}>
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
