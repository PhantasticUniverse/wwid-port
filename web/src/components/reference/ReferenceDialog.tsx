import { For, Show, createResource, onCleanup, onMount } from "solid-js";
import { REFERENCE_ARTICLES, REF_CATEGORIES, loadArticle } from "../../reference/articles";
import { closeReference, referenceSlug, showReferenceArticle } from "../../stores/reference";
import { renderMarkdown } from "./renderMarkdown";

export default function ReferenceDialog() {
  let contentPane: HTMLDivElement | undefined;

  const [html] = createResource(referenceSlug, async (slug) => {
    const md = await loadArticle(slug);
    return renderMarkdown(md);
  });

  onMount(() => {
    const onKey = (e: KeyboardEvent) => { if (e.key === "Escape") closeReference(); };
    document.addEventListener("keydown", onKey);
    onCleanup(() => document.removeEventListener("keydown", onKey));
  });

  function handleContentClick(e: MouseEvent) {
    const anchor = (e.target as HTMLElement).closest("a[data-ref-slug]");
    if (anchor) {
      e.preventDefault();
      const slug = anchor.getAttribute("data-ref-slug");
      if (slug) {
        showReferenceArticle(slug);
        contentPane?.scrollTo({ top: 0 });
      }
    }
  }

  return (
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-6"
      style={{ background: "rgba(31, 28, 24, 0.32)" }}
      onClick={(e) => {
        if (e.target === e.currentTarget) closeReference();
      }}
    >
      <section class="ws-panel flex max-h-[86vh] w-full max-w-5xl flex-col overflow-hidden">
        <header class="flex items-start justify-between gap-4 border-b p-5" style={{ "border-color": "var(--color-border)" }}>
          <div>
            <div class="ws-eyebrow mb-1">Reference</div>
            <h2 class="ws-serif text-2xl font-semibold">Native American flute design notes</h2>
            <p class="mt-1 max-w-2xl text-sm" style={{ color: "var(--color-text-muted)" }}>
              Maker-oriented articles on geometry, voicing, tuning, and how far the acoustic model can be trusted.
            </p>
          </div>
          <button class="ws-btn ws-btn--ghost" onClick={closeReference}>Close</button>
        </header>

        <div class="flex min-h-0 flex-1">
          <nav class="w-60 shrink-0 overflow-y-auto border-r p-4" style={{ "border-color": "var(--color-border)" }}>
            <For each={REF_CATEGORIES}>
              {(category) => (
                <section class="mb-4 last:mb-0">
                  <div class="ws-eyebrow mb-1.5">{category}</div>
                  <ul class="space-y-0.5">
                    <For each={REFERENCE_ARTICLES.filter((a) => a.category === category)}>
                      {(article) => (
                        <li>
                          <button
                            class="w-full rounded px-2 py-1 text-left text-sm transition-colors"
                            style={{
                              background: article.slug === referenceSlug() ? "var(--color-accent-soft)" : "transparent",
                              color: article.slug === referenceSlug() ? "var(--color-text)" : "var(--color-text-muted)",
                            }}
                            onClick={() => {
                              showReferenceArticle(article.slug);
                              contentPane?.scrollTo({ top: 0 });
                            }}
                          >
                            {article.title}
                          </button>
                        </li>
                      )}
                    </For>
                  </ul>
                </section>
              )}
            </For>
          </nav>

          <div ref={contentPane} class="min-w-0 flex-1 overflow-auto p-6" onClick={handleContentClick}>
            <Show
              when={!html.loading}
              fallback={<p class="text-sm" style={{ color: "var(--color-text-muted)" }}>Loading article…</p>}
            >
              <Show
                when={!html.error}
                fallback={<p class="text-sm" style={{ color: "var(--color-error)" }}>Could not load article.</p>}
              >
                <div class="ws-prose" innerHTML={html()} />
              </Show>
            </Show>
          </div>
        </div>

        <footer class="border-t px-5 py-2.5 text-xs" style={{ "border-color": "var(--color-border)", color: "var(--color-text-muted)" }}>
          Articles written by the app author for the local NAF encyclopedia project. Flutopedia (flutopedia.com) is cited by URL only; no Flutopedia content is included.
        </footer>
      </section>
    </div>
  );
}
