# Material Choice Decision Matrix

## Scope

This page helps agents answer broad material-selection questions such as wood
versus bamboo versus cane versus PVC versus resin versus PLA or PETG versus
printed sections. It is a routing and tradeoff matrix, not a recommendation
that any material is safe, stable, sustainable, beginner-ready, or release
approved.

Use it when the prompt asks for the best material, a humid-climate material,
beginner material, low-cost material, mouth-contact material, allergy-aware
material, shared-workshop material, outdoor material, sustainable material,
wet-out resistant material, or a material comparison across several options.

## Agent Summary

Material choice is a multi-claim decision. A material can be easy to machine
but poor for mouth contact; stable in humidity but unsupported by source
governance; acoustically adequate but hard to clean; printable but not
beginner-repeatable; traditional in a source but not a public safety approval.

The safe answer is to compare the claim surfaces, then route each positive
claim to its own evidence gate. Current encyclopedia coverage supports
conservative tradeoff reasoning. It does not prove any universal "best NAF
material" or public-ready material claim without exact source, product, lot,
finish, cleaning, wet-state, manufacturing, and release evidence.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/fcat.htm` | Local Flutopedia category route for material and maker-topic discovery. |
| `www.flutopedia.com/care.htm` | Local care and moisture-handling context. |
| `research/source-catalog.json` | Current-source caution for safety, legal, environmental, and material claims. |
| `docs/naf-encyclopedia/materials-species-sourcing.md` | Species, body stock, reclaimed material, and sourcing boundary. |
| `docs/naf-encyclopedia/synthetic-polymers-stabilized-woods.md` | Synthetic polymer, stabilized wood, resin, PVC, PLA, PETG, and composite routing. |
| `docs/naf-encyclopedia/materials-contact-safety.md` | Material, finish, adhesive, case, and mouth-contact safety boundary. |
| `docs/naf-encyclopedia/breath-path-material-release-packet.md` | Exact product and lot packet for breath-path material claims. |
| `docs/naf-encyclopedia/material-hygiene-cleaning-claim-matrix.md` | Cleaning, shared-use, odor, residue, and hygiene claim matrix. |
| `docs/naf-encyclopedia/wet-out-moisture-state.md` | Wet-state separation from dry tuning and safety claims. |
| `docs/naf-encyclopedia/additive-manufacturing-printed-nafs.md` | Printed NAF, internal plug, coupon, and open-release boundary. |
| `docs/independent-agent-research/explore/2026-06-04-alternative-body-material-comparison.md` | Broad alternative-body material comparison report. |
| `docs/independent-agent-research/explore/2026-06-04-wood-species-stability-selection.md` | Wood stability and species-selection report. |
| `docs/independent-agent-research/explore/2026-06-04-contact-material-safety.md` | Contact-material safety research plan. |
| `indexes/material-release-gate-fields.v0.json` | Machine-readable field families for material, climate, hygiene, and printing claims. |

## Decision Matrix

| Candidate class | Useful for asking | Immediate holds |
| --- | --- | --- |
| Solid wood | Bore stability, tooling, repairability, weight, aesthetics, local maker workflow | Species movement, cracking, finish, allergens, restricted species, ethical sourcing, mouth-contact packet |
| Bamboo or cane | Natural tube, low tooling, bore already present, lightweight prototypes | Node geometry, splitting, treatment history, contamination, tuning repeatability, finish and mouth-contact evidence |
| PVC or other pipe | Low-cost prototypes, classroom layout tests, dimensional availability | Exact formulation, additives, intended use, mouth-contact packet, heat/UV aging, public safety wording |
| PLA or PETG printed body | Rapid iteration, sectional design, internal features, reproducible geometry | Layer lines, support residue, porosity, material lot, cleaning compatibility, process-window drift, beginner repeatability |
| Resin printed parts | Fine detail, smoothness after post-process, small components | Resin chemistry, cure completion, residue, brittleness, cleanability, breath-path and skin-contact evidence |
| Stabilized wood or composites | Dimensional stability, hybrid aesthetics, machinability | Resin identity, vacuum/cure history, leachables, dust/shop hazards, finish compatibility, repair behavior |
| Reclaimed or donated stock | Cost and reuse goals | Unknown contamination, prior finish, smoke or mold exposure, provenance, quarantine, source record |
| Metal or decorative hardware | Rings, pins, weights, inlays, covers | Corrosion, galvanic contact, skin contact, choking/small-parts, finish migration, breath-path separation |

## Routing Rules

| User wants | First page | Gate before a positive claim |
| --- | --- | --- |
| Best material for a humid climate | `docs/naf-encyclopedia/materials-species-sourcing.md` | Product or species evidence, storage/climate record, and no "crack-proof" wording |
| Mouth-contact material comparison | `docs/naf-encyclopedia/breath-path-material-release-packet.md` | `make breath-path-material-release-hold` |
| Washable, sanitized, school-safe, or shared-use material | `docs/naf-encyclopedia/material-hygiene-cleaning-claim-matrix.md` | `make public-wording-claims`, `make inclusive-session-release-hold` |
| Printed material or sectional internal plug | `docs/naf-encyclopedia/additive-manufacturing-printed-nafs.md` | `make printed-plug-release-hold`, `make printed-plug-wet-state-join-hold` |
| Wet-out resistant material | `docs/naf-encyclopedia/wet-out-moisture-state.md` | `make wet-block-release-hold` |
| Ethical or sustainable material | `docs/naf-encyclopedia/materials-species-sourcing.md` | Current source review and source-governance release |
| Beginner material choice | This page, then the narrower safety, tooling, and release pages | Do not use beginner-ready wording without release evidence |

## Maker Workflow

1. Identify the actual decision: prototype, private instrument, classroom kit,
   public product, open-source print, repair, or research comparison.
2. Split the material question into stability, acoustics, tooling, cost,
   contact safety, cleaning, wet-state behavior, sustainability, source rights,
   and release wording.
3. Reject any universal "best material" answer unless the user has specified
   the key, bore, player, environment, contact path, cleaning protocol,
   manufacturing process, and release tier.
4. For a candidate material, write down the missing evidence before writing
   any positive claim.
5. If several materials remain plausible, propose a coupon or prototype test
   plan instead of ranking them from source prose alone.

## Data Fields

Material comparison records should include candidate class, exact source or
product id, lot, species or formulation, supplier, intended non-flute use,
contact path, finish, adhesive, bore process, moisture conditioning, climate
band, cleaning protocol, wet-state behavior, acoustic test plan, tooling risk,
repair path, sustainability/source-governance state, public wording state, and
release decision.

## Do Not Overclaim

Do not say one material is best for all NAFs.

Do not treat natural, traditional, food-safe, pipe-grade, medical-grade,
printed, stabilized, waterproof, or easy-to-clean wording as approval for
mouth contact, shared use, children, schools, public sale, or open release.

Do not convert a source-book material example into a current safety,
sustainability, or public-release claim.

Do not rank materials on acoustic tone alone when the user asked about
manufacturing, cleaning, allergy, beginner access, or public release.

## Open Gaps

Future output: `indexes/materials/material_choice_tradeoff_matrix.v0.json`.

Future output: `indexes/materials/material_coupon_comparison_records.jsonl`.

Future output: `indexes/materials/humid_climate_material_trials.jsonl`.

Needed work includes real material coupons, exact product/lot packets,
humidity-conditioning records, cleaning and residue tests, beginner
manufacturing trials, wet-state before-after acoustic data, and current
official-source review for legal, safety, and environmental claims.
