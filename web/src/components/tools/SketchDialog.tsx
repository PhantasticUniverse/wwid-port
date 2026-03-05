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
  const PADDING = 30;
  const SVG_WIDTH = 580;
  const SVG_HEIGHT = 320;

  // Compute scale: map bore length to SVG horizontal, max diameter to SVG vertical
  const maxDia = Math.max(
    d.flange_diameter,
    ...d.bore_points.map((p) => p.diameter),
    ...d.holes.map((h) => h.diameter),
  );
  const scaleX = (SVG_WIDTH - 2 * PADDING) / d.bore_length;
  const scaleY = (SVG_HEIGHT - 2 * PADDING) / (maxDia * 1.5);
  const scale = Math.min(scaleX, scaleY);

  const centerY = SVG_HEIGHT / 2;
  const toX = (pos: number) => PADDING + pos * scale;
  const toYTop = (dia: number) => centerY - (dia / 2) * scale;
  const toYBot = (dia: number) => centerY + (dia / 2) * scale;

  // Build bore polygon points (top edge left→right, bottom edge right→left)
  const topEdge = d.bore_points.map((p) => `${toX(p.position)},${toYTop(p.diameter)}`).join(" ");
  const botEdge = [...d.bore_points]
    .reverse()
    .map((p) => `${toX(p.position)},${toYBot(p.diameter)}`)
    .join(" ");
  const borePolygon = `${topEdge} ${botEdge}`;

  // Mouthpiece label
  const mpLabel = d.mouthpiece.type;

  // Units abbreviation
  const unit = d.length_type === "Millimetres" ? "mm" : d.length_type === "Inches" ? "in" : d.length_type;

  return (
    <div>
      <h2 class="text-lg font-semibold mb-3">{d.name}</h2>
      <svg
        width={SVG_WIDTH}
        height={SVG_HEIGHT}
        viewBox={`0 0 ${SVG_WIDTH} ${SVG_HEIGHT}`}
        style={{ background: "var(--color-surface-alt)", "border-radius": "6px" }}
      >
        {/* Center axis */}
        <line
          x1={PADDING}
          y1={centerY}
          x2={PADDING + d.bore_length * scale}
          y2={centerY}
          stroke="#4b5563"
          stroke-width="0.5"
          stroke-dasharray="4,3"
        />

        {/* Bore profile */}
        <polygon
          points={borePolygon}
          fill="#3b82f620"
          stroke="#3b82f6"
          stroke-width="1.5"
        />

        {/* Flange at the end */}
        {d.flange_diameter > 0 && (
          <line
            x1={toX(d.bore_length)}
            y1={toYTop(d.flange_diameter)}
            x2={toX(d.bore_length)}
            y2={toYBot(d.flange_diameter)}
            stroke="#60a5fa"
            stroke-width="2"
          />
        )}

        {/* Tone holes */}
        {d.holes.map((hole, i) => {
          // Find bore diameter at hole position (interpolate)
          const boreDia = interpolateBore(d.bore_points, hole.position);
          const hx = toX(hole.position);
          const holeHalfW = (hole.diameter / 2) * scale;
          const chimneyH = hole.height * scale;
          const boreTopY = centerY - (boreDia / 2) * scale;

          return (
            <g>
              {/* Hole chimney extending upward from bore wall */}
              <rect
                x={hx - holeHalfW}
                y={boreTopY - chimneyH}
                width={holeHalfW * 2}
                height={chimneyH}
                fill="#f59e0b30"
                stroke="#f59e0b"
                stroke-width="1"
              />
              {/* Hole label */}
              <text
                x={hx}
                y={boreTopY - chimneyH - 4}
                text-anchor="middle"
                font-size="9"
                fill="#f59e0b"
              >
                {hole.name ?? `H${i + 1}`}
              </text>
            </g>
          );
        })}

        {/* Mouthpiece indicator */}
        <text
          x={toX(d.mouthpiece.position)}
          y={SVG_HEIGHT - 8}
          text-anchor="middle"
          font-size="10"
          fill="#a78bfa"
        >
          {mpLabel}
        </text>
        <line
          x1={toX(d.mouthpiece.position)}
          y1={toYBot(interpolateBore(d.bore_points, d.mouthpiece.position))}
          x2={toX(d.mouthpiece.position)}
          y2={SVG_HEIGHT - 16}
          stroke="#a78bfa"
          stroke-width="1"
          stroke-dasharray="2,2"
        />
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
