import { Show, For, createSignal, onMount } from "solid-js";
import { sessionStore } from "../../stores/session";

interface DocEntry {
  doc_id: number;
  name: string;
}

type WizardStep = "temperament" | "scale" | "tuning";

export default function WizardDialog(props: { onClose: () => void }) {
  const [step, setStep] = createSignal<WizardStep>("temperament");

  // Temperament state
  const [temperaments, setTemperaments] = createSignal<DocEntry[]>([]);
  const [tempSource, setTempSource] = createSignal<"ET12" | "JI12" | "loaded">("ET12");
  const [selectedTempId, setSelectedTempId] = createSignal<number | null>(null);

  // Scale state
  const [symbolSource, setSymbolSource] = createSignal<"sharps" | "flats">("sharps");
  const [refName, setRefName] = createSignal("A4");
  const [refFreq, setRefFreq] = createSignal(440.0);
  const [scaleName, setScaleName] = createSignal("Generated Scale");
  const [generatedScaleId, setGeneratedScaleId] = createSignal<number | null>(null);

  // Tuning state
  const [patterns, setPatterns] = createSignal<DocEntry[]>([]);
  const [selectedPatternId, setSelectedPatternId] = createSignal<number | null>(null);
  const [tuningName, setTuningName] = createSignal("Generated Tuning");

  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  onMount(async () => {
    const temps = await sessionStore.listTemperaments();
    setTemperaments(temps);
    const pats = await sessionStore.listFingeringPatterns();
    setPatterns(pats);
  });

  async function handleLoadFile(event: Event) {
    const input = event.target as HTMLInputElement;
    const files = input.files;
    if (!files) return;
    for (const file of Array.from(files)) {
      const xml = await file.text();
      await sessionStore.openXml(xml);
    }
    input.value = "";
    // Refresh lists
    setTemperaments(await sessionStore.listTemperaments());
    setPatterns(await sessionStore.listFingeringPatterns());
  }

  async function handleGenerateScale() {
    setLoading(true);
    setError(null);
    const opts: Parameters<typeof sessionStore.generateScale>[0] = {
      refName: refName(),
      refFrequency: refFreq(),
      scaleName: scaleName(),
      symbols: symbolSource() === "sharps" ? "scientific_sharps" : "scientific_flats",
    };
    if (tempSource() === "loaded" && selectedTempId() != null) {
      opts.temperamentId = selectedTempId()!;
    } else {
      opts.temperament = tempSource();
    }
    const result = await sessionStore.generateScale(opts);
    if (result) {
      setGeneratedScaleId(result.doc_id);
      setStep("tuning");
    } else {
      setError("Failed to generate scale");
    }
    setLoading(false);
  }

  async function handleGenerateTuning() {
    const scaleId = generatedScaleId();
    const patId = selectedPatternId();
    if (scaleId == null || patId == null) return;
    setLoading(true);
    setError(null);
    const result = await sessionStore.generateTuning(scaleId, patId, tuningName());
    if (result) {
      props.onClose();
    } else {
      setError("Failed to generate tuning");
    }
    setLoading(false);
  }

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
        class="rounded-lg shadow-lg p-6"
        style={{
          background: "var(--color-surface)",
          border: "1px solid var(--color-border)",
          width: "480px",
          "max-height": "90vh",
          "overflow-y": "auto",
        }}
      >
        <h2 class="text-lg font-semibold mb-1">Tuning Wizard</h2>
        <p class="text-xs mb-4" style={{ color: "var(--color-text-muted)" }}>
          Generate a tuning from temperament + scale + fingering pattern
        </p>

        {/* Step indicator */}
        <div class="flex gap-2 mb-4">
          {(["temperament", "scale", "tuning"] as WizardStep[]).map((s, i) => (
            <div
              class="flex-1 text-center py-1 rounded text-xs font-medium"
              style={{
                background: step() === s ? "var(--color-accent)" : "var(--color-surface-alt)",
                color: step() === s ? "white" : "var(--color-text-muted)",
              }}
            >
              {i + 1}. {s.charAt(0).toUpperCase() + s.slice(1)}
            </div>
          ))}
        </div>

        <Show when={error()}>
          <div class="text-sm mb-3 px-2 py-1 rounded" style={{ background: "#dc262620", color: "#ef4444" }}>
            {error()}
          </div>
        </Show>

        {/* Step 1: Temperament */}
        <Show when={step() === "temperament"}>
          <div class="flex flex-col gap-3">
            <label class="text-sm font-medium">Select Temperament</label>
            <div class="flex flex-col gap-2">
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="radio"
                  name="temp"
                  checked={tempSource() === "ET12"}
                  onChange={() => setTempSource("ET12")}
                />
                Equal Temperament (12-tone)
              </label>
              <label class="flex items-center gap-2 text-sm">
                <input
                  type="radio"
                  name="temp"
                  checked={tempSource() === "JI12"}
                  onChange={() => setTempSource("JI12")}
                />
                Just Intonation (12-tone)
              </label>
              <Show when={temperaments().length > 0}>
                <label class="flex items-center gap-2 text-sm">
                  <input
                    type="radio"
                    name="temp"
                    checked={tempSource() === "loaded"}
                    onChange={() => {
                      setTempSource("loaded");
                      if (!selectedTempId() && temperaments().length > 0)
                        setSelectedTempId(temperaments()[0].doc_id);
                    }}
                  />
                  Loaded:
                  <select
                    class="px-2 py-0.5 rounded text-sm border"
                    style={inputStyle}
                    value={selectedTempId() ?? ""}
                    onChange={(e) => {
                      setSelectedTempId(parseInt(e.currentTarget.value));
                      setTempSource("loaded");
                    }}
                  >
                    <For each={temperaments()}>
                      {(t) => <option value={t.doc_id}>{t.name}</option>}
                    </For>
                  </select>
                </label>
              </Show>
            </div>

            <label
              class="px-3 py-1 rounded text-xs cursor-pointer text-center"
              style={{ border: "1px solid var(--color-border)", color: "var(--color-text-muted)" }}
            >
              Load temperament/pattern from file...
              <input type="file" accept=".xml" multiple class="hidden" onChange={handleLoadFile} />
            </label>

            <div class="flex justify-end mt-2">
              <button
                class="px-4 py-1.5 rounded text-sm font-medium"
                style={{ background: "var(--color-accent)", color: "white" }}
                onClick={() => setStep("scale")}
              >
                Next
              </button>
            </div>
          </div>
        </Show>

        {/* Step 2: Scale */}
        <Show when={step() === "scale"}>
          <div class="flex flex-col gap-3">
            <label class="text-sm font-medium">Scale Parameters</label>

            <div class="flex items-center justify-between">
              <label class="text-sm">Note symbols:</label>
              <select
                class="px-2 py-1 rounded text-sm border"
                style={inputStyle}
                value={symbolSource()}
                onChange={(e) => setSymbolSource(e.currentTarget.value as "sharps" | "flats")}
              >
                <option value="sharps">Sharps (C, C#, D, ...)</option>
                <option value="flats">Flats (C, Db, D, ...)</option>
              </select>
            </div>

            <div class="flex items-center justify-between">
              <label class="text-sm">Reference note:</label>
              <input
                type="text"
                class="w-20 px-2 py-1 rounded text-sm text-right"
                style={inputStyle}
                value={refName()}
                onInput={(e) => setRefName(e.currentTarget.value)}
              />
            </div>

            <div class="flex items-center justify-between">
              <label class="text-sm">Reference freq (Hz):</label>
              <input
                type="number"
                step="0.1"
                class="w-24 px-2 py-1 rounded text-sm text-right"
                style={inputStyle}
                value={refFreq()}
                onInput={(e) => setRefFreq(parseFloat(e.currentTarget.value) || 440)}
              />
            </div>

            <div class="flex items-center justify-between">
              <label class="text-sm">Scale name:</label>
              <input
                type="text"
                class="w-48 px-2 py-1 rounded text-sm"
                style={inputStyle}
                value={scaleName()}
                onInput={(e) => setScaleName(e.currentTarget.value)}
              />
            </div>

            <div class="flex justify-between mt-2">
              <button
                class="px-4 py-1.5 rounded text-sm"
                style={{ color: "var(--color-text-muted)" }}
                onClick={() => setStep("temperament")}
              >
                Back
              </button>
              <button
                class="px-4 py-1.5 rounded text-sm font-medium"
                style={{ background: "var(--color-accent)", color: "white" }}
                disabled={loading()}
                onClick={handleGenerateScale}
              >
                {loading() ? "Generating..." : "Generate Scale"}
              </button>
            </div>
          </div>
        </Show>

        {/* Step 3: Tuning */}
        <Show when={step() === "tuning"}>
          <div class="flex flex-col gap-3">
            <label class="text-sm font-medium">Generate Tuning</label>

            <Show
              when={patterns().length > 0}
              fallback={
                <div class="text-sm" style={{ color: "var(--color-text-muted)" }}>
                  No fingering patterns loaded. Load a pattern file first.
                  <label
                    class="block mt-2 px-3 py-1 rounded text-xs cursor-pointer text-center"
                    style={{ border: "1px solid var(--color-border)" }}
                  >
                    Load pattern from file...
                    <input type="file" accept=".xml" class="hidden" onChange={handleLoadFile} />
                  </label>
                </div>
              }
            >
              <div class="flex items-center justify-between">
                <label class="text-sm">Fingering pattern:</label>
                <select
                  class="px-2 py-1 rounded text-sm border"
                  style={inputStyle}
                  value={selectedPatternId() ?? ""}
                  onChange={(e) => setSelectedPatternId(parseInt(e.currentTarget.value))}
                >
                  <For each={patterns()}>
                    {(p) => <option value={p.doc_id}>{p.name}</option>}
                  </For>
                </select>
              </div>
            </Show>

            <div class="flex items-center justify-between">
              <label class="text-sm">Tuning name:</label>
              <input
                type="text"
                class="w-48 px-2 py-1 rounded text-sm"
                style={inputStyle}
                value={tuningName()}
                onInput={(e) => setTuningName(e.currentTarget.value)}
              />
            </div>

            <div class="flex justify-between mt-2">
              <button
                class="px-4 py-1.5 rounded text-sm"
                style={{ color: "var(--color-text-muted)" }}
                onClick={() => setStep("scale")}
              >
                Back
              </button>
              <button
                class="px-4 py-1.5 rounded text-sm font-medium"
                style={{ background: "var(--color-accent)", color: "white" }}
                disabled={loading() || selectedPatternId() == null || generatedScaleId() == null}
                onClick={handleGenerateTuning}
              >
                {loading() ? "Generating..." : "Generate Tuning"}
              </button>
            </div>
          </div>
        </Show>

        {/* Cancel */}
        <div class="flex justify-end mt-4 pt-3 border-t" style={{ "border-color": "var(--color-border)" }}>
          <button
            class="px-3 py-1 rounded text-sm"
            style={{ color: "var(--color-text-muted)" }}
            onClick={props.onClose}
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
