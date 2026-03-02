import { createSignal } from "solid-js";
import { sessionStore } from "../../stores/session";

export default function SettingsDialog(props: {
  onClose: () => void;
}) {
  const p = sessionStore.params();
  const [temp, setTemp] = createSignal(p?.temperature ?? 20.0);
  const [humidity, setHumidity] = createSignal(p?.humidity ?? 45.0);

  return (
    <div
      class="fixed inset-0 flex items-center justify-center"
      style={{ background: "rgba(0,0,0,0.4)", "z-index": "50" }}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <div
        class="rounded-lg shadow-lg p-6 w-96"
        style={{ background: "var(--color-surface)", border: "1px solid var(--color-border)" }}
      >
        <h2 class="text-lg font-semibold mb-4">Settings</h2>

        <div class="flex flex-col gap-4">
          <div class="flex items-center justify-between">
            <label class="text-sm" style={{ color: "var(--color-text)" }}>
              Temperature, C:
            </label>
            <input
              type="number"
              step="0.1"
              class="w-24 px-2 py-1 rounded text-sm text-right"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={temp()}
              onInput={(e) => setTemp(parseFloat(e.currentTarget.value) || 0)}
            />
          </div>

          <div class="flex items-center justify-between">
            <label class="text-sm" style={{ color: "var(--color-text)" }}>
              Relative Humidity, %:
            </label>
            <input
              type="number"
              step="1"
              min="0"
              max="100"
              class="w-24 px-2 py-1 rounded text-sm text-right"
              style={{
                background: "var(--color-surface-alt)",
                border: "1px solid var(--color-border)",
                color: "var(--color-text)",
              }}
              value={humidity()}
              onInput={(e) => setHumidity(parseFloat(e.currentTarget.value) || 0)}
            />
          </div>
        </div>

        <div class="flex justify-end gap-2 mt-6">
          <button
            class="px-3 py-1.5 rounded text-sm"
            style={{ color: "var(--color-text-muted)" }}
            onClick={() => props.onClose()}
          >
            Cancel
          </button>
          <button
            class="px-3 py-1.5 rounded text-sm font-medium"
            style={{ background: "var(--color-accent)", color: "white" }}
            onClick={async () => {
              await sessionStore.updateParams(temp(), humidity());
              props.onClose();
            }}
          >
            Apply
          </button>
        </div>
      </div>
    </div>
  );
}
