/// <reference lib="webworker" />

import type { WorkerCommand, WorkerResponse } from "../types/protocol";

// WASM module — loaded dynamically in init()
let session: any = null;
let cancelled = false;

function post(msg: WorkerResponse) {
  self.postMessage(msg);
}

async function handleInit(studyKind: string) {
  // Dynamic import of the WASM module
  const wasm = await import("@wasm/wid_wasm");
  await wasm.default();
  session = new wasm.WasmSession(studyKind);
  post({ type: "ready" });
}

function handleExec(id: number, cmd: string, args?: Record<string, unknown>) {
  if (!session) {
    post({ type: "error", id, error: "Session not initialized" });
    return;
  }

  const commandJson = JSON.stringify({ cmd, args: args ?? {} });
  const resultJson = session.execute(commandJson);

  try {
    const result = JSON.parse(resultJson);
    if (result.ok) {
      post({ type: "result", id, ok: true, data: result.data });
    } else {
      post({ type: "error", id, error: result.error ?? "Unknown error" });
    }
  } catch {
    post({ type: "error", id, error: `Invalid response JSON: ${resultJson}` });
  }
}

function handleOptimize(id: number) {
  if (!session) {
    post({ type: "error", id, error: "Session not initialized" });
    return;
  }

  cancelled = false;

  const progressCallback = (progressJson: string) => {
    if (cancelled) return false;
    try {
      const progress = JSON.parse(progressJson);
      post({
        type: "progress",
        evaluations: progress.evaluations ?? 0,
        bestNorm: progress.best_norm ?? 0,
      });
    } catch {
      // Ignore parse errors in progress
    }
    return !cancelled;
  };

  const resultJson = session.optimize(progressCallback);

  try {
    const result = JSON.parse(resultJson);
    if (result.ok) {
      post({ type: "optimizeResult", id, ok: true, data: result.data });
    } else {
      post({ type: "optimizeError", id, error: result.error ?? "Optimization failed" });
    }
  } catch {
    post({ type: "optimizeError", id, error: `Invalid response JSON: ${resultJson}` });
  }
}

self.onmessage = async (event: MessageEvent<WorkerCommand>) => {
  const msg = event.data;

  switch (msg.type) {
    case "init":
      try {
        await handleInit(msg.studyKind);
      } catch (e) {
        post({ type: "error", id: 0, error: `Init failed: ${e}` });
      }
      break;

    case "exec":
      handleExec(msg.id, msg.cmd, msg.args);
      break;

    case "optimize":
      handleOptimize(msg.id);
      break;

    case "cancel":
      cancelled = true;
      break;
  }
};
