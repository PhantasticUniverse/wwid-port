#!/usr/bin/env node
// Imports the curated reference articles and the Wood Wind F#4 tuning from the
// local-flute-encyclopedia repo. Only content authored by the repo owner may be
// copied: articles must contain no embedded images or hyperlinks (Flutopedia and
// other sources are cited as plain backticked paths, linkified at render time).
// Re-run after upstream article edits; fails closed if the rights guard trips.

import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const REPO_ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const SOURCE_REPO = resolve(REPO_ROOT, "../local-flute-encyclopedia");
const ARTICLE_SRC = join(SOURCE_REPO, "docs/naf-encyclopedia");
const ARTICLE_DEST = join(REPO_ROOT, "web/src/reference/articles");
const TUNING_SRC = join(
  SOURCE_REPO,
  "sources/user_wood_wind_chromatic_tuning_xml/fsharp4-et-6-hole-naf-chromatic-wid.xml",
);
const TUNING_DEST = join(
  REPO_ROOT,
  "web/public/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_WoodWind_tuning.xml",
);

const ARTICLE_SLUGS = [
  "native-american-flute-overview",
  "instrument-taxonomy-organology",
  "acoustics-design-evidence-map",
  "terminology-aliases-glossary",
  "bore-sac-body-geometry",
  "flue-tsh-fipple-voicing",
  "tone-holes-undercut-direction-holes",
  "unit-coordinate-drift",
  "breath-pressure-environment-tuning",
  "breath-pressure-curve-model",
  "fingering-systems",
  "widesigner-tuning-xml",
  "warble-register-timbre",
  "warble-geometry-thresholds",
  "wet-out-moisture-state",
  "design-calculators-model-cards",
  "widesigner-evidence-boundaries",
  "materials-species-sourcing",
  "material-choice-decision-matrix",
  "making-process-overview",
];

// Any embedded image or hyperlink means the file is no longer plain authored
// text and must be reviewed by hand before bundling.
const RIGHTS_GUARDS = [/!\[/, /<img/i, /\]\(/, /<a\s/i];

if (!existsSync(ARTICLE_SRC)) {
  console.error(`Source not found: ${ARTICLE_SRC}`);
  process.exit(1);
}

const sourceCommit = execFileSync("git", ["-C", SOURCE_REPO, "rev-parse", "HEAD"], {
  encoding: "utf8",
}).trim();

mkdirSync(ARTICLE_DEST, { recursive: true });

const imported = [];
for (const slug of ARTICLE_SLUGS) {
  const srcPath = join(ARTICLE_SRC, `${slug}.md`);
  const text = readFileSync(srcPath, "utf8");
  for (const guard of RIGHTS_GUARDS) {
    if (guard.test(text)) {
      console.error(`Rights guard ${guard} matched in ${srcPath} — not copying. Review by hand.`);
      process.exit(1);
    }
  }
  writeFileSync(join(ARTICLE_DEST, `${slug}.md`), text);
  const title = text.match(/^# (.+)$/m)?.[1] ?? slug;
  imported.push({ slug, title });
  console.log(`article  ${slug}.md`);
}

// The tuning is the user's own file; the <name> is adjusted so it is
// distinguishable from the stock F#4 sample in the document list.
const tuningXml = readFileSync(TUNING_SRC, "utf8").replace(
  "<name>F#4 ET 6-hole NAF chromatic tuning</name>",
  "<name>F#4 ET 6-hole NAF chromatic tuning (Wood Wind)</name>",
);
if (!tuningXml.includes("(Wood Wind)")) {
  console.error("Tuning <name> rewrite did not apply — check the source XML.");
  process.exit(1);
}
writeFileSync(TUNING_DEST, tuningXml);
console.log(`tuning   ${TUNING_DEST}`);

const provenance = `# Reference Content Provenance

- Source repository: \`local-flute-encyclopedia\` (local sibling checkout)
- Source commit: \`${sourceCommit}\`
- Imported: ${new Date().toISOString().slice(0, 10)} by \`tools/import-reference.mjs\`

All articles under \`articles/\` were authored by the repository owner in the
local-flute-encyclopedia project (\`docs/naf-encyclopedia/\`). They are copied
verbatim. Flutopedia (flutopedia.com) is cited by URL only; no Flutopedia
content (text, images, audio) is included. Citation paths in Evidence Map
tables refer to files in the source repository and are rendered as plain text
or external links, never bundled.

The sample tuning
\`web/public/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_WoodWind_tuning.xml\`
is the owner's own WIDesigner tuning file
(\`sources/user_wood_wind_chromatic_tuning_xml/fsharp4-et-6-hole-naf-chromatic-wid.xml\`),
with the \`<name>\` suffixed "(Wood Wind)" for distinguishability. Its alternate
G5 (closed) fingering carries optimizationWeight 0, excluding it from the
residual norm per WIDesigner semantics.

## Imported articles

${imported.map((a) => `- \`${a.slug}.md\` — ${a.title}`).join("\n")}
`;
writeFileSync(join(REPO_ROOT, "web/src/reference/PROVENANCE.md"), provenance);
console.log(`wrote    web/src/reference/PROVENANCE.md (${imported.length} articles)`);
