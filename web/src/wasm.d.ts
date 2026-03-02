declare module "@wasm/wid_wasm" {
  export class WasmSession {
    constructor(study_kind: string);
    free(): void;
    execute(command_json: string): string;
    optimize(progress_callback: Function): string;
  }
  export default function init(): Promise<void>;
}
