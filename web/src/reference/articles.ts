export type RefCategory =
  | "Overview"
  | "Geometry"
  | "Tuning & Breath"
  | "Models & Optimization"
  | "Materials & Making";

export interface RefArticle {
  slug: string;
  title: string;
  category: RefCategory;
}

export const REF_CATEGORIES: RefCategory[] = [
  "Overview",
  "Geometry",
  "Tuning & Breath",
  "Models & Optimization",
  "Materials & Making",
];

// Titles mirror each article's H1 so the nav renders without loading content.
export const REFERENCE_ARTICLES: RefArticle[] = [
  { slug: "native-american-flute-overview", title: "Native American Flute Overview", category: "Overview" },
  { slug: "instrument-taxonomy-organology", title: "Instrument Taxonomy And Organology", category: "Overview" },
  { slug: "acoustics-design-evidence-map", title: "Acoustics And Design Evidence Map", category: "Overview" },
  { slug: "terminology-aliases-glossary", title: "Terminology, Aliases, And Glossary", category: "Overview" },
  { slug: "bore-sac-body-geometry", title: "Bore, SAC, And Body Geometry", category: "Geometry" },
  { slug: "flue-tsh-fipple-voicing", title: "Flue, TSH, Fipple, And Voicing", category: "Geometry" },
  { slug: "tone-holes-undercut-direction-holes", title: "Tone Holes, Undercut, And Direction Holes", category: "Geometry" },
  { slug: "unit-coordinate-drift", title: "Unit, Coordinate, And Drift Gates", category: "Geometry" },
  { slug: "breath-pressure-environment-tuning", title: "Breath Pressure, Environment, And Tuning", category: "Tuning & Breath" },
  { slug: "breath-pressure-curve-model", title: "Breath Pressure Curve Model", category: "Tuning & Breath" },
  { slug: "fingering-systems", title: "Fingering Systems And Tuning Targets", category: "Tuning & Breath" },
  { slug: "widesigner-tuning-xml", title: "WIDesigner Tuning XML", category: "Tuning & Breath" },
  { slug: "warble-register-timbre", title: "Warble, Register, And Timbre", category: "Tuning & Breath" },
  { slug: "warble-geometry-thresholds", title: "Warble Geometry Thresholds", category: "Tuning & Breath" },
  { slug: "wet-out-moisture-state", title: "Wet-Out And Moisture State", category: "Tuning & Breath" },
  { slug: "design-calculators-model-cards", title: "Design Calculators And Model Cards", category: "Models & Optimization" },
  { slug: "widesigner-evidence-boundaries", title: "WIDesigner Evidence Boundaries", category: "Models & Optimization" },
  { slug: "materials-species-sourcing", title: "Materials, Species, And Sourcing", category: "Materials & Making" },
  { slug: "material-choice-decision-matrix", title: "Material Choice Decision Matrix", category: "Materials & Making" },
  { slug: "making-process-overview", title: "Making Process Overview", category: "Materials & Making" },
];

export const DEFAULT_ARTICLE_SLUG = "native-american-flute-overview";

export function articleBySlug(slug: string): RefArticle | undefined {
  return REFERENCE_ARTICLES.find((a) => a.slug === slug);
}

const loaders = import.meta.glob("./articles/*.md", { query: "?raw", import: "default" });

export async function loadArticle(slug: string): Promise<string> {
  const loader = loaders[`./articles/${slug}.md`];
  if (!loader) throw new Error(`Unknown reference article: ${slug}`);
  return (await loader()) as string;
}
