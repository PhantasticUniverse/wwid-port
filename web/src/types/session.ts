/** Document ID (mirrors Rust DocId). */
export type DocId = number;

/** Types of documents managed by the session. */
export type DocKind = "Instrument" | "Tuning" | "Constraints";

/** Info about a stored document. */
export interface DocInfo {
  doc_id: DocId;
  name: string;
  kind: DocKind;
}

/** Current session selection state (mirrors Rust Selection). */
export interface Selection {
  instrument_id: DocId | null;
  tuning_id: DocId | null;
  optimizer_key: string | null;
  constraints_id: DocId | null;
}

/** Result of opening an XML file (mirrors Rust OpenResult). */
export interface OpenResult {
  doc_id: DocId;
  doc_kind: DocKind;
  name: string;
}

/** Single row in an evaluation result (mirrors Rust EvalRow). */
export interface EvalRow {
  note: string;
  target_freq: number;
  predicted_freq: number;
  cents: number;
  weight: number;
}

/** Full evaluation result (mirrors Rust TuningResult). */
export interface TuningResult {
  rows: EvalRow[];
  net_error: number;
  mean_deviation: number;
}

/** Optimization progress update (mirrors Rust OptProgress). */
export interface OptProgress {
  evaluations: number;
  best_norm: number;
}

/** Result of an optimization run (mirrors Rust OptimizeResult). */
export interface OptimizeResult {
  new_instrument_id: DocId;
  initial_norm: number;
  final_norm: number;
  evaluations: number;
}

/** Result of calibration (mirrors Rust CalibResult — all fields optional except norm). */
export interface CalibResult {
  initial_fipple_factor?: number;
  final_fipple_factor?: number;
  initial_window_height?: number;
  final_window_height?: number;
  initial_airstream_length?: number;
  final_airstream_length?: number;
  initial_alpha?: number;
  final_alpha?: number;
  initial_beta?: number;
  final_beta?: number;
  initial_norm: number;
  final_norm: number;
}

/** Info about an available optimizer (mirrors Rust OptimizerInfo). */
export interface OptimizerInfo {
  key: string;
  display_name: string;
  objective_function_name: string;
}

/** Physical parameters of the session. */
export interface PhysicalParams {
  temperature: number;
  pressure: number;
  humidity: number;
  co2Ppm: number;
  speedOfSound: number;
  density: number;
  epsilonConstant: number;
}

/** An open editor tab in the workspace. */
export interface WorkspaceTab {
  id: string;
  docId: DocId;
  kind: DocKind;
  title: string;
}
