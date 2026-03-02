import { createSignal, createMemo } from "solid-js";
import { createStore, produce } from "solid-js/store";
import type {
  DocId,
  DocInfo,
  Selection,
  TuningResult,
  OptimizerInfo,
  PhysicalParams,
} from "../types/session";
import { ComputeService } from "../services/ComputeService";

// ── Compute service singleton ────────────────────────────────
const compute = new ComputeService();

// ── Reactive state ───────────────────────────────────────────
const [ready, setReady] = createSignal(false);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);

const [instruments, setInstruments] = createStore<DocInfo[]>([]);
const [tunings, setTunings] = createStore<DocInfo[]>([]);
const [constraints, setConstraints] = createStore<DocInfo[]>([]);

const [selection, setSelection] = createSignal<Selection>({
  instrument_id: null,
  tuning_id: null,
  optimizer_key: null,
  constraints_id: null,
});

const [optimizers, setOptimizers] = createSignal<OptimizerInfo[]>([]);
const [params, setParams] = createSignal<PhysicalParams | null>(null);
const [lastEval, setLastEval] = createSignal<TuningResult | null>(null);
const [consoleLogs, setConsoleLogs] = createStore<string[]>([]);

function log(msg: string) {
  setConsoleLogs(produce((logs) => logs.push(msg)));
}

// ── Gating (computed from selection) ─────────────────────────
const canTune = createMemo(() => {
  const sel = selection();
  return sel.instrument_id !== null && sel.tuning_id !== null;
});

const canOptimize = createMemo(() => {
  const sel = selection();
  return canTune() && sel.optimizer_key !== null && sel.constraints_id !== null;
});

const canSketch = createMemo(() => {
  return selection().instrument_id !== null;
});

// ── Actions ──────────────────────────────────────────────────

async function init() {
  try {
    setLoading(true);
    await compute.init("NAF");
    const opts = await compute.run<OptimizerInfo[]>("available_optimizers");
    setOptimizers(opts);
    const p = await compute.run<PhysicalParams>("get_params");
    setParams(p);
    log(
      `Properties of air at ${p.temperature.toFixed(2)} C, ` +
        `${p.pressure.toFixed(3)} kPa, ` +
        `${p.humidity.toFixed(0)}% humidity, ` +
        `${p.co2Ppm.toFixed(0)} ppm CO2:`
    );
    log(`Speed of sound is ${p.speedOfSound.toFixed(3)} m/s.`);
    log(`Density is ${p.density.toFixed(4)} kg/m^3.`);
    log(`Epsilon factor is ${p.epsilonConstant.toExponential(3)}.`);
    setReady(true);
  } catch (e) {
    setError(`Failed to initialize: ${e}`);
  } finally {
    setLoading(false);
  }
}

async function openXml(xml: string) {
  try {
    setLoading(true);
    setError(null);
    const result = await compute.run<{ doc_id: number; doc_kind: string; name: string }>(
      "open_xml",
      { xml }
    );
    const info: DocInfo = {
      doc_id: result.doc_id,
      name: result.name,
      kind: result.doc_kind as DocInfo["kind"],
    };
    switch (info.kind) {
      case "Instrument":
        setInstruments(produce((list) => list.push(info)));
        log(`Opened instrument: ${info.name}`);
        break;
      case "Tuning":
        setTunings(produce((list) => list.push(info)));
        log(`Opened tuning: ${info.name}`);
        break;
      case "Constraints":
        setConstraints(produce((list) => list.push(info)));
        log(`Opened constraints: ${info.name}`);
        break;
    }
    return info;
  } catch (e) {
    setError(`Failed to open XML: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function selectInstrument(docId: DocId) {
  await compute.run("select_instrument", { docId });
  setSelection((s) => ({ ...s, instrument_id: docId }));
  await refreshGating();
}

async function selectTuning(docId: DocId) {
  await compute.run("select_tuning", { docId });
  setSelection((s) => ({ ...s, tuning_id: docId }));
  await refreshGating();
}

async function selectOptimizer(key: string) {
  await compute.run("select_optimizer", { key });
  setSelection((s) => ({ ...s, optimizer_key: key }));
}

async function selectConstraints(docId: DocId) {
  await compute.run("select_constraints", { docId });
  setSelection((s) => ({ ...s, constraints_id: docId }));
}

async function refreshGating() {
  // Fetch the actual gating state from the engine
  const sel = await compute.run<Selection>("get_selection");
  setSelection(sel);
}

async function evaluateTuning(): Promise<TuningResult | null> {
  try {
    setLoading(true);
    setError(null);
    const result = await compute.run<TuningResult>("evaluate_tuning");
    setLastEval(result);
    log(
      `Evaluation: net error ${result.net_error.toFixed(2)} cents, ` +
        `mean deviation ${result.mean_deviation.toFixed(2)} cents`
    );
    return result;
  } catch (e) {
    setError(`Evaluation failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function updateParams(temperature: number, humidity: number) {
  try {
    setError(null);
    const p = await compute.run<PhysicalParams>("set_params", { temperature, humidity });
    setParams(p);
    log(
      `Settings updated: ${p.temperature.toFixed(2)} C, ` +
        `${p.humidity.toFixed(0)}% humidity`
    );
    log(`Speed of sound is ${p.speedOfSound.toFixed(3)} m/s.`);
    log(`Density is ${p.density.toFixed(4)} kg/m^3.`);
  } catch (e) {
    setError(`Failed to update params: ${e}`);
  }
}

async function exportXml(docId: DocId): Promise<string | null> {
  try {
    return await compute.run<string>("export_xml", { docId });
  } catch (e) {
    setError(`Export failed: ${e}`);
    return null;
  }
}

async function saveInstrumentXml(docId: DocId) {
  const xml = await exportXml(docId);
  if (!xml) return;
  // Find the instrument name for the filename
  const inst = instruments.find((d) => d.doc_id === docId);
  const name = inst?.name ?? "instrument";
  const blob = new Blob([xml], { type: "application/xml" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${name}.xml`;
  a.click();
  URL.revokeObjectURL(url);
  log(`Saved ${name}.xml`);
}

// ── Exported store ───────────────────────────────────────────
export const sessionStore = {
  // State
  ready,
  loading,
  error,
  instruments,
  tunings,
  constraints,
  selection,
  optimizers,
  params,
  lastEval,
  consoleLogs,

  // Gating
  canTune,
  canOptimize,
  canSketch,

  // Actions
  init,
  openXml,
  selectInstrument,
  selectTuning,
  selectOptimizer,
  selectConstraints,
  evaluateTuning,
  exportXml,
  saveInstrumentXml,
  updateParams,
  log,

  // Raw compute access for advanced use
  compute,
};
