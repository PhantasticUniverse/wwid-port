import { For, createMemo } from "solid-js";
import type { SampleBundle, SampleStudy } from "../../data/sampleBundles";
import { SAMPLE_BUNDLES } from "../../data/sampleBundles";

export default function SampleLoaderDialog(props: {
  study: string;
  onLoad: (bundle: SampleBundle) => void;
  onClose: () => void;
}) {
  const grouped = createMemo(() => {
    const order: SampleStudy[] = ["NAF", "Whistle", "Flute", "Reed"];
    return order.map((study) => ({
      study,
      bundles: SAMPLE_BUNDLES.filter((bundle) => bundle.study === study),
    }));
  });

  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: "rgba(31, 28, 24, 0.32)" }}
      onClick={(e) => {
        if (e.target === e.currentTarget) props.onClose();
      }}
    >
      <section class="ws-panel max-h-[86vh] w-full max-w-4xl overflow-hidden">
        <header class="flex items-start justify-between gap-4 border-b p-5" style={{ "border-color": "var(--color-border)" }}>
          <div>
            <div class="ws-eyebrow mb-1">Sample library</div>
            <h2 class="ws-serif text-2xl font-semibold">Load a complete study bundle</h2>
            <p class="mt-1 max-w-2xl text-sm" style={{ color: "var(--color-text-muted)" }}>
              Pick a bundled instrument, tuning, and matching constraints set. Files are loaded through the same XML path as user documents.
            </p>
          </div>
          <button class="ws-btn ws-btn--ghost" onClick={props.onClose}>Close</button>
        </header>

        <div class="max-h-[64vh] overflow-auto p-5">
          <For each={grouped()}>
            {(group) => (
              <section class="mb-5 last:mb-0">
                <div class="ws-eyebrow mb-2">{group.study} study</div>
                <div class="grid gap-3 md:grid-cols-2">
                  <For each={group.bundles}>
                    {(bundle) => (
                      <article
                        class="rounded border p-4"
                        style={{
                          background: bundle.study === props.study ? "var(--color-accent-soft)" : "var(--color-surface)",
                          "border-color": bundle.study === props.study ? "var(--color-accent)" : "var(--color-border)",
                        }}
                      >
                        <div class="flex items-start justify-between gap-3">
                          <div>
                            <h3 class="text-base font-semibold">{bundle.title}</h3>
                            <p class="mt-1 text-xs leading-relaxed" style={{ color: "var(--color-text-muted)" }}>
                              {bundle.description}
                            </p>
                          </div>
                          <button class="ws-btn ws-btn--signal" onClick={() => props.onLoad(bundle)}>
                            Load
                          </button>
                        </div>
                        <ul class="mt-3 space-y-1">
                          <For each={bundle.files}>
                            {(file) => (
                              <li class="flex items-center gap-2 text-xs">
                                <span class="ws-mono rounded px-1.5 py-0.5" style={{ background: "var(--color-surface-alt)", color: "var(--color-text-muted)" }}>
                                  {file.kind}
                                </span>
                                <span>{file.label}</span>
                              </li>
                            )}
                          </For>
                        </ul>
                      </article>
                    )}
                  </For>
                </div>
              </section>
            )}
          </For>
        </div>
      </section>
    </div>
  );
}
