# WIDesigner Evidence Boundaries

## Scope

Use this page when a prompt asks whether `widesigner-2.6-docs/` is upstream
WIDesigner, whether it can be cited as source code, whether the upstream
checkout/JAR/release ZIP are committed, or which WIDesigner evidence class is
allowed for a model, fixture, calculator, XML, CAD, benchmark, or public claim.

## Agent Summary

`widesigner-2.6-docs/` is a user-authored documentation and maintenance study
bundle. It is high-value evidence for understanding WIDesigner 2.6 concepts,
claim ownership, caveats, and provenance instructions, but it is not bundled
upstream WIDesigner software.

Edward Kort's upstream WIDesigner source and release evidence are represented
separately by `sources/widesigner_2_6_upstream_public_repo.json`. The actual
source checkout, wiki checkout, release unpack, release ZIP, JAR, transcript
cache, and dependency payloads belong under the ignored local evidence root
`sources/widesigner_2_6_upstream_public_repo/payload/`.

The physical layout is intentional: the repo keeps Xavier's authored docs
bundle at `widesigner-2.6-docs/`, and keeps third-party upstream software
evidence under `sources/widesigner_2_6_upstream_public_repo/`. Legacy ignore
rules also block accidental upstream payloads under `widesigner-2.6-docs/`, but
new upstream evidence should not be placed there.

For encyclopedia answers, cite the authored docs bundle for explanations and
caveats. Cite the upstream source locator, source manifest, hashes, release
roots, and golden fixtures when a claim depends on upstream behavior. Do not
convert docs-only agreement, screenshots, sample XMLs, or optimizer residuals
into measured NAF validation, release readiness, material safety, or public CAD
approval.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `widesigner-2.6-docs/STATUS.md` | Authored bundle status and caveats. |
| `widesigner-2.6-docs/README.md` | Authored documentation map. |
| `widesigner-2.6-docs/AGENTS.md` | Agent navigation and high-risk-claim cautions. |
| `widesigner-2.6-docs/PROVENANCE.md` | Instructions for recreating upstream evidence roots. |
| `widesigner-2.6-docs/docs/acoustic-model-reference.html` | Derived explanation of acoustics, NAF concepts, fipple factor, mouthpiece, and tonehole modeling. |
| `widesigner-2.6-docs/docs/optimization-reference.html` | Derived explanation of optimization routing, residuals, and optimizer behavior. |
| `widesigner-2.6-docs/docs/evidence-matrix.html` | Authored high-risk claim ownership and evidence trail. |
| `widesigner-2.6-docs/docs/review-log.html` | Hostile review history and residual risks. |
| `sources/widesigner_2_6_upstream_public_repo.json` | Separate upstream WIDesigner source/release locator. |
| `sources/widesigner_2_6_upstream_public_repo/README.md` | Tracked note that this source directory is a locator only. |
| `indexes/widesigner/widesigner_2_6_source_manifest.v0.json` | Source/release hash manifest and ignored evidence-root status. |
| `indexes/widesigner/widesigner_claim_limits.v0.json` | Machine-readable allowed/blocked WIDesigner claim boundaries. |
| `indexes/widesigner/widesigner_fixture_status.v0.json` | Agent-facing fixture readiness index; records five narrow materialized fixtures and the static-only runtime boundary. |
| `indexes/widesigner/golden_fixtures/fipple_factor_lowest_note.json` | Narrow upstream/release-bound fipple-factor fixture. |
| `indexes/widesigner/golden_fixtures/naf_tonehole_multiplier_no_finger_adjustment.json` | Narrow upstream/release-bound NAF tonehole constructor fixture. |
| `indexes/widesigner/golden_fixtures/transfer_matrix_impedance_sample.json` | Narrow upstream source-test-bound transfer-matrix fixture; not broad runtime parity or measured validation. |
| `indexes/widesigner/golden_fixtures/unit_conversion_sample_xml.json` | Narrow upstream/release-bound XML lengthType unit-conversion fixture. |
| `indexes/widesigner/golden_fixtures/weighted_residual_optimizer_route.json` | Narrow upstream/release-bound weighted residual route fixture; not optimizer convergence or release proof. |
| `indexes/model-cards/widesigner_2_6_model_card.v0.json` | Model-card state and validation limits. |
| `docs/independent-agent-research/stress-tests/2026-06-05-widesigner-tonehole-unit-drift-stress.md` | Stress note that identified the P0 tonehole multiplier/no-finger-adjustment fixture gap without treating it as measured validation. |

## Decision Rules

| Question | Agent answer |
| --- | --- |
| Is `widesigner-2.6-docs/` upstream WIDesigner? | No. It is this repo's authored documentation bundle about WIDesigner 2.6. |
| Is the upstream WIDesigner repo committed here? | No. The tracked source manifest points to upstream; payload roots are ignored local evidence. |
| Can I cite the docs bundle for concepts? | Yes, with caveats and attribution as derived documentation. |
| Can I cite the docs bundle as source-code proof? | No. Use the upstream locator, source manifest, and recreated evidence roots. |
| Can I use the WIDesigner fixtures as full model validation? | No. They are narrow fipple-factor, NAF tonehole-constructor, transfer-matrix impedance sample, XML unit-conversion, and weighted residual route fixtures, not optimizer convergence/output, CAD datum, broad runtime parity, or measured-flute validation. |
| Can WIDesigner output make CAD publishable or sale-ready? | No by itself. CAD/export, source governance, measured pitch, manufacturing, material, and release gates still apply. |

## Gate Commands

Run these before making WIDesigner-backed claims:

```bash
make widesigner-model-card-gate
make widesigner-model-card-gate-local
make model-parity-conflicts
make model-card-release
make pitch-corpus-release-hold
```

Use the local gate only when the ignored upstream evidence roots are present.
Passing gates with synthetic or narrow fixtures proves boundary behavior, not
real flute validation.

## Do Not Overclaim

Do not say that `widesigner-2.6-docs/` is bundled WIDesigner software, upstream
source code, a release ZIP, a JAR, or a full numerical validation suite.

Do not treat WIDesigner sample XMLs, screenshots, docs text, tutorial video
leads, or optimization residuals as proof of physical tuning, acoustic
playability, material/contact safety, cleanability, manufacturability, or
public release readiness.

Do not collapse `widesigner_2_6_docs_bundle` and
`widesigner_2_6_upstream_public_repo` source IDs. They are separate evidence
classes and should remain separate in page indexes, model cards, and reports.

## Open Gaps

Future work still needs broader WIDesigner transfer-matrix propagation fixtures,
plus deeper optimizer convergence/output replay and
cross-calculator parity against measured pitch records.
Those fixtures should stay commit-safe and refer to ignored upstream evidence
roots by hash and path instead of committing upstream payloads.
