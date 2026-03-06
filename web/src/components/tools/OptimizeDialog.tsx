import { Show, createEffect, onCleanup } from "solid-js";
import type { OptProgress, OptimizeResult, CalibResult } from "../../types/session";

/** Type guard: CalibResult has initial_norm but NOT new_instrument_id. */
function isCalibResult(r: OptimizeResult | CalibResult): r is CalibResult {
  return !("new_instrument_id" in r);
}

export interface OptimizeDialogProps {
  open: boolean;
  isFipple: boolean;
  progress: OptProgress | null;
  result: OptimizeResult | CalibResult | null;
  onCancel: () => void;
  onClose: () => void;
}

export default function OptimizeDialog(props: OptimizeDialogProps) {
  // Esc to close only when showing result (not during progress)
  createEffect(() => {
    if (!props.open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape" && props.result) props.onClose();
    };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  return (
    <Show when={props.open}>
      <div
        class="fixed inset-0 flex items-center justify-center"
        style={{ background: "rgba(0,0,0,0.4)", "z-index": "50" }}
        onClick={(e) => {
          if (e.target === e.currentTarget && props.result) props.onClose();
        }}
      >
        <div
          class="rounded-lg shadow-lg p-6 w-96"
          style={{ background: "var(--color-surface)", border: "1px solid var(--color-border)" }}
        >
          <Show when={!props.result} fallback={<ResultView {...props} />}>
            <ProgressView {...props} />
          </Show>
        </div>
      </div>
    </Show>
  );
}

/** In-progress state: spinner + live stats + cancel button. */
function ProgressView(props: OptimizeDialogProps) {
  return (
    <div class="flex flex-col items-center gap-4">
      <h2 class="text-lg font-semibold">
        {props.isFipple ? "Calibrating Fipple Factor..." : "Optimizing..."}
      </h2>

      {/* Spinner */}
      <div
        class="w-8 h-8 rounded-full border-2 animate-spin"
        style={{
          "border-color": "var(--color-border)",
          "border-top-color": "var(--color-accent)",
        }}
      />

      {/* Progress stats (hole optimization only) */}
      <Show when={!props.isFipple && props.progress}>
        {(p) => (
          <div class="text-sm text-center" style={{ color: "var(--color-text-muted)" }}>
            <div>Evaluations: {p().evaluations.toLocaleString()}</div>
            <div>Best norm: {p().best_norm.toFixed(4)}</div>
          </div>
        )}
      </Show>

      <button
        class="px-4 py-1.5 rounded text-sm font-medium transition-colors"
        style={{ background: "#dc2626", color: "white" }}
        onClick={props.onCancel}
      >
        Cancel
      </button>
    </div>
  );
}

/** Result state: shows outcome and a close button. */
function ResultView(props: OptimizeDialogProps) {
  const result = () => props.result!;

  return (
    <div class="flex flex-col gap-4">
      <h2 class="text-lg font-semibold">
        {props.isFipple ? "Calibration Complete" : "Optimization Complete"}
      </h2>

      <Show when={isCalibResult(result())}>
        {(_) => {
          const r = result() as CalibResult;
          return (
            <div class="text-sm flex flex-col gap-1" style={{ color: "var(--color-text)" }}>
              {r.initial_fipple_factor != null && r.final_fipple_factor != null && (
                <Row label="Fipple factor" before={r.initial_fipple_factor.toFixed(4)} after={r.final_fipple_factor.toFixed(4)} />
              )}
              {r.initial_window_height != null && r.final_window_height != null && (
                <Row label="Window height" before={r.initial_window_height.toFixed(6)} after={r.final_window_height.toFixed(6)} />
              )}
              {r.initial_airstream_length != null && r.final_airstream_length != null && (
                <Row label="Airstream length" before={r.initial_airstream_length.toFixed(6)} after={r.final_airstream_length.toFixed(6)} />
              )}
              {r.initial_alpha != null && r.final_alpha != null && (
                <Row label="Alpha" before={r.initial_alpha.toFixed(6)} after={r.final_alpha.toFixed(6)} />
              )}
              {r.initial_beta != null && r.final_beta != null && (
                <Row label="Beta" before={r.initial_beta.toFixed(6)} after={r.final_beta.toFixed(6)} />
              )}
              <Row label="Norm" before={r.initial_norm.toFixed(4)} after={r.final_norm.toFixed(4)} />
            </div>
          );
        }}
      </Show>

      <Show when={!isCalibResult(result())}>
        {(_) => {
          const r = result() as OptimizeResult;
          return (
            <div class="text-sm flex flex-col gap-1" style={{ color: "var(--color-text)" }}>
              <div>Evaluations: {r.evaluations.toLocaleString()}</div>
              <Row label="Norm" before={r.initial_norm.toFixed(4)} after={r.final_norm.toFixed(4)} />
              <div class="mt-2" style={{ color: "var(--color-text-muted)" }}>
                New instrument added to the file list.
              </div>
            </div>
          );
        }}
      </Show>

      <div class="flex justify-end">
        <button
          class="px-4 py-1.5 rounded text-sm font-medium"
          style={{ background: "var(--color-accent)", color: "white" }}
          onClick={props.onClose}
        >
          Close
        </button>
      </div>
    </div>
  );
}

/** Before/after row for result display. */
function Row(props: { label: string; before: string; after: string }) {
  return (
    <div class="flex justify-between">
      <span>{props.label}:</span>
      <span style={{ "font-family": "monospace" }}>
        {props.before} → {props.after}
      </span>
    </div>
  );
}
