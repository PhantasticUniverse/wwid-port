import { Show, createSignal, onMount } from "solid-js";
import { sessionStore } from "../../stores/session";

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

interface SketchMouthpiece {
  type: string;
  position: number;
  [key: string]: unknown;
}

interface SketchData {
  name: string;
  length_type: string;
  bore_length: number;
  bore_points: SketchBorePoint[];
  holes: SketchHole[];
  mouthpiece: SketchMouthpiece;
  flange_diameter: number;
}

export default function SketchDialog(props: { onClose: () => void }) {
  const [data, setData] = createSignal<SketchData | null>(null);
  const [loading, setLoading] = createSignal(true);

  onMount(async () => {
    const result = await sessionStore.sketchInstrument();
    if (result) setData(result as SketchData);
    setLoading(false);
  });

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
          width: "640px",
          "max-height": "90vh",
          "overflow-y": "auto",
        }}
      >
        <Show when={!loading()} fallback={<div class="text-sm" style={{ color: "var(--color-text-muted)" }}>Loading sketch...</div>}>
          <Show when={data()} fallback={<div class="text-sm" style={{ color: "var(--color-text-muted)" }}>Failed to load sketch data.</div>}>
            {(d) => <SketchContent data={d()} />}
          </Show>
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

function SketchContent(props: { data: SketchData }) {
  const d = props.data;
  const PADDING = 40;
  const TICK_LEN = 5;
  const SVG_WIDTH = 580;
  const SVG_HEIGHT = 340;

  // Units abbreviation
  const unit = d.length_type === "Millimetres" ? "mm" : d.length_type === "Inches" ? "in" : d.length_type;

  // Compute scale: map bore length to SVG horizontal, max diameter to SVG vertical
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

  // Build bore polygon points (top edge left->right, bottom edge right->left)
  const topEdge = d.bore_points.map((p) => `${toX(p.position)},${toYTop(p.diameter)}`).join(" ");
  const botEdge = [...d.bore_points]
    .reverse()
    .map((p) => `${toX(p.position)},${toYBot(p.diameter)}`)
    .join(" ");
  const borePolygon = `${topEdge} ${botEdge}`;

  // Mouthpiece label
  const mpLabel = d.mouthpiece.type;

  // Axis tick generation
  const xTicks = generateTicks(0, d.bore_length, 5);
  const halfMaxDia = maxDia * 0.75;
  const yTicks = generateTicks(-halfMaxDia, halfMaxDia, 4);

  return (
    <div>
      <h2 class="text-lg font-semibold mb-3">{d.name}</h2>
      <svg
        width={SVG_WIDTH}
        height={SVG_HEIGHT}
        viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
        style={{ background: "var(--color-surface-alt)", "border-radius": "6px" }}
      >
        {/* X-axis (Length) */}
        <line
          x1={PADDING}
          y1={SVG_HEIGHT - PADDING}
          x2={PADDING + d.bore_length * scale}
          y2={SVG_HEIGHT - PADDING}
          stroke="#6b7280"
          stroke-width="1"
        />
        {xTicks.map((v) => (
          <g>
            <line
              x1={toX(v)}
              y1={SVG_HEIGHT - PADDING}
              x2={toX(v)}
              y2={SVG_HEIGHT - PADDING + TICK_LEN}
              stroke="#6b7280"
              stroke-width="1"
            />
            <text
              x={toX(v)}
              y={SVG_HEIGHT - PADDING + TICK_LEN + 10}
              text-anchor="middle"
              font-size="9"
              fill="#9ca3af"
            >
              {formatTick(v)}
            </text>
          </g>
        ))}
        <text
          x={PADDING + (d.bore_length * scale) / 2}
          y={SVG_HEIGHT - 4}
          text-anchor="middle"
          font-size="10"
          fill="#9ca3af"
        >
          Length ({unit})
        </text>

        {/* Y-axis (Width) */}
        <line
          x1={PADDING}
          y1={toYTop(maxDia * 1.3)}
          x2={PADDING}
          y2={toYBot(maxDia * 1.3)}
          stroke="#6b7280"
          stroke-width="1"
        />
        {yTicks.map((v) => {
          const yPos = centerY - v * scale;
          return (
            <g>
              <line
                x1={PADDING - TICK_LEN}
                y1={yPos}
                x2={PADDING}
                y2={yPos}
                stroke="#6b7280"
                stroke-width="1"
              />
              <text
                x={PADDING - TICK_LEN - 2}
                y={yPos + 3}
                text-anchor="end"
                font-size="9"
                fill="#9ca3af"
              >
                {formatTick(Math.abs(v))}
              </text>
            </g>
          );
        })}
        <text
          x={10}
          y={centerY}
          text-anchor="middle"
          font-size="10"
          fill="#9ca3af"
          transform={`rotate(-90, 10, ${centerY})`}
        >
          Width ({unit})
        </text>

        {/* Center axis (dashed) */}
        <line
          x1={PADDING}
          y1={centerY}
          x2={PADDING + d.bore_length * scale}
          y2={centerY}
          stroke="#4b5563"
          stroke-width="0.5"
          stroke-dasharray="4,3"
        />

        {/* Bore profile — dashed outline, no fill (engineering style) */}
        <polygon
          points={borePolygon}
          fill="none"
          stroke="#9ca3af"
          stroke-width="1.5"
          stroke-dasharray="6,3"
        />

        {/* Flange at the end */}
        {d.flange_diameter > 0 && (
          <line
            x1={toX(d.bore_length)}
            y1={toYTop(d.flange_diameter)}
            x2={toX(d.bore_length)}
            y2={toYBot(d.flange_diameter)}
            stroke="#9ca3af"
            stroke-width="2"
          />
        )}

        {/* Tone holes — circles (top-view, diameter proportional) */}
        {d.holes.map((hole, i) => {
          const hx = toX(hole.position);
          const boreDia = interpolateBore(d.bore_points, hole.position);
          const boreTopY = centerY - (boreDia / 2) * scale;
          const holeRadius = (hole.diameter / 2) * scale;
          // Position circle on the center line (straddling the bore), matching Java
          const cy = centerY;

          return (
            <g>
              {/* Hole circle */}
              <circle
                cx={hx}
                cy={cy}
                r={Math.max(holeRadius, 3)}
                fill="none"
                stroke="#d1d5db"
                stroke-width="1.5"
              />
              {/* Hole label — above bore top edge */}
              <text
                x={hx}
                y={boreTopY - 6}
                text-anchor="middle"
                font-size="9"
                fill="#9ca3af"
              >
                {hole.name ?? `H${i + 1}`}
              </text>
            </g>
          );
        })}

        {/* Mouthpiece indicator — small rectangle */}
        {(() => {
          const mpX = toX(d.mouthpiece.position);
          const mpDia = interpolateBore(d.bore_points, d.mouthpiece.position);
          const mpY = toYBot(mpDia) + 4;
          return (
            <g>
              <rect
                x={mpX - 4}
                y={mpY}
                width={8}
                height={6}
                fill="none"
                stroke="#9ca3af"
                stroke-width="1"
              />
              <text
                x={mpX}
                y={mpY + 18}
                text-anchor="middle"
                font-size="9"
                fill="#9ca3af"
              >
                {mpLabel}
              </text>
            </g>
          );
        })()}
      </svg>

      {/* Summary table */}
      <div class="mt-3 grid grid-cols-2 gap-x-6 gap-y-1 text-sm" style={{ color: "var(--color-text)" }}>
        <div>Bore Length:</div>
        <div style={{ "font-family": "monospace" }}>{d.bore_length.toFixed(2)} {unit}</div>
        <div>Holes:</div>
        <div style={{ "font-family": "monospace" }}>{d.holes.length}</div>
        <div>Mouthpiece:</div>
        <div>{mpLabel}</div>
        <div>Flange Diameter:</div>
        <div style={{ "font-family": "monospace" }}>{d.flange_diameter.toFixed(2)} {unit}</div>
      </div>
    </div>
  );
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
  if (v === 0) return "0";
  if (Math.abs(v) >= 1) return v.toFixed(1).replace(/\.0$/, "");
  return v.toFixed(3).replace(/0+$/, "").replace(/\.$/, "");
}
