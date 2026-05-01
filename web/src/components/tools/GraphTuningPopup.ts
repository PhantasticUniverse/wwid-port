/** Graph Tuning (Impedance Pattern) in a popup window (matches Java WIDesigner behavior). */

import { Chart, LineController, LineElement, PointElement, LinearScale, Legend, Tooltip } from "chart.js";

Chart.register(LineController, LineElement, PointElement, LinearScale, Legend, Tooltip);

interface TuningCurve {
  note_name: string;
  target_freq: number;
  predicted_freq: number;
  freq_min?: number;
  freq_max?: number;
  y_at_fmin?: number;
  y_at_fmax?: number;
  y_at_target?: number;
  points: [number, number][];
}

interface GraphTuningResult {
  curves: TuningCurve[];
}

export function openGraphTuningPopup(result: GraphTuningResult): boolean {
  const popup = window.open(
    "",
    `graph-${Date.now()}`,
    "width=900,height=560,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    return false;
  }

  const doc = popup.document;
  doc.title = "Impedance Pattern";

  const style = doc.createElement("style");
  style.textContent = `
    body {
      margin: 0; padding: 16px;
      background: #0f1117; color: #e4e6ef;
      font-family: "Inter", system-ui, -apple-system, sans-serif;
      font-size: 13px;
    }
    h2 { margin: 0 0 12px; font-size: 15px; font-weight: 600; }
  `;
  doc.head.appendChild(style);

  const h2 = doc.createElement("h2");
  h2.textContent = "Impedance Pattern";
  doc.body.appendChild(h2);

  const container = doc.createElement("div");
  container.style.height = "460px";
  container.style.position = "relative";
  doc.body.appendChild(container);

  const canvas = doc.createElement("canvas");
  container.appendChild(canvas);

  // Build chart after DOM is ready
  popup.requestAnimationFrame(() => {
    if (popup.closed) return;
    buildChart(canvas, result);
  });
  return true;
}

function buildChart(canvas: HTMLCanvasElement, d: GraphTuningResult) {
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
    if (curve.freq_max != null && curve.freq_max > 0 && curve.y_at_fmax != null) {
      fmaxPts.push({ x: curve.freq_max, y: curve.y_at_fmax });
    }
    if (curve.freq_min != null && curve.freq_min > 0 && curve.y_at_fmin != null) {
      fminPts.push({ x: curve.freq_min, y: curve.y_at_fmin });
    }
    if (curve.target_freq > 0 && curve.y_at_target != null) {
      const inRange =
        curve.freq_min != null &&
        curve.freq_max != null &&
        curve.target_freq >= curve.freq_min &&
        curve.target_freq <= curve.freq_max;
      (inRange ? targetIn : targetOut).push({ x: curve.target_freq, y: curve.y_at_target });
    }
  }

  // Match Java's PlotPlayingRanges.buildGraph() Y-axis logic:
  // 1. minY=0, maxY=0 (range always includes zero)
  // 2. Expand from Y values at fmin and fmax frequencies only
  // 3. Add 10% padding
  // 4. Clamp all marker Y values to [minY, maxY]
  let minY = 0.0;
  let maxY = 0.0;
  for (const p of fminPts) { minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y); }
  for (const p of fmaxPts) { minY = Math.min(minY, p.y); maxY = Math.max(maxY, p.y); }
  if (maxY > minY) {
    const range = maxY - minY;
    maxY += 0.10 * range;
    minY -= 0.10 * range;
  }
  if (maxY <= minY) { maxY = 1; minY = -1; }
  const clamp = (y: number) => Math.max(minY, Math.min(maxY, y));
  for (const p of fmaxPts) { p.y = clamp(p.y); }
  for (const p of fminPts) { p.y = clamp(p.y); }
  for (const p of targetIn) { p.y = clamp(p.y); }
  for (const p of targetOut) { p.y = clamp(p.y); }

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

  new Chart(canvas, {
    type: "line",
    data: { datasets: [...lineDatasets, ...markerDatasets] },
    options: {
      responsive: true,
      maintainAspectRatio: false,
      interaction: { mode: "nearest", intersect: false },
      scales: {
        x: {
          type: "linear",
          title: { display: true, text: "Frequency", color: "#8b8fa3" },
          ticks: { color: "#8b8fa3" },
          grid: { color: "#1a1d27" },
        },
        y: {
          type: "linear",
          title: { display: true, text: "Reactance Ratio, X/R", color: "#8b8fa3" },
          ticks: { color: "#8b8fa3" },
          grid: { color: "#1a1d27" },
          min: minY,
          max: maxY,
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
              `${ctx.dataset.label || "curve"}: ${(ctx.parsed.y ?? 0).toFixed(3)} @ ${(ctx.parsed.x ?? 0).toFixed(1)} Hz`,
          },
        },
      },
    },
  });
}
