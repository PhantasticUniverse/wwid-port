# Terminology, Aliases, And Glossary

## Scope

This page covers agent-facing terminology: common aliases, historical labels,
maker vocabulary, source-specific terms, and terms that should never be
silently treated as exact equivalents.

## Agent Summary

NAF language is full of near-synonyms that are useful in conversation but
dangerous in engineering records. SAC, breath chamber, slow air chamber, true
sound hole, window, TSH, fipple, flue, edge, bird, block, saddle, fetish,
finger hole, tone hole, direction hole, tuning hole, Plains, Woodlands,
Native American-style, and Native American flute can all carry different
source, mechanism, cultural, or release implications.

Additive-manufacturing and response-surface terms now need the same care:
FDM/FFF, slicer, support interface, bridge, overhang, bore-down, plug insert,
section joint, PLA, PETG, resin, wetout, windway, airway, ramp, pressure curve,
blow curve, artificial mouth, mutual loading, hand shadowing, and register
break are useful search terms, but they are not automatically evidence,
acceptance criteria, or public wording.

Agents should preserve source wording, map aliases explicitly, and state which
meaning is being used. A search alias is not a public label, and a local maker
term is not automatically a taxonomy or cultural-authority term.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/glossary.htm` | Local glossary source. |
| `www.flutopedia.com/anatomy.htm` | Anatomy vocabulary and diagrams. |
| `www.flutopedia.com/flute_classification.htm` | Classification and term-disagreement context. |
| `www.flutopedia.com/fipple.htm` | Fipple/TSH measurement vocabulary. |
| `docs/naf-encyclopedia/instrument-taxonomy-organology.md` | Mechanism and classification boundary. |
| `docs/naf-encyclopedia/flue-tsh-fipple-voicing.md` | Sound-mechanism vocabulary. |
| `docs/naf-encyclopedia/fingering-systems.md` | Fingering-system and hole-order vocabulary. |
| `docs/naf-encyclopedia/additive-manufacturing-printed-nafs.md` | Printed-section, slicer, support, plug/SAC, and open-release vocabulary. |
| `docs/independent-agent-research/explore/2026-06-04-naf-taxonomy-boundary-standard.md` | Taxonomy boundary report. |
| `docs/independent-agent-research/extend/2026-06-04-sensitive-name-search-alias-release-contract.md` | Sensitive name and search-alias release contract. |

## Maker Workflow

1. Preserve the term exactly as used by the source, maker, catalog, or user.
2. Map the term to a canonical internal field only after recording source,
   meaning, confidence, and context.
3. Distinguish search aliases from public display labels.
4. Keep mechanism terms separate from cultural, legal, market, and authenticity
   terms.
5. For hole order, record serialized order, physical order, player-facing
   diagram order, and CAD axis separately.
6. Route contested or sensitive names through name-governance review.
7. For printed designs, preserve slicer/manufacturing terms separately from
   acoustic terms: a bridge is a print span, while a pressure curve is measured
   acoustic behavior.

## Data Fields

Useful fields include term, alias, canonical field, source id, source path,
source wording, language/context, public display permission, search-only flag,
claim type, mechanism meaning, cultural meaning, legal/market meaning,
confidence, deprecated/offensive flag, replacement wording, and review state.

Additive and acoustic-response alias records should also include process
family, material family, measurement state, claim gate, and whether the term is
allowed in public instructions or only in internal search.

## Do Not Overclaim

Do not treat fipple factor as physical fipple geometry.

Do not merge TSH, window, edge, flue, block, nest, and bird terms without
recording the source's meaning.

Do not infer cultural or legal authority from a mechanism alias.

Do not use sensitive names or community labels as public aliases just because
they help search.

Do not map `ramp`, `bevel`, `edge`, `splitting edge`, and `fipple factor` as
the same object. Preserve which feature the source means.

Do not map `wet out`, `wetout`, `condensation`, `flue blockage`, and `wet-state
failure` as identical outcomes without a measured state label.

Do not map `food safe`, `FDA compliant`, `biocompatible`, `medical grade`,
`dental resin`, `BPA-free`, and `non-toxic` into approved mouthpath status.
Route them to `docs/naf-encyclopedia/breath-path-material-release-packet.md`
and refusal unless a reviewed packet says otherwise.

Do not map `therapy`, `therapeutic`, `healing`, `medical`, `clinically
validated`, `asthma`, `COPD`, `PTSD`, `anxiety`, `depression`, or
`rehabilitation` into supported health claims. Route them to
`docs/naf-encyclopedia/medical-therapy-claim-boundaries.md`.

Do not map `patent safe`, `expired patent`, `prior art`, `freedom to operate`,
`claim chart`, or `design around` into legal clearance. Route them to
`docs/naf-encyclopedia/patent-prior-art-maker-risk.md`.

## Open Gaps

Materialized routing artifact: `indexes/naf-encyclopedia/page_aliases.v0.json`.
Materialized validation gates: `make page-aliases-strict` for routing and
`make public-wording-claims` for approved public wording packages. Needed work
still includes canonical page redirects, source-specific term maps,
hole-order convention fixtures, and coverage checks that flag pages missing
aliases or canonical claim-type labels.

High-priority aliases for the next index pass include FDM, FFF, slicer,
support interface, bridge, overhang, bore-down, plug insert, section joint,
PLA, PETG, resin, soluble support, support scar, ramp, windway, airway,
surface energy, contact angle, pressure curve, blow curve, breath-to-pitch,
artificial mouth, mutual loading, hand shadowing, low NAF, register break,
stable warble, weak fundamental, projection, support-interface variants,
fingering-system aliases, and WIDesigner field names.
