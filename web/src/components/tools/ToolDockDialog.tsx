import { For, Match, Switch, createMemo } from "solid-js";
import type { TuningResult } from "../../types/session";
import type { CompareResult } from "./ComparePopup";
import type { SketchData } from "./SketchPopup";

export interface SupplementaryRow {
  note: string;
  freq: number;
  im_z_correction: number;
  air_speed?: number;
  air_flow_rate?: number;
  gain: number;
  q_factor: number;
}

export interface SupplementaryResult {
  rows: SupplementaryRow[];
}

export interface TuningCurve {
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

export interface GraphTuningResult {
  curves: TuningCurve[];
}

export interface SpectrumPoint {
  freq: number;
  impedance_ratio: number;
  loop_gain: number;
}

export interface NoteSpectrumResult {
  note_name: string;
  target_freq: number;
  points: SpectrumPoint[];
}

export type ToolDockContent =
  | { kind: "eval"; result: TuningResult; instrumentName: string }
  | { kind: "supplementary"; result: SupplementaryResult; instrumentName: string }
  | { kind: "compare"; result: CompareResult }
  | { kind: "graph"; result: GraphTuningResult }
  | { kind: "spectrum"; result: NoteSpectrumResult }
  | { kind: "sketch"; result: SketchData }
  | { kind: "json"; title: string; data: unknown };

export default function ToolDockDialog(props: {
  content: ToolDockContent;
  onClose: () => void;
}) {
  const title = () =>
    props.content.kind === "eval" ? `Evaluation — ${props.content.instrumentName}` :
    props.content.kind === "supplementary" ? `Supplementary — ${props.content.instrumentName}` :
    props.content.kind === "compare" ? `${props.content.result.old_name} vs ${props.content.result.new_name}` :
    props.content.kind === "graph" ? "Impedance Pattern" :
    props.content.kind === "spectrum" ? `Note Spectrum — ${props.content.result.note_name}` :
    props.content.kind === "sketch" ? `Sketch — ${props.content.result.name}` :
    props.content.title;

  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: "rgba(31, 28, 24, 0.32)" }}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <section class="ws-panel max-h-[86vh] w-full max-w-5xl overflow-hidden">
        <header class="flex items-center justify-between border-b p-4" style={{ "border-color": "var(--color-border)" }}>
          <div>
            <div class="ws-eyebrow">Tool output</div>
            <h2 class="ws-serif text-2xl font-semibold">{title()}</h2>
          </div>
          <button class="ws-btn ws-btn--ghost" onClick={props.onClose}>Close</button>
        </header>
        <div class="max-h-[70vh] overflow-auto p-4">
          <ToolContent content={props.content} />
        </div>
      </section>
    </div>
  );
}

function ToolContent(props: { content: ToolDockContent }) {
  return (
    <Switch>
      <Match when={props.content.kind === "eval"}>
        <EvalView result={(props.content as { kind: "eval"; result: TuningResult }).result} />
      </Match>
      <Match when={props.content.kind === "supplementary"}>
        <SupplementaryView result={(props.content as { kind: "supplementary"; result: SupplementaryResult }).result} />
      </Match>
      <Match when={props.content.kind === "compare"}>
        <CompareView result={(props.content as { kind: "compare"; result: CompareResult }).result} />
      </Match>
      <Match when={props.content.kind === "graph"}>
        <GraphView result={(props.content as { kind: "graph"; result: GraphTuningResult }).result} />
      </Match>
      <Match when={props.content.kind === "spectrum"}>
        <SpectrumView result={(props.content as { kind: "spectrum"; result: NoteSpectrumResult }).result} />
      </Match>
      <Match when={props.content.kind === "sketch"}>
        <SketchView result={(props.content as { kind: "sketch"; result: SketchData }).result} />
      </Match>
      <Match when={props.content.kind === "json"}>
        <JsonView data={(props.content as { kind: "json"; data: unknown }).data} />
      </Match>
    </Switch>
  );
}

function EvalView(props: { result: TuningResult }) {
  return (
    <table class="w-full border-collapse text-xs">
      <thead>
        <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
          <th class="py-2 pr-3 text-left">Note</th>
          <th class="py-2 px-3 text-right">Target Hz</th>
          <th class="py-2 px-3 text-right">Predicted Hz</th>
          <th class="py-2 px-3 text-right">Deviation</th>
          <th class="py-2 pl-3 text-right">Weight</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.result.rows}>
          {(row) => (
            <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
              <td class="py-1.5 pr-3 font-semibold">{row.note}</td>
              <td class="py-1.5 px-3 text-right ws-mono">{row.target_freq.toFixed(2)}</td>
              <td class="py-1.5 px-3 text-right ws-mono">{row.predicted_freq.toFixed(2)}</td>
              <td
                class="py-1.5 px-3 text-right ws-mono"
                style={{
                  color: Math.abs(row.cents) < 5
                    ? "var(--color-success)"
                    : Math.abs(row.cents) < 15
                      ? "var(--color-warning)"
                      : "var(--color-error)",
                }}
              >
                {row.cents >= 0 ? "+" : ""}{row.cents.toFixed(2)}
              </td>
              <td class="py-1.5 pl-3 text-right ws-mono">{row.weight}</td>
            </tr>
          )}
        </For>
      </tbody>
      <tfoot>
        <tr>
          <td class="py-3 font-semibold" colspan={3}>Net error</td>
          <td class="py-3 text-right ws-mono">{props.result.net_error.toFixed(2)} cents</td>
          <td />
        </tr>
        <tr>
          <td class="py-1 font-semibold" colspan={3}>Mean deviation</td>
          <td class="py-1 text-right ws-mono">{props.result.mean_deviation.toFixed(2)} cents</td>
          <td />
        </tr>
      </tfoot>
    </table>
  );
}

function SupplementaryView(props: { result: SupplementaryResult }) {
  const hasAirSpeed = () => props.result.rows.some((row) => row.air_speed != null);
  const hasAirFlow = () => props.result.rows.some((row) => row.air_flow_rate != null);
  return (
    <table class="w-full border-collapse text-xs">
      <thead>
        <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
          <th class="py-2 pr-3 text-left">Note</th>
          <th class="py-2 px-3 text-right">Freq Hz</th>
          <th class="py-2 px-3 text-right">Im(Z) Corr</th>
          {hasAirSpeed() && <th class="py-2 px-3 text-right">Air Speed</th>}
          {hasAirFlow() && <th class="py-2 px-3 text-right">Air Flow</th>}
          <th class="py-2 px-3 text-right">Gain</th>
          <th class="py-2 pl-3 text-right">Q</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.result.rows}>
          {(row) => (
            <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
              <td class="py-1.5 pr-3 font-semibold">{row.note}</td>
              <td class="py-1.5 px-3 text-right ws-mono">{row.freq.toFixed(2)}</td>
              <td class="py-1.5 px-3 text-right ws-mono">{row.im_z_correction.toFixed(4)}</td>
              {hasAirSpeed() && <td class="py-1.5 px-3 text-right ws-mono">{row.air_speed?.toFixed(2) ?? "—"}</td>}
              {hasAirFlow() && <td class="py-1.5 px-3 text-right ws-mono">{row.air_flow_rate?.toFixed(2) ?? "—"}</td>}
              <td class="py-1.5 px-3 text-right ws-mono" style={{ color: row.gain >= 1 ? "var(--color-success)" : "var(--color-error)" }}>{row.gain.toFixed(4)}</td>
              <td class="py-1.5 pl-3 text-right ws-mono">{row.q_factor.toFixed(1)}</td>
            </tr>
          )}
        </For>
      </tbody>
    </table>
  );
}

function CompareView(props: { result: CompareResult }) {
  let lastCategory = "";
  return (
    <table class="w-full border-collapse text-xs">
      <thead>
        <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
          <th class="py-2 pr-3 text-left">Category</th>
          <th class="py-2 px-3 text-left">Field</th>
          <th class="py-2 px-3 text-right">{props.result.old_name}</th>
          <th class="py-2 px-3 text-right">{props.result.new_name}</th>
          <th class="py-2 px-3 text-right">Diff</th>
          <th class="py-2 pl-3 text-right">%</th>
        </tr>
      </thead>
      <tbody>
        <For each={props.result.rows}>
          {(row) => {
            const showCategory = row.category !== lastCategory;
            lastCategory = row.category;
            return (
              <tr style={{ "border-bottom": "1px solid var(--color-border)" }}>
                <td class="py-1.5 pr-3" style={{ color: "var(--color-text-muted)" }}>{showCategory ? row.category : ""}</td>
                <td class="py-1.5 px-3">{row.field}</td>
                <td class="py-1.5 px-3 text-right ws-mono">{row.old_value != null ? formatNumber(row.old_value) : "—"}</td>
                <td class="py-1.5 px-3 text-right ws-mono">{row.new_value != null ? formatNumber(row.new_value) : "—"}</td>
                <td class="py-1.5 px-3 text-right ws-mono">{row.difference != null ? formatSigned(row.difference) : "—"}</td>
                <td class="py-1.5 pl-3 text-right ws-mono">{row.percent_change != null ? `${row.percent_change >= 0 ? "+" : ""}${row.percent_change.toFixed(2)}%` : "—"}</td>
              </tr>
            );
          }}
        </For>
      </tbody>
    </table>
  );
}

function GraphView(props: { result: GraphTuningResult }) {
  const plot = createMemo(() => {
    const points = props.result.curves.flatMap((curve) => curve.points.map(([x, y]) => ({ x, y })));
    const xs = points.map((p) => p.x);
    const ys = points.map((p) => p.y);
    return {
      minX: Math.min(...xs),
      maxX: Math.max(...xs),
      minY: Math.min(...ys, -1),
      maxY: Math.max(...ys, 1),
    };
  });
  const sx = (x: number) => 48 + ((x - plot().minX) / (plot().maxX - plot().minX || 1)) * 780;
  const sy = (y: number) => 300 - ((y - plot().minY) / (plot().maxY - plot().minY || 1)) * 250;
  return (
    <div>
      <svg viewBox="0 0 860 330" class="w-full rounded border" style={{ background: "var(--color-field)", "border-color": "var(--color-border)" }}>
        <line x1="48" y1="300" x2="830" y2="300" stroke="var(--color-border-strong)" />
        <line x1="48" y1="40" x2="48" y2="300" stroke="var(--color-border-strong)" />
        <For each={props.result.curves}>
          {(curve) => (
            <polyline
              points={curve.points.map(([x, y]) => `${sx(x)},${sy(y)}`).join(" ")}
              fill="none"
              stroke="var(--color-text-muted)"
              stroke-width="1"
              opacity="0.55"
            />
          )}
        </For>
        <For each={props.result.curves}>
          {(curve) => (
            <>
              <circle cx={sx(curve.predicted_freq)} cy={sy(0)} r="3" fill="var(--color-success)" />
              <rect x={sx(curve.target_freq) - 3} y={sy(curve.y_at_target ?? 0) - 3} width="6" height="6" transform={`rotate(45 ${sx(curve.target_freq)} ${sy(curve.y_at_target ?? 0)})`} fill="var(--color-accent)" />
            </>
          )}
        </For>
      </svg>
      <p class="mt-2 text-xs" style={{ color: "var(--color-text-muted)" }}>Curves use real Graph Tuning output; circles mark predicted resonance frequencies and diamonds mark target frequencies.</p>
    </div>
  );
}

function SpectrumView(props: { result: NoteSpectrumResult }) {
  const bounds = createMemo(() => {
    const xs = props.result.points.map((p) => p.freq);
    const ys = props.result.points.map((p) => p.impedance_ratio);
    return { minX: Math.min(...xs), maxX: Math.max(...xs), minY: Math.min(...ys), maxY: Math.max(...ys) };
  });
  const sx = (x: number) => 48 + ((x - bounds().minX) / (bounds().maxX - bounds().minX || 1)) * 780;
  const sy = (y: number) => 300 - ((y - bounds().minY) / (bounds().maxY - bounds().minY || 1)) * 250;
  const sampled = () => props.result.points.filter((_, i) => i % 8 === 0);
  return (
    <div>
      <svg viewBox="0 0 860 330" class="w-full rounded border" style={{ background: "var(--color-field)", "border-color": "var(--color-border)" }}>
        <line x1="48" y1="300" x2="830" y2="300" stroke="var(--color-border-strong)" />
        <line x1="48" y1="40" x2="48" y2="300" stroke="var(--color-border-strong)" />
        <polyline
          points={sampled().map((p) => `${sx(p.freq)},${sy(p.impedance_ratio)}`).join(" ")}
          fill="none"
          stroke="var(--color-accent)"
          stroke-width="1.5"
        />
        <For each={sampled().filter((p) => p.loop_gain >= 1)}>
          {(p) => <circle cx={sx(p.freq)} cy={sy(p.impedance_ratio)} r="2" fill="var(--color-success)" opacity="0.65" />}
        </For>
      </svg>
      <p class="mt-2 text-xs" style={{ color: "var(--color-text-muted)" }}>Green dots indicate sampled points with loop gain at or above 1.0.</p>
    </div>
  );
}

function SketchView(props: { result: SketchData }) {
  const width = 860;
  const height = 320;
  const pad = 42;
  const maxDia = Math.max(props.result.flange_diameter, ...props.result.bore_points.map((p) => p.diameter), ...props.result.holes.map((h) => h.diameter), 1);
  const scaleX = (width - pad * 2) / props.result.bore_length;
  const scaleY = (height - pad * 2) / (maxDia * 1.8);
  const scale = Math.min(scaleX, scaleY);
  const centerY = height / 2;
  const x = (pos: number) => pad + pos * scale;
  const yTop = (dia: number) => centerY - (dia / 2) * scale;
  const yBot = (dia: number) => centerY + (dia / 2) * scale;
  const top = props.result.bore_points.map((p) => `${x(p.position)},${yTop(p.diameter)}`).join(" ");
  const bot = [...props.result.bore_points].reverse().map((p) => `${x(p.position)},${yBot(p.diameter)}`).join(" ");
  return (
    <div>
      <svg viewBox={`0 0 ${width} ${height}`} class="w-full rounded border" style={{ background: "var(--color-field)", "border-color": "var(--color-border)" }}>
        <line x1={pad} y1={centerY} x2={x(props.result.bore_length)} y2={centerY} stroke="var(--color-border-strong)" stroke-dasharray="5 4" />
        <polygon points={`${top} ${bot}`} fill="none" stroke="var(--color-text-muted)" stroke-width="1.5" stroke-dasharray="6 3" />
        <For each={props.result.holes}>
          {(hole) => (
            <line x1={x(hole.position)} y1={centerY} x2={x(hole.position)} y2={centerY - Math.max(12, hole.height * scale)} stroke="var(--color-accent)" stroke-width={Math.max(2, hole.diameter * scale)} />
          )}
        </For>
      </svg>
      <div class="mt-3 grid grid-cols-2 gap-x-6 gap-y-1 text-xs">
        <span style={{ color: "var(--color-text-muted)" }}>Bore length</span><span class="ws-mono">{props.result.bore_length.toFixed(3)} {props.result.length_type}</span>
        <span style={{ color: "var(--color-text-muted)" }}>Holes</span><span class="ws-mono">{props.result.holes.length}</span>
        <span style={{ color: "var(--color-text-muted)" }}>Mouthpiece</span><span>{props.result.mouthpiece.type}</span>
      </div>
    </div>
  );
}

function JsonView(props: { data: unknown }) {
  return (
    <pre class="overflow-auto rounded border p-3 ws-mono text-xs leading-relaxed" style={{ background: "var(--color-field)", "border-color": "var(--color-border)" }}>
      {JSON.stringify(props.data, null, 2)}
    </pre>
  );
}

function formatNumber(v: number): string {
  return Math.abs(v) < 0.001 ? v.toExponential(4) : v.toFixed(4);
}

function formatSigned(v: number): string {
  return `${v >= 0 ? "+" : ""}${formatNumber(v)}`;
}
