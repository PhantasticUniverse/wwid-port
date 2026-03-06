/** Instrument sketch in a popup window (matches Java WIDesigner behavior). */

interface SketchBorePoint {
  position: number;
  diameter: number;
}

interface SketchHole {
  name: string | null;
  position: number;
  diameter: number;
  height: number;
}

interface SketchMouthpieceFipple {
  type: "Fipple";
  position: number;
  window_length: number;
  window_width: number;
  fipple_factor: number | null;
  window_height: number | null;
  windway_height: number | null;
  windway_length: number | null;
}

interface SketchMouthpieceEmbouchure {
  type: "Embouchure";
  position: number;
  length: number;
  width: number;
  height: number;
  airstream_length: number;
  airstream_height: number;
}

interface SketchMouthpieceReed {
  type: "SingleReed" | "DoubleReed" | "LipReed";
  position: number;
  [key: string]: unknown;
}

type SketchMouthpiece = SketchMouthpieceFipple | SketchMouthpieceEmbouchure | SketchMouthpieceReed;

export interface SketchData {
  name: string;
  length_type: string;
  bore_length: number;
  bore_points: SketchBorePoint[];
  holes: SketchHole[];
  mouthpiece: SketchMouthpiece;
  flange_diameter: number;
}

export function openSketchPopup(data: SketchData) {
  const popup = window.open(
    "",
    `sketch-${Date.now()}`,
    "width=900,height=500,menubar=no,toolbar=no,location=no,status=no"
  );
  if (!popup) {
    alert("Popup blocked — please allow popups for the instrument sketch.");
    return;
  }

  const doc = popup.document;
  doc.title = `Sketch — ${data.name}`;

  const style = doc.createElement("style");
  style.textContent = `
    body {
      margin: 0; padding: 16px;
      background: #0f1117; color: #e4e6ef;
      font-family: "Inter", system-ui, -apple-system, sans-serif;
      font-size: 13px;
    }
    h2 { margin: 0 0 12px; font-size: 15px; font-weight: 600; }
    .summary { display: grid; grid-template-columns: auto 1fr; gap: 2px 24px; margin-top: 12px; }
    .summary .label { color: #8b8fa3; }
    .summary .value { font-family: monospace; }
  `;
  doc.head.appendChild(style);

  const h2 = doc.createElement("h2");
  h2.textContent = data.name;
  doc.body.appendChild(h2);

  // Build SVG
  const svgEl = buildSketchSVG(doc, data);
  doc.body.appendChild(svgEl);

  // Summary table
  const summary = doc.createElement("div");
  summary.className = "summary";
  const rows: [string, string][] = [
    ["Bore Length", `${data.bore_length.toFixed(2)}`],
    ["Holes", `${data.holes.length}`],
    ["Mouthpiece", data.mouthpiece.type],
    ["Flange Diameter", `${data.flange_diameter.toFixed(2)}`],
  ];
  for (const [label, value] of rows) {
    const lbl = doc.createElement("div");
    lbl.className = "label";
    lbl.textContent = label;
    summary.appendChild(lbl);
    const val = doc.createElement("div");
    val.className = "value";
    val.textContent = value;
    summary.appendChild(val);
  }
  doc.body.appendChild(summary);
}

function buildSketchSVG(doc: Document, d: SketchData): SVGElement {
  const PADDING = 40;
  const TICK_LEN = 5;
  const SVG_WIDTH = 860;
  const SVG_HEIGHT = 340;

  const maxDia = Math.max(
    d.flange_diameter,
    ...d.bore_points.map((p) => p.diameter),
    ...d.holes.map((h) => h.diameter),
  );
  const drawW = SVG_WIDTH - 2 * PADDING;
  const drawH = SVG_HEIGHT - 2 * PADDING;
  const scaleX = drawW / d.bore_length;
  const scaleY = drawH / (maxDia * 1.5);
  const scale = Math.min(scaleX, scaleY);

  const centerY = SVG_HEIGHT / 2;
  const toX = (pos: number) => PADDING + pos * scale;
  const toYTop = (dia: number) => centerY - (dia / 2) * scale;
  const toYBot = (dia: number) => centerY + (dia / 2) * scale;

  const ns = "http://www.w3.org/2000/svg";
  const svg = doc.createElementNS(ns, "svg");
  svg.setAttribute("width", String(SVG_WIDTH));
  svg.setAttribute("height", String(SVG_HEIGHT));
  svg.setAttribute("viewBox", `0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`);
  svg.style.background = "#1a1d27";
  svg.style.borderRadius = "6px";

  // Helper to create SVG elements
  function el(tag: string, attrs: Record<string, string>): SVGElement {
    const e = doc.createElementNS(ns, tag);
    for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
    return e;
  }

  function text(x: number, y: number, content: string, attrs: Record<string, string> = {}): SVGElement {
    const t = el("text", { x: String(x), y: String(y), fill: "#9ca3af", "font-size": "9", ...attrs });
    t.textContent = content;
    return t;
  }

  // X-axis
  svg.appendChild(el("line", {
    x1: String(PADDING), y1: String(SVG_HEIGHT - PADDING),
    x2: String(PADDING + d.bore_length * scale), y2: String(SVG_HEIGHT - PADDING),
    stroke: "#6b7280", "stroke-width": "1",
  }));

  const xTicks = generateTicks(0, d.bore_length, 5);
  for (const v of xTicks) {
    svg.appendChild(el("line", {
      x1: String(toX(v)), y1: String(SVG_HEIGHT - PADDING),
      x2: String(toX(v)), y2: String(SVG_HEIGHT - PADDING + TICK_LEN),
      stroke: "#6b7280", "stroke-width": "1",
    }));
    svg.appendChild(text(toX(v), SVG_HEIGHT - PADDING + TICK_LEN + 10, formatTick(v), { "text-anchor": "middle" }));
  }
  svg.appendChild(text(
    PADDING + (d.bore_length * scale) / 2, SVG_HEIGHT - 4, "Length",
    { "text-anchor": "middle", "font-size": "10" }
  ));

  // Y-axis
  svg.appendChild(el("line", {
    x1: String(PADDING), y1: String(toYTop(maxDia * 1.3)),
    x2: String(PADDING), y2: String(toYBot(maxDia * 1.3)),
    stroke: "#6b7280", "stroke-width": "1",
  }));

  const halfMaxDia = maxDia * 0.75;
  const yTicks = generateTicks(-halfMaxDia, halfMaxDia, 5);
  for (const v of yTicks) {
    const yPos = centerY - v * scale;
    svg.appendChild(el("line", {
      x1: String(PADDING - TICK_LEN), y1: String(yPos),
      x2: String(PADDING), y2: String(yPos),
      stroke: "#6b7280", "stroke-width": "1",
    }));
    svg.appendChild(text(PADDING - TICK_LEN - 2, yPos + 3, formatTick(v), { "text-anchor": "end" }));
  }
  const widthLabel = text(10, centerY, "Width", { "text-anchor": "middle", "font-size": "10" });
  widthLabel.setAttribute("transform", `rotate(-90, 10, ${centerY})`);
  svg.appendChild(widthLabel);

  // Center axis (dashed)
  svg.appendChild(el("line", {
    x1: String(PADDING), y1: String(centerY),
    x2: String(PADDING + d.bore_length * scale), y2: String(centerY),
    stroke: "#4b5563", "stroke-width": "0.5", "stroke-dasharray": "4,3",
  }));

  // Bore profile — dashed outline, no fill
  const topEdge = d.bore_points.map((p) => `${toX(p.position)},${toYTop(p.diameter)}`).join(" ");
  const botEdge = [...d.bore_points].reverse().map((p) => `${toX(p.position)},${toYBot(p.diameter)}`).join(" ");
  svg.appendChild(el("polygon", {
    points: `${topEdge} ${botEdge}`,
    fill: "none", stroke: "#9ca3af", "stroke-width": "1.5", "stroke-dasharray": "6,3",
  }));

  // Flange
  if (d.flange_diameter > 0) {
    svg.appendChild(el("line", {
      x1: String(toX(d.bore_length)), y1: String(toYTop(d.flange_diameter)),
      x2: String(toX(d.bore_length)), y2: String(toYBot(d.flange_diameter)),
      stroke: "#9ca3af", "stroke-width": "2",
    }));
  }

  // Tone holes — circles on center line
  for (let i = 0; i < d.holes.length; i++) {
    const hole = d.holes[i];
    const hx = toX(hole.position);
    const boreDia = interpolateBore(d.bore_points, hole.position);
    const boreTopY = centerY - (boreDia / 2) * scale;
    const holeRadius = (hole.diameter / 2) * scale;

    svg.appendChild(el("circle", {
      cx: String(hx), cy: String(centerY),
      r: String(Math.max(holeRadius, 3)),
      fill: "none", stroke: "#d1d5db", "stroke-width": "1.5",
    }));
    svg.appendChild(text(hx, boreTopY - 6, hole.name ?? `H${i + 1}`, { "text-anchor": "middle" }));
  }

  // Mouthpiece
  const mp = d.mouthpiece;
  if (mp.type === "Fipple") {
    const fipple = mp as SketchMouthpieceFipple;
    // Fipple window: solid rectangle from (position - window_length) to (position)
    const winRight = toX(fipple.position);
    const winLeft = toX(fipple.position - fipple.window_length);
    const winHalfH = (fipple.window_width / 2) * scale;
    const winTop = centerY - winHalfH;
    const winW = winRight - winLeft;
    const winH = winHalfH * 2;
    if (winW > 0 && winH > 0) {
      svg.appendChild(el("rect", {
        x: String(winLeft), y: String(winTop),
        width: String(winW), height: String(winH),
        fill: "none", stroke: "#d1d5db", "stroke-width": "1.5",
      }));
    }

    // Windway: dashed rectangle extending left from window (if windway_length exists)
    if (fipple.windway_length != null && fipple.windway_length > 0) {
      const wwRight = winLeft;
      const wwLeft = toX(fipple.position - fipple.window_length - fipple.windway_length);
      const wwW = wwRight - wwLeft;
      if (wwW > 0) {
        svg.appendChild(el("rect", {
          x: String(wwLeft), y: String(winTop),
          width: String(wwW), height: String(winH),
          fill: "none", stroke: "#d1d5db", "stroke-width": "1",
          "stroke-dasharray": "5,2",
        }));
      }
    }
  } else if (mp.type === "Embouchure") {
    const emb = mp as SketchMouthpieceEmbouchure;
    // Embouchure hole: ellipse centered on position
    const cx = toX(emb.position);
    const rx = (emb.length / 2) * scale;
    const ry = (emb.width / 2) * scale;
    if (rx > 0 && ry > 0) {
      svg.appendChild(el("ellipse", {
        cx: String(cx), cy: String(centerY),
        rx: String(Math.max(rx, 3)), ry: String(Math.max(ry, 3)),
        fill: "none", stroke: "#d1d5db", "stroke-width": "1.5",
      }));
    }
  }
  // Reed/LipReed: no mouthpiece drawing (matches Java)

  return svg;
}

/** Linearly interpolate bore diameter at a given position. */
function interpolateBore(points: SketchBorePoint[], pos: number): number {
  if (points.length === 0) return 0;
  if (pos <= points[0].position) return points[0].diameter;
  if (pos >= points[points.length - 1].position) return points[points.length - 1].diameter;
  for (let i = 0; i < points.length - 1; i++) {
    const a = points[i];
    const b = points[i + 1];
    if (pos >= a.position && pos <= b.position) {
      const t = (pos - a.position) / (b.position - a.position);
      return a.diameter + t * (b.diameter - a.diameter);
    }
  }
  return points[points.length - 1].diameter;
}

/** Generate nice tick values for an axis range. */
function generateTicks(min: number, max: number, approxCount: number): number[] {
  const range = max - min;
  if (range <= 0) return [min];
  const rawStep = range / approxCount;
  const mag = Math.pow(10, Math.floor(Math.log10(rawStep)));
  const norm = rawStep / mag;
  const niceStep = norm < 1.5 ? mag : norm < 3 ? 2 * mag : norm < 7 ? 5 * mag : 10 * mag;
  const ticks: number[] = [];
  let v = Math.ceil(min / niceStep) * niceStep;
  while (v <= max + niceStep * 0.01) {
    ticks.push(v);
    v += niceStep;
  }
  return ticks;
}

/** Format tick value: drop trailing zeros. */
function formatTick(v: number): string {
  if (v === 0 || Object.is(v, -0)) return "0";
  if (Math.abs(v) < 1e-10) return "0";
  if (Math.abs(v) >= 1) return v.toFixed(1).replace(/\.0$/, "");
  return v.toFixed(3).replace(/0+$/, "").replace(/\.$/, "");
}
