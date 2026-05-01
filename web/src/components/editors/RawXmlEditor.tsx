import { Show, createEffect, createSignal, on } from "solid-js";
import { sessionStore } from "../../stores/session";

export default function RawXmlEditor(props: {
  docId: number;
  refreshKey?: number;
  onApplied?: () => void;
}) {
  const [xml, setXml] = createSignal("");
  const [original, setOriginal] = createSignal("");
  const [loading, setLoading] = createSignal(true);
  const [message, setMessage] = createSignal<string | null>(null);
  const [localError, setLocalError] = createSignal<string | null>(null);

  createEffect(
    on(
      () => [props.docId, props.refreshKey] as const,
      async ([docId]) => {
        setLoading(true);
        setMessage(null);
        setLocalError(null);
        const exported = await sessionStore.exportXml(docId);
        setXml(exported ?? "");
        setOriginal(exported ?? "");
        setLoading(false);
      }
    )
  );

  const dirty = () => xml() !== original();

  async function apply() {
    setMessage(null);
    setLocalError(null);
    const ok = await sessionStore.replaceXml(props.docId, xml());
    if (ok) {
      setOriginal(xml());
      setMessage("XML applied.");
      props.onApplied?.();
    } else {
      setLocalError(sessionStore.error() ?? "XML could not be applied.");
    }
  }

  return (
    <Show when={!loading()} fallback={<p class="text-sm opacity-60">Loading XML...</p>}>
      <div class="flex min-h-[28rem] flex-col gap-3">
        <div class="flex items-center justify-between gap-3">
          <p class="text-xs" style={{ color: "var(--color-text-muted)" }}>
            Edit the raw WIDesigner XML. Apply validates that the root type still matches this document.
          </p>
          <div class="flex gap-2">
            <button
              class="ws-btn"
              disabled={!dirty()}
              onClick={() => {
                setXml(original());
                setMessage(null);
                setLocalError(null);
              }}
            >
              Revert
            </button>
            <button class="ws-btn ws-btn--signal" disabled={!dirty()} onClick={apply}>
              Apply XML
            </button>
          </div>
        </div>
        <div class="grid flex-1 grid-cols-[auto_1fr] overflow-hidden rounded border" style={{ "border-color": "var(--color-border)" }}>
          <pre class="select-none overflow-hidden border-r px-3 py-2 text-right ws-mono text-xs leading-5" style={{ background: "var(--color-surface-alt)", "border-color": "var(--color-border)", color: "var(--color-text-faint)" }}>
            {xml().split("\n").map((_, i) => i + 1).join("\n")}
          </pre>
          <textarea
            class="min-h-[28rem] resize-none overflow-auto border-0 bg-transparent px-3 py-2 ws-mono text-xs leading-5 outline-none"
            style={{ color: "var(--color-text)" }}
            spellcheck={false}
            value={xml()}
            onInput={(e) => setXml(e.currentTarget.value)}
          />
        </div>
        <Show when={message()}>
          <p class="text-xs" style={{ color: "var(--color-success)" }}>{message()}</p>
        </Show>
        <Show when={localError()}>
          <p class="rounded border px-3 py-2 text-xs" style={{ color: "var(--color-error)", background: "var(--color-error-soft)", "border-color": "var(--color-error)" }}>
            {localError()}
          </p>
        </Show>
      </div>
    </Show>
  );
}
