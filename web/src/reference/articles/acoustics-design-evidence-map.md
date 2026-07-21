# Acoustics And Design Evidence Map

## Scope

This page explains how agents should judge evidence strength for NAF design
claims. It covers Flutopedia source snapshots, derived WIDesigner documentation,
normalized local artifacts, agent research reports, measured validation needs,
and publication boundaries.

## Agent Summary

The current encyclopedia is strong at identifying design variables and research
gaps, but it is not yet a golden predictive design system. Treat Flutopedia as
a valuable source snapshot, WIDesigner docs as a derived but high-value
algorithmic reference, and the agent reports as engineering syntheses. Treat
release claims, safety claims, tuning claims, and cultural claims as governed
claims that need explicit evidence class and review state.

The highest-risk pattern is over-upgrading evidence: turning a calculator
starting point into proof, a derived WIDesigner explanation into upstream source
evidence, a report proposal into measured validation, or a private CAD/XML
intake into final design truth.

## Evidence Map

Core source and research controls:

| Evidence path | Role |
| --- | --- |
| `research/source-catalog.json` | Evidence spine for sources, limits, topics, and live-source cautions. |
| `docs/independent-agent-research/README.md` | Research corpus status, gates, and promotion direction into encyclopedia pages. |
| `docs/independent-agent-research/2026-06-04-naf-acoustics-design-research-map.md` | High-level acoustics and design map from the Q&A corpus. |
| `docs/independent-agent-research/2026-06-04-saturation-summary.md` | Saturation state and remaining frontier after the large explore rounds. |
| `docs/source-intake-protocol.md` | Gate between incoming material and promoted evidence. |
| `widesigner-2.6-docs/PROVENANCE.md` | Boundary for WIDesigner evidence roots and re-verification. |
| `widesigner-2.6-docs/docs/evidence-matrix.html` | WIDesigner high-risk claim ownership and review trail. |
| `sources/widesigner_2_6_upstream_public_repo.json` | Separate upstream WIDesigner source/release locator; not the authored docs bundle and not a committed upstream payload. |
| `indexes/widesigner/widesigner_2_6_source_manifest.v0.json` | Local ignored evidence-root state, source/release hashes, and WIDesigner fixture backlog. |

Design source snapshots and calculators:

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/dimensions.htm` | NAF geometry vocabulary and maker-source context. |
| `www.flutopedia.com/acoustic_length.htm` | Acoustic length framing and correction context. |
| `www.flutopedia.com/naflutomat.htm` | Calculator lead and starting-point warning. |
| `www.flutopedia.com/breath_pressure.htm` | Breath-pressure sample evidence, not a universal player envelope. |

## Maker Workflow

1. Classify the claim: geometry, tuning, model behavior, measured performance,
   material safety, source rights, cultural context, or release decision.
2. Find the strongest local evidence class available.
3. If the evidence is a source snapshot or derived documentation, use it to
   frame the question, not to certify the result.
4. If the claim drives machining, release, public claims, or safety decisions,
   require measured validation, current source review, or qualified review.
5. Record the remaining uncertainty in the relevant page or artifact.

## Data Fields

Useful cross-page fields include source id, evidence class, local path,
canonical URL, capture date, rights status, CARE sensitivity, claim type,
claim owner, validation state, measured fixture id, and release decision.

## Do Not Overclaim

Do not say the corpus has "solved" NAF design. It has mapped the design space,
identified many variables, created validation contracts, and captured key
source leads. It still needs measured corpora and regression fixtures before it
can make strong predictive statements.

Do not cite `widesigner-2.6-docs/` as bundled WIDesigner software. It is an
authored deep-study documentation bundle with provenance instructions; the
actual upstream source is Edward Kort's `github.com/edwardkort/WWIDesigner`
repository, cataloged separately as `widesigner_2_6_upstream_public_repo`.
Ignored local source/release roots can support named fixtures, but they remain
outside ordinary commits.

Do not use private source payloads, recordings, collection data, Indigenous
cultural material, or safety-sensitive materials as generic training or public
content without the source-governance layer.

## Open Gaps

The main gaps are executable model parity tests, measured geometry/audio
corpora, pressure curves by fingering and player state, wet-out datasets,
tone-hole interaction fixtures, validated release templates, and current-source
review for safety/legal/material claims.
