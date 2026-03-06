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

/** Find the Y value in a curve nearest to a given X frequency. */
function nearestY(points: [number, number][], targetX: number): number | null {
  if (points.length === 0) return null;
  let best = points[0];
  let bestDist = Math.abs(points[0][0] - targetX);
  for (const p of points) {
    const dist = Math.abs(p[0] - targetX);
    if (dist < bestDist) {
      bestDist = dist;
      best = p;
    }
  }
  return best[1];
}

export default function GraphTuningDialog(props: { onClose: () => void }) {
  const [data, setData] = createSignal<GraphTuningResult | null>(null);
  const [loading, setLoading] = createSignal(true);
  let canvasRef: HTMLCanvasElement | undefined;
  let chartInstance: Chart | undefined;

  onMount(async () => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") props.onClose(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
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

    // Line datasets: one per curve, all muted gray, hidden from legend
    const lineDatasets = d.curves.map((curve) => ({
      label: "",
      data: curve.points.map(([freq, ratio]) => ({ x: freq, y: ratio })),
      borderColor: "#6b7280",
      backgroundColor: "transparent",
      borderWidth: 1,
      pointRadius: 0,
      tension: 0.1,
    }));

    // Collect marker points from each curve
    const fmaxPts: { x: number; y: number }[] = [];
    const fminPts: { x: number; y: number }[] = [];
    const targetIn: { x: number; y: number }[] = [];
    const targetOut: { x: number; y: number }[] = [];

    for (const curve of d.curves) {
      if (curve.freq_max != null && curve.freq_max > 0) {
        const y = nearestY(curve.points, curve.freq_max);
        if (y != null) fmaxPts.push({ x: curve.freq_max, y });
      }
      if (curve.freq_min != null && curve.freq_min > 0) {
        const y = nearestY(curve.points, curve.freq_min);
        if (y != null) fminPts.push({ x: curve.freq_min, y });
      }
      if (curve.target_freq > 0) {
        const y = nearestY(curve.points, curve.target_freq);
        if (y != null) {
          const inRange =
            curve.freq_min != null &&
            curve.freq_max != null &&
            curve.target_freq >= curve.freq_min &&
            curve.target_freq <= curve.freq_max;
          (inRange ? targetIn : targetOut).push({ x: curve.target_freq, y });
        }
      }
    }

    // Scatter overlay datasets (markers only, no connecting lines)
    const markerDatasets = [
      fmaxPts.length > 0 && {
        label: "Peak (fmax)",
        data: fmaxPts,
        borderColor: "#22c55e",
        backgroundColor: "#22c55e",
        pointRadius: 5,
        pointStyle: "circle" as const,
        showLine: false,
      },
      fminPts.length > 0 && {
        label: "Zero (fmin)",
        data: fminPts,
        borderColor: "#3b82f6",
        backgroundColor: "transparent",
        pointRadius: 5,
        pointStyle: "circle" as const,
        borderWidth: 2,
        showLine: false,
      },
      targetIn.length > 0 && {
        label: "Target (in range)",
        data: targetIn,
        borderColor: "#22c55e",
        backgroundColor: "#22c55e",
        pointRadius: 6,
        pointStyle: "rectRot" as const,
        showLine: false,
      },
      targetOut.length > 0 && {
        label: "Target (out of range)",
        data: targetOut,
        borderColor: "#ef4444",
        backgroundColor: "#ef4444",
        pointRadius: 6,
        pointStyle: "rectRot" as const,
        showLine: false,
      },
    ].filter(Boolean) as any[];

    chartInstance = new Chart(canvasRef, {
      type: "line",
      data: { datasets: [...lineDatasets, ...markerDatasets] },
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
            title: { display: true, text: "Reactance Ratio, X/R", color: "#8b8fa3" },
            ticks: { color: "#8b8fa3" },
            grid: { color: "#1a1d27" },
          },
        },
        plugins: {
          legend: {
            position: "top",
            labels: {
              color: "#e4e6ef",
              boxWidth: 12,
              font: { size: 11 },
              usePointStyle: true,
              filter: (item) => item.text !== "",
            },
          },
          tooltip: {
            callbacks: {
              label: (ctx) =>
                `${ctx.dataset.label || "curve"}: ${ctx.parsed.y.toFixed(3)} @ ${ctx.parsed.x.toFixed(1)} Hz`,
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
        <h2 class="text-lg font-semibold mb-3">Impedance Pattern</h2>

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
