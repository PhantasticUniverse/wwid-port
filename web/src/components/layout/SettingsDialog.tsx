import { createSignal, onMount, onCleanup } from "solid-js";
import { sessionStore } from "../../stores/session";

export default function SettingsDialog(props: {
  onClose: () => void;
}) {
  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") props.onClose(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  const p = sessionStore.params();
  const [temp, setTemp] = createSignal(Math.round((p?.temperature ?? 20.0) * 100) / 100);
  const [humidity, setHumidity] = createSignal(p?.humidity ?? 45.0);
  const [useDirect, setUseDirect] = createSignal(getUseDirect());
  const [lengthType, setLengthType] = createSignal(getLengthType());
  const [spectrumMult, setSpectrumMult] = createSignal(getSpectrumMult());

  const inputStyle = {
    background: "var(--color-surface-alt)",
    border: "1px solid var(--color-border)",
    color: "var(--color-text)",
  };

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
              Length Type:
            </label>
            <select
              class="w-24 px-2 py-1 rounded text-sm"
              style={inputStyle}
              value={lengthType()}
              onChange={(e) => setLengthType(e.currentTarget.value)}
              title="Default length unit for display"
            >
              <option value="in">IN</option>
              <option value="mm">mm</option>
              <option value="cm">cm</option>
              <option value="m">m</option>
              <option value="ft">ft</option>
            </select>
          </div>

          <div class="flex items-center justify-between">
            <label class="text-sm" style={{ color: "var(--color-text)" }}>
              Temperature, C:
            </label>
            <input
              type="number"
              step="0.1"
              class="w-24 px-2 py-1 rounded text-sm text-right"
              style={inputStyle}
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
              style={inputStyle}
              value={humidity()}
              onInput={(e) => setHumidity(parseFloat(e.currentTarget.value) || 0)}
            />
          </div>

          <div class="flex items-center justify-between">
            <label class="text-sm" style={{ color: "var(--color-text)" }}>
              Use DIRECT optimizer (slow &amp; thorough):
            </label>
            <input
              type="checkbox"
              checked={useDirect()}
              onChange={(e) => setUseDirect(e.currentTarget.checked)}
              class="w-4 h-4"
            />
          </div>

          <div class="flex items-center justify-between">
            <label class="text-sm" style={{ color: "var(--color-text)" }}>
              Max Note Spectrum freq (multiplier):
            </label>
            <input
              type="number"
              step="0.01"
              min="1"
              max="100"
              class="w-24 px-2 py-1 rounded text-sm text-right"
              style={inputStyle}
              value={spectrumMult()}
              onInput={(e) => setSpectrumMult(parseFloat(e.currentTarget.value) || 3.17)}
              title="Upper frequency bound for Note Spectrum as a multiple of the note frequency (Java default: 3.17)"
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
              setUseDirectPref(useDirect());
              setLengthTypePref(lengthType());
              setSpectrumMultPref(spectrumMult());
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

/** Read DIRECT preference from localStorage. Default: true (matching Java). */
export function getUseDirect(): boolean {
  const stored = localStorage.getItem("wid_use_direct");
  if (stored === null) return true;
  return stored === "true";
}

/** Persist DIRECT preference. */
function setUseDirectPref(value: boolean) {
  localStorage.setItem("wid_use_direct", String(value));
}

/** Read Length Type preference. Default: "in" (matching Java's IN). */
export function getLengthType(): string {
  return localStorage.getItem("wid_length_type") ?? "in";
}

/** Persist Length Type preference. */
function setLengthTypePref(value: string) {
  localStorage.setItem("wid_length_type", value);
}

/** Read spectrum freq multiplier. Default: 3.17 (matching Java). */
export function getSpectrumMult(): number {
  const stored = localStorage.getItem("wid_spectrum_mult");
  if (stored === null) return 3.17;
  const v = parseFloat(stored);
  return isNaN(v) ? 3.17 : v;
}

/** Persist spectrum freq multiplier. */
function setSpectrumMultPref(value: number) {
  localStorage.setItem("wid_spectrum_mult", String(value));
}
