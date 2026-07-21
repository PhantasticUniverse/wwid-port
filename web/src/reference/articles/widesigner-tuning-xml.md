# WIDesigner Tuning XML

## Scope

This page covers the local user-provided WIDesigner tuning XML, the normalized
JSON artifact derived from it, the inferred F#4/A4=440 target set, duplicate G5
rows, optimization weights, and caveats around physical hole order and the
related private CAD model.

## Agent Summary

The XML is the current captured provisional artifact for the alternate Wood
Wind / Edward Kort-style fingering rows. It is not measured tuning, not a calibrated
fipple-factor source, not proof of the related CAD model's final key, and not a
general Flutopedia chart. The all-closed first row targets F#4, the A4 row is
440 Hz, and the title says `F#4 ET 6-hole NAF chromatic tuning`; those support a
provisional F#4 equal-tempered interpretation. Preserve row order and the
serialized open-hole order until physical mapping is verified.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `sources/user_wood_wind_chromatic_tuning_xml/fsharp4-et-6-hole-naf-chromatic-wid.xml` | Raw user-provided WIDesigner tuning XML. |
| `sources/user_wood_wind_chromatic_tuning_xml.json` | Source manifest and capture notes. |
| `docs/source-intake/user_wood_wind_chromatic_tuning_xml.md` | Intake summary, hash notes, semantics, and prohibited uses. |
| `indexes/fingering-schemes/user_wood_wind_fsharp4_chromatic_tuning.v0.json` | Normalized machine-readable rows and validation caveats. |
| `indexes/source-governance/user_wood_wind_fsharp4_chromatic_tuning_release_card.v0.json` | Artifact-level release card preserving private, provisional, do-not-train limits. |
| `sources/widesigner_intro_video_fingering_scheme.json` | Lead-only video manifest. |
| `docs/independent-agent-research/extend/2026-06-04-alternate-fingering-scheme-cad-intake-contract.md` | Full XML/CAD/video intake contract. |
| `widesigner-2.6-docs/docs/data-and-ui-reference.html` | WIDesigner data and UI meanings. |
| `widesigner-2.6-docs/docs/optimization-reference.html` | WIDesigner optimization-weight and residual-routing lead. |
| `indexes/widesigner/golden_fixtures/weighted_residual_optimizer_route.json` | Source/release-bound fixture showing WIDesigner excludes nonpositive optimization weights from the residual norm; not validation of this private XML's physical role. |

## Maker Workflow

1. Normalize the XML with `python3 tools/archive.py widesigner-tuning`.
2. Preserve the source row sequence, note labels, target frequencies, and raw
   weights.
3. Mark nonzero weights as rough/unverified unless the user or source confirms
   them.
4. Treat zero-weight rows as excluded from WIDesigner's weighted residual norm,
   while keeping this private XML's duplicate-row role and physical mapping
   provisional.
5. Compare rows to Flutopedia only after naming the chart family.
6. Link the XML to CAD as an associated target source, not as proof of the CAD
   model's measured or final tuning.

## Data Fields

Minimum fields include tuning name, XML namespace, number of holes, row
sequence, note name, target frequency, pitch standard inference, temperament
inference, open/closed pattern, serialized hole order, physical hole-order
confidence, optimization weight, weight confidence, duplicate target flag,
source manifest, normalized artifact id, and validation state.

## Do Not Overclaim

Do not deduplicate the two G5 rows. `G5 (open)` is `OOOOOO` with weight `1`;
`G5 (closed)` is `XOXXXX` with weight `0`. They share the same target
frequency, but they carry different fingering and optimization roles.

Do not describe the XML as a full low-register chromatic run. It starts at F#4,
jumps to A4, then runs chromatically from A4 to A5 with duplicate G5 options.

Do not assert physical mouth-to-foot hole order from serialized XML order.

Do not treat the related Fusion model as definitively F#4 solely because this
XML implies F#4 targets.

Do not promote the normalized JSON rows to a public tuning dataset, embedding,
or training input. The generated artifact must preserve its release-card pointer
and pass `make derived-artifact-release`.

## Open Gaps

Needed work includes timestamped video notes, physical hole-order verification,
runtime replay for the exact private XML/design if it is ever promoted,
measured pitch and pressure data for the related design, and a broader schema
that can compare WIDesigner XML rows to Flutopedia chart families without
collapsing them.
