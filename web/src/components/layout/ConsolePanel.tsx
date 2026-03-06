import { For } from "solid-js";
import { sessionStore } from "../../stores/session";

export default function ConsolePanel() {
  return (
    <div
      class="border-t overflow-y-auto"
      style={{
        background: "var(--color-surface-alt)",
        "border-color": "var(--color-border)",
        height: "120px",
        "min-height": "80px",
      }}
    >
      <div
        class="px-3 py-1 text-xs font-semibold uppercase tracking-wider"
        style={{ color: "var(--color-text-muted)", "border-bottom": "1px solid var(--color-border)" }}
      >
        Console
      </div>
      <div
        class="px-3 py-1 font-mono text-sm leading-relaxed"
        style={{ color: "var(--color-text)", opacity: "0.7" }}
      >
        <For each={sessionStore.consoleLogs}>{(line) => <div>{line}</div>}</For>
      </div>
    </div>
  );
}
