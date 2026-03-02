import type { WorkerCommand, WorkerResponse } from "../types/protocol";
import type { OptProgress } from "../types/session";

type PendingRequest = {
  resolve: (data: unknown) => void;
  reject: (error: Error) => void;
};

/**
 * Promise-based wrapper around the WASM compute worker.
 * All session commands go through exec(). Optimization gets
 * its own method because it needs a progress callback.
 */
export class ComputeService {
  private worker: Worker;
  private nextId = 1;
  private pending = new Map<number, PendingRequest>();
  private readyResolve: (() => void) | null = null;
  private optimizePending: PendingRequest | null = null;
  private onProgress: ((p: OptProgress) => void) | null = null;

  constructor() {
    this.worker = new Worker(
      new URL("../worker/compute-worker.ts", import.meta.url),
      { type: "module" }
    );
    this.worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      this.handleMessage(event.data);
    };
  }

  /** Initialize the WASM module and create a session. */
  async init(studyKind: string): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.readyResolve = resolve;

      // Also handle init errors
      const errorHandler = (event: MessageEvent<WorkerResponse>) => {
        const msg = event.data;
        if (msg.type === "error" && msg.id === 0) {
          this.readyResolve = null;
          reject(new Error(msg.error));
        }
      };
      this.worker.addEventListener("message", errorHandler, { once: true });

      this.send({ type: "init", studyKind });
    });
  }

  /** Run a session command. Returns the response data. */
  async run<T = unknown>(cmd: string, args?: Record<string, unknown>): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve, reject) => {
      this.pending.set(id, {
        resolve: resolve as (data: unknown) => void,
        reject,
      });
      this.send({ type: "exec", id, cmd, args });
    });
  }

  /** Run optimization with progress callback. */
  async optimize(onProgress: (p: OptProgress) => void): Promise<unknown> {
    const id = this.nextId++;
    this.onProgress = onProgress;
    return new Promise((resolve, reject) => {
      this.optimizePending = { resolve, reject };
      this.send({ type: "optimize", id });
    });
  }

  /** Cancel a running optimization. */
  cancel(): void {
    this.send({ type: "cancel" });
  }

  /** Terminate the worker. */
  destroy(): void {
    this.worker.terminate();
  }

  private send(msg: WorkerCommand): void {
    this.worker.postMessage(msg);
  }

  private handleMessage(msg: WorkerResponse): void {
    switch (msg.type) {
      case "ready":
        if (this.readyResolve) {
          this.readyResolve();
          this.readyResolve = null;
        }
        break;

      case "result": {
        const p = this.pending.get(msg.id);
        if (p) {
          this.pending.delete(msg.id);
          p.resolve(msg.data);
        }
        break;
      }

      case "error": {
        const p = this.pending.get(msg.id);
        if (p) {
          this.pending.delete(msg.id);
          p.reject(new Error(msg.error));
        }
        break;
      }

      case "progress":
        if (this.onProgress) {
          this.onProgress({
            evaluations: msg.evaluations,
            best_norm: msg.bestNorm,
          });
        }
        break;

      case "optimizeResult":
        if (this.optimizePending) {
          this.optimizePending.resolve(msg.data);
          this.optimizePending = null;
          this.onProgress = null;
        }
        break;

      case "optimizeError":
        if (this.optimizePending) {
          this.optimizePending.reject(new Error(msg.error));
          this.optimizePending = null;
          this.onProgress = null;
        }
        break;
    }
  }
}
