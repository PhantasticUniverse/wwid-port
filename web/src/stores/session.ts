import { createSignal, createMemo } from "solid-js";
import { createStore, produce } from "solid-js/store";
import type {
  DocId,
  DocInfo,
  DocKind,
  Selection,
  TuningResult,
  OptProgress,
  OptimizeResult,
  CalibResult,
  OptimizerInfo,
  PhysicalParams,
  WorkspaceTab,
} from "../types/session";
import type {
  InstrumentData,
  TuningData,
  ConstraintsData,
} from "../types/documents";
import { ComputeService } from "../services/ComputeService";
import { openEvalPopup } from "../components/tools/EvalPopup";

// ── Compute service singleton ────────────────────────────────
const compute = new ComputeService();

// ── Reactive state ───────────────────────────────────────────
const [ready, setReady] = createSignal(false);
const [loading, setLoading] = createSignal(false);
const [error, setError] = createSignal<string | null>(null);
const [studyKind, setStudyKind] = createSignal("NAF");

const [instruments, setInstruments] = createStore<DocInfo[]>([]);
const [tunings, setTunings] = createStore<DocInfo[]>([]);
const [constraints, setConstraintsList] = createStore<DocInfo[]>([]);

const [selection, setSelection] = createSignal<Selection>({
  instrument_id: null,
  tuning_id: null,
  optimizer_key: null,
  constraints_id: null,
});

const [optimizers, setOptimizers] = createSignal<OptimizerInfo[]>([]);
const [params, setParams] = createSignal<PhysicalParams | null>(null);
const [consoleLogs, setConsoleLogs] = createStore<string[]>([]);
const [optimizing, setOptimizing] = createSignal(false);
const [optProgress, setOptProgress] = createSignal<OptProgress | null>(null);

// Incremented after fipple calibration so editors re-fetch instrument data
// (calibration mutates the instrument in-place on the Rust side)
const [calibrationCount, setCalibrationCount] = createSignal(0);

// ── Tab state ────────────────────────────────────────────────
const [tabs, setTabs] = createStore<WorkspaceTab[]>([]);
const [activeTabId, setActiveTabId] = createSignal<string | null>(null);

function log(msg: string) {
  setConsoleLogs(produce((logs) => logs.push(msg)));
}

// ── Gating (computed from selection) ─────────────────────────
const canTune = createMemo(() => {
  const sel = selection();
  return sel.instrument_id !== null && sel.tuning_id !== null;
});

const CALIBRATOR_KEYS = new Set([
  "FippleFactorObjectiveFunction",
  "WhistleCalibrationObjectiveFunction",
  "WindowHeightObjectiveFunction",
  "BetaObjectiveFunction",
  "FluteCalibrationObjectiveFunction",
  "AirstreamLengthObjectiveFunction",
  "ReedCalibratorObjectiveFunction",
]);

const isCalibratorSelected = createMemo(() => {
  const key = selection().optimizer_key;
  return key !== null && CALIBRATOR_KEYS.has(key);
});

const canOptimize = createMemo(() => {
  const sel = selection();
  if (!canTune() || sel.optimizer_key === null) return false;
  // Calibrators don't require constraints
  if (CALIBRATOR_KEYS.has(sel.optimizer_key)) return true;
  return sel.constraints_id !== null;
});

const canSketch = createMemo(() => {
  return selection().instrument_id !== null;
});

const canCreateConstraints = createMemo(() => {
  const sel = selection();
  return sel.instrument_id !== null && sel.optimizer_key !== null;
});

// ── Actions ──────────────────────────────────────────────────

async function init() {
  try {
    setLoading(true);
    await compute.init(studyKind());
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
        await selectInstrument(info.doc_id);
        break;
      case "Tuning":
        setTunings(produce((list) => list.push(info)));
        log(`Opened tuning: ${info.name}`);
        await selectTuning(info.doc_id);
        break;
      case "Constraints":
        setConstraintsList(produce((list) => list.push(info)));
        log(`Opened constraints: ${info.name}`);
        await selectConstraints(info.doc_id);
        break;
    }
    openTab(info.doc_id, info.kind, info.name);
    return info;
  } catch (e) {
    setError(`Failed to open XML: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function selectInstrument(docId: DocId) {
  setError(null);
  await compute.run("select_instrument", { docId });
  setSelection((s) => ({ ...s, instrument_id: docId }));
  await refreshGating();
}

async function selectTuning(docId: DocId) {
  setError(null);
  await compute.run("select_tuning", { docId });
  setSelection((s) => ({ ...s, tuning_id: docId }));
  await refreshGating();
}

async function selectOptimizer(key: string) {
  setError(null);
  await compute.run("select_optimizer", { key });
  setSelection((s) => ({ ...s, optimizer_key: key }));
  await refreshGating();
}

async function selectConstraints(docId: DocId) {
  setError(null);
  await compute.run("select_constraints", { docId });
  setSelection((s) => ({ ...s, constraints_id: docId }));
  await refreshGating();
}

async function refreshGating() {
  const sel = await compute.run<Selection>("get_selection");
  setSelection(sel);
}

// ── Tab management ───────────────────────────────────────────

function openTab(docId: DocId, kind: DocKind, title: string) {
  const existing = tabs.find((t) => t.docId === docId);
  if (existing) {
    setActiveTabId(existing.id);
    return;
  }
  const id = `tab-${docId}-${Date.now()}`;
  const tab: WorkspaceTab = { id, docId, kind, title };
  setTabs(produce((list) => list.push(tab)));
  setActiveTabId(id);
}

function closeTab(tabId: string) {
  setTabs(produce((list) => {
    const idx = list.findIndex((t) => t.id === tabId);
    if (idx >= 0) list.splice(idx, 1);
  }));
  if (activeTabId() === tabId) {
    setActiveTabId(tabs.length > 0 ? tabs[tabs.length - 1]?.id ?? null : null);
  }
}

// ── Document get/set ─────────────────────────────────────────

async function getInstrument(docId: DocId): Promise<InstrumentData> {
  return compute.run<InstrumentData>("get_instrument", { docId });
}

async function setInstrument(docId: DocId, data: InstrumentData) {
  await compute.run("set_instrument", { docId, data });
  // Update name in doc list
  setInstruments(
    (d) => d.doc_id === docId,
    "name",
    data.name,
  );
}

async function getTuning(docId: DocId): Promise<TuningData> {
  return compute.run<TuningData>("get_tuning", { docId });
}

async function setTuning(docId: DocId, data: TuningData) {
  await compute.run("set_tuning", { docId, data });
  setTunings(
    (d) => d.doc_id === docId,
    "name",
    data.name,
  );
}

async function getConstraints(docId: DocId): Promise<ConstraintsData> {
  return compute.run<ConstraintsData>("get_constraints", { docId });
}

async function setConstraints(docId: DocId, data: ConstraintsData) {
  await compute.run("set_constraints", { docId, data });
  setConstraintsList(
    (d) => d.doc_id === docId,
    "name",
    data.constraintsName,
  );
}

// ── Evaluation (popup) ──────────────────────────────────────

async function evaluateTuning(): Promise<TuningResult | null> {
  try {
    setLoading(true);
    setError(null);
    const result = await compute.run<TuningResult>("evaluate_tuning");
    log(
      `Evaluation: net error ${result.net_error.toFixed(2)} cents, ` +
        `mean deviation ${result.mean_deviation.toFixed(2)} cents`
    );
    // Open results in a popup window
    const instName =
      instruments.find((d) => d.doc_id === selection().instrument_id)?.name ?? "Instrument";
    openEvalPopup(result, instName);
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

/** Save any document (instrument, tuning, or constraints) as an XML download. */
async function saveDocXml(docId: DocId) {
  const xml = await exportXml(docId);
  if (!xml) return;
  const name =
    instruments.find((d) => d.doc_id === docId)?.name ??
    tunings.find((d) => d.doc_id === docId)?.name ??
    constraints.find((d) => d.doc_id === docId)?.name ??
    `doc-${docId}`;
  const blob = new Blob([xml], { type: "application/xml" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${name}.xml`;
  a.click();
  URL.revokeObjectURL(url);
  log(`Saved ${name}.xml`);
}

// ── Optimization / Calibration ────────────────────────────────

/** Run optimization (hole) or calibration (fipple) based on current selection. */
async function runOptimize(): Promise<OptimizeResult | CalibResult | null> {
  try {
    setOptimizing(true);
    setOptProgress(null);
    setError(null);

    if (isCalibratorSelected()) {
      // Calibration — sync command, modifies instrument in-place
      const result = await compute.run<CalibResult>("calibrate");
      setCalibrationCount((c) => c + 1);
      const parts: string[] = [];
      if (result.initial_fipple_factor != null && result.final_fipple_factor != null)
        parts.push(`fipple factor ${result.initial_fipple_factor.toFixed(4)} → ${result.final_fipple_factor.toFixed(4)}`);
      if (result.initial_window_height != null && result.final_window_height != null)
        parts.push(`window height ${result.initial_window_height.toFixed(6)} → ${result.final_window_height.toFixed(6)}`);
      if (result.initial_airstream_length != null && result.final_airstream_length != null)
        parts.push(`airstream length ${result.initial_airstream_length.toFixed(6)} → ${result.final_airstream_length.toFixed(6)}`);
      if (result.initial_alpha != null && result.final_alpha != null)
        parts.push(`alpha ${result.initial_alpha.toFixed(6)} → ${result.final_alpha.toFixed(6)}`);
      if (result.initial_beta != null && result.final_beta != null)
        parts.push(`beta ${result.initial_beta.toFixed(6)} → ${result.final_beta.toFixed(6)}`);
      parts.push(`norm ${result.initial_norm.toFixed(4)} → ${result.final_norm.toFixed(4)}`);
      log(`Calibration complete: ${parts.join(", ")}`);
      return result;
    }

    // Hole optimization — async with progress streaming
    const result = await compute.optimize((p) => setOptProgress(p)) as OptimizeResult;

    // Fetch name of the new instrument and add it to the doc list
    const newInst = await compute.run<InstrumentData>("get_instrument", { docId: result.new_instrument_id });
    const info: DocInfo = {
      doc_id: result.new_instrument_id,
      name: newInst.name,
      kind: "Instrument",
    };
    setInstruments(produce((list) => list.push(info)));
    setSelection((s) => ({ ...s, instrument_id: result.new_instrument_id }));
    openTab(result.new_instrument_id, "Instrument", newInst.name);

    await refreshGating();
    log(
      `Optimization complete: ${result.evaluations} evaluations, ` +
        `norm ${result.initial_norm.toFixed(4)} → ${result.final_norm.toFixed(4)}`
    );
    return result;
  } catch (e) {
    const msg = String(e);
    // Cancellation is graceful, not an error
    if (msg.includes("cancelled") || msg.includes("Cancelled")) {
      log("Optimization cancelled.");
      return null;
    }
    setError(`Optimization failed: ${msg}`);
    return null;
  } finally {
    setOptimizing(false);
    setOptProgress(null);
  }
}

function cancelOptimize() {
  compute.cancel();
}

// ── Constraint creation ──────────────────────────────────────

/** Create default constraints for the currently selected optimizer. */
async function createDefaultConstraints() {
  const key = selection().optimizer_key;
  if (!key) return;
  try {
    setError(null);
    const result = await compute.run<{ doc_id: number; doc_kind: string; name: string }>(
      "create_default_constraints",
      { optimizerKey: key }
    );
    const info: DocInfo = { doc_id: result.doc_id, name: result.name, kind: "Constraints" };
    setConstraintsList(produce((list) => list.push(info)));
    await selectConstraints(info.doc_id);
    openTab(info.doc_id, "Constraints", info.name);
    log(`Created default constraints: ${info.name}`);
  } catch (e) {
    setError(`Failed to create constraints: ${e}`);
  }
}

/** Create blank constraints for the currently selected optimizer. */
async function createBlankConstraints() {
  const key = selection().optimizer_key;
  if (!key) return;
  try {
    setError(null);
    const result = await compute.run<{ doc_id: number; doc_kind: string; name: string }>(
      "create_blank_constraints",
      { optimizerKey: key }
    );
    const info: DocInfo = { doc_id: result.doc_id, name: result.name, kind: "Constraints" };
    setConstraintsList(produce((list) => list.push(info)));
    await selectConstraints(info.doc_id);
    openTab(info.doc_id, "Constraints", info.name);
    log(`Created blank constraints: ${info.name}`);
  } catch (e) {
    setError(`Failed to create constraints: ${e}`);
  }
}

// ── Study model switching ────────────────────────────────────

async function switchStudyModel(kind: string) {
  setStudyKind(kind);
  // Clear all state
  setInstruments([]);
  setTunings([]);
  setConstraintsList([]);
  setSelection({
    instrument_id: null,
    tuning_id: null,
    optimizer_key: null,
    constraints_id: null,
  });
  setTabs([]);
  setActiveTabId(null);
  setOptimizers([]);
  setParams(null);
  setReady(false);
  setError(null);
  // Re-initialize with new study kind
  await init();
}

// ── Tool actions ─────────────────────────────────────────────

async function sketchInstrument() {
  try {
    setLoading(true);
    setError(null);
    return await compute.run("sketch_instrument");
  } catch (e) {
    setError(`Sketch failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function supplementaryInfo() {
  try {
    setLoading(true);
    setError(null);
    return await compute.run("supplementary_info");
  } catch (e) {
    setError(`Supplementary info failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function compareInstruments(oldId: DocId, newId: DocId) {
  try {
    setLoading(true);
    setError(null);
    return await compute.run("compare_instruments", { oldDocId: oldId, newDocId: newId });
  } catch (e) {
    setError(`Compare failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function graphTuning() {
  try {
    setLoading(true);
    setError(null);
    return await compute.run("graph_tuning");
  } catch (e) {
    setError(`Graph tuning failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

async function noteSpectrum(fingeringIndex: number, freqMult?: number) {
  try {
    setLoading(true);
    setError(null);
    return await compute.run("note_spectrum", { fingeringIndex, ...(freqMult != null && { freqMult }) });
  } catch (e) {
    setError(`Note spectrum failed: ${e}`);
    return null;
  } finally {
    setLoading(false);
  }
}

// ── Wizard actions ───────────────────────────────────────────

async function listTemperaments(): Promise<DocInfo[]> {
  try {
    return await compute.run<DocInfo[]>("list_temperaments");
  } catch (e) {
    setError(`List temperaments failed: ${e}`);
    return [];
  }
}

async function listScales(): Promise<DocInfo[]> {
  try {
    return await compute.run<DocInfo[]>("list_scales");
  } catch (e) {
    setError(`List scales failed: ${e}`);
    return [];
  }
}

async function listFingeringPatterns(): Promise<DocInfo[]> {
  try {
    return await compute.run<DocInfo[]>("list_fingering_patterns");
  } catch (e) {
    setError(`List fingering patterns failed: ${e}`);
    return [];
  }
}

async function listScaleSymbolLists(): Promise<DocInfo[]> {
  try {
    return await compute.run<DocInfo[]>("list_scale_symbol_lists");
  } catch (e) {
    setError(`List scale symbol lists failed: ${e}`);
    return [];
  }
}

async function generateScale(opts: {
  temperamentId?: DocId;
  temperament?: string;
  symbolsId?: DocId;
  symbols?: string;
  refName: string;
  refFrequency: number;
  scaleName?: string;
}) {
  try {
    setError(null);
    const args: Record<string, unknown> = {
      refName: opts.refName,
      refFrequency: opts.refFrequency,
    };
    if (opts.scaleName) args.scaleName = opts.scaleName;
    if (opts.temperamentId != null) args.temperamentId = opts.temperamentId;
    else if (opts.temperament) args.temperament = opts.temperament;
    if (opts.symbolsId != null) args.symbolsId = opts.symbolsId;
    else if (opts.symbols) args.symbols = opts.symbols;

    const result = await compute.run<{ doc_id: number; doc_kind: string; name: string }>(
      "generate_scale",
      args
    );
    log(`Generated scale: ${result.name}`);
    return result;
  } catch (e) {
    setError(`Generate scale failed: ${e}`);
    return null;
  }
}

async function generateTuning(scaleId: DocId, patternId: DocId, name: string) {
  try {
    setError(null);
    const result = await compute.run<{ doc_id: number; doc_kind: string; name: string }>(
      "generate_tuning",
      { scaleId, patternId, name }
    );
    const info: DocInfo = { doc_id: result.doc_id, name: result.name, kind: "Tuning" };
    setTunings(produce((list) => list.push(info)));
    await selectTuning(info.doc_id);
    openTab(info.doc_id, "Tuning", info.name);
    log(`Generated tuning: ${result.name}`);
    return result;
  } catch (e) {
    setError(`Generate tuning failed: ${e}`);
    return null;
  }
}

// ── Exported store ───────────────────────────────────────────
export const sessionStore = {
  // State
  ready,
  loading,
  error,
  studyKind,
  instruments,
  tunings,
  constraints,
  selection,
  optimizers,
  params,
  consoleLogs,
  tabs,
  activeTabId,
  optimizing,
  optProgress,
  calibrationCount,

  // Gating
  canTune,
  canOptimize,
  canSketch,
  isCalibratorSelected,
  canCreateConstraints,

  // Actions
  init,
  switchStudyModel,
  openXml,
  selectInstrument,
  selectTuning,
  selectOptimizer,
  selectConstraints,
  evaluateTuning,
  exportXml,
  saveDocXml,
  updateParams,
  log,

  // Optimization / Calibration
  runOptimize,
  cancelOptimize,
  createDefaultConstraints,
  createBlankConstraints,
  CALIBRATOR_KEYS,

  // Tab management
  openTab,
  closeTab,
  setActiveTabId,

  // Document access
  getInstrument,
  setInstrument,
  getTuning,
  setTuning,
  getConstraints,
  setConstraints,

  // Tool actions
  sketchInstrument,
  supplementaryInfo,
  compareInstruments,
  graphTuning,
  noteSpectrum,

  // Wizard actions
  listTemperaments,
  listScales,
  listFingeringPatterns,
  listScaleSymbolLists,
  generateScale,
  generateTuning,

  // Raw compute access for advanced use
  compute,
};
