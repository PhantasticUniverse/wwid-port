/** Note Spectrum in a popup window (matches Java WIDesigner behavior). */

import { Chart, LineController, LineElement, PointElement, LinearScale, Legend, Tooltip } from "chart.js";
import type { ChartConfiguration } from "chart.js";
import { sessionStore } from "../../stores/session";
import { getSpectrumMult } from "../layout/SettingsDialog";

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

interface EvalNote {
  note: string;
  target_freq: number;
}

export function openNoteSpectrumPopup(notes: EvalNote[]): boolean {
  const popup = window.open(
    "",
    `spectrum-${Date.now()}`,
    "width=900,height=560,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    return false;
  }

  const doc = popup.document;
  doc.title = "Note Spectrum";

  const style = doc.createElement("style");
  style.textContent = `
    body {
      margin: 0; padding: 16px;
      background: #0f1117; color: #e4e6ef;
      font-family: "Inter", system-ui, -apple-system, sans-serif;
      font-size: 13px;
    }
    h2 { margin: 0 0 12px; font-size: 15px; font-weight: 600; }
    .controls { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
    .controls label { color: #8b8fa3; font-size: 13px; }
    .controls select {
      padding: 4px 8px; border-radius: 4px; font-size: 13px;
      background: #1a1d27; color: #e4e6ef; border: 1px solid #2e3345;
    }
    .controls .info { color: #8b8fa3; font-size: 12px; }
    .status { color: #8b8fa3; font-size: 13px; padding: 40px 0; text-align: center; }
  `;
  doc.head.appendChild(style);

  const h2 = doc.createElement("h2");
  h2.textContent = "Note Spectrum";
  doc.body.appendChild(h2);

  // Controls row
  const controls = doc.createElement("div");
  controls.className = "controls";

  const label = doc.createElement("label");
  label.textContent = "Fingering:";
  controls.appendChild(label);

  const select = doc.createElement("select");
  for (let i = 0; i < notes.length; i++) {
    const opt = doc.createElement("option");
    opt.value = String(i);
    opt.textContent = `${notes[i].note} (${notes[i].target_freq.toFixed(1)} Hz)`;
    select.appendChild(opt);
  }
  controls.appendChild(select);

  const info = doc.createElement("span");
  info.className = "info";
  controls.appendChild(info);

  doc.body.appendChild(controls);

  // Chart container
  const container = doc.createElement("div");
  container.style.height = "420px";
  container.style.position = "relative";
  doc.body.appendChild(container);

  const canvas = doc.createElement("canvas");
  container.appendChild(canvas);

  // Status element (shown while loading)
  const status = doc.createElement("div");
  status.className = "status";
  status.textContent = "Computing spectrum...";
  container.appendChild(status);

  let chartInstance: Chart | undefined;

  async function loadSpectrum(idx: number) {
    status.style.display = "block";
    canvas.style.display = "none";
    chartInstance?.destroy();
    chartInstance = undefined;

    const result = await sessionStore.noteSpectrum(idx, getSpectrumMult()) as NoteSpectrumResult | null;
    if (!result) {
      status.textContent = "Failed to compute spectrum.";
      return;
    }

    if (popup!.closed) return;
    info.textContent = `${result.note_name} — target ${result.target_freq.toFixed(1)} Hz`;
    status.style.display = "none";
    canvas.style.display = "block";

    popup!.requestAnimationFrame(() => {
      chartInstance = buildChart(canvas, result);
    });
  }

  select.addEventListener("change", () => {
    loadSpectrum(parseInt(select.value));
  });

  // Load first note
  if (notes.length > 0) loadSpectrum(0);
  return true;
}

function buildChart(canvas: HTMLCanvasElement, d: NoteSpectrumResult): Chart {
  // Split gain data into playable (>=1) and damped (<1) segments
  const gainPlayable = d.points.map((p) => ({
    x: p.freq,
    y: p.loop_gain >= 1.0 ? p.loop_gain : NaN,
  }));
  const gainDamped = d.points.map((p) => ({
    x: p.freq,
    y: p.loop_gain < 1.0 ? p.loop_gain : NaN,
  }));

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
          title: { display: true, text: "Frequency", color: "#8b8fa3" },
          ticks: { color: "#8b8fa3" },
          grid: { color: "#1a1d27" },
        },
        y: {
          type: "linear",
          position: "left",
          title: { display: true, text: "Reactance Ratio, X/R", color: "#9ca3af" },
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
            label: (ctx) => `${ctx.dataset.label}: ${(ctx.parsed.y ?? 0).toFixed(4)} @ ${(ctx.parsed.x ?? 0).toFixed(1)} Hz`,
          },
        },
      },
    },
  };

  return new Chart(canvas, config);
}
