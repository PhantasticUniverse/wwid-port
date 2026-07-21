import { Marked } from "marked";
import { articleBySlug } from "../../reference/articles";

function escapeHtml(text: string): string {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

// Articles cite sources as backticked plain-text paths. Two kinds get linkified:
// Flutopedia paths become external links (the mirror itself is never bundled),
// and cross-references to bundled articles become in-dialog navigation anchors.
// Everything else stays inert code text.
const marked = new Marked({
  gfm: true,
  renderer: {
    codespan({ text }: { text: string }): string | false {
      const flutopedia = text.match(/^www\.flutopedia\.com\/(\S+)$/);
      if (flutopedia) {
        return `<a href="https://www.flutopedia.com/${escapeHtml(flutopedia[1])}" target="_blank" rel="noopener">${escapeHtml(text)}</a>`;
      }
      const crossRef = text.match(/^docs\/naf-encyclopedia\/([a-z0-9-]+)\.md$/);
      if (crossRef && articleBySlug(crossRef[1])) {
        return `<a href="#" data-ref-slug="${crossRef[1]}">${escapeHtml(text)}</a>`;
      }
      return false;
    },
  },
});

export function renderMarkdown(md: string): string {
  return marked.parse(md) as string;
}
