/** Messages sent from main thread to worker. */
export type WorkerCommand =
  | { type: "init"; studyKind: string }
  | { type: "exec"; id: number; cmd: string; args?: Record<string, unknown> }
  | { type: "optimize"; id: number }
  | { type: "cancel" };

/** Messages sent from worker to main thread. */
export type WorkerResponse =
  | { type: "ready" }
  | { type: "result"; id: number; ok: true; data: unknown }
  | { type: "error"; id: number; error: string }
  | { type: "progress"; evaluations: number; bestNorm: number }
  | { type: "optimizeResult"; id: number; ok: true; data: unknown }
  | { type: "optimizeError"; id: number; error: string };
