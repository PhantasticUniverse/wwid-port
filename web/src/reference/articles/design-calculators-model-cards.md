# Design Calculators And Model Cards

## Scope

This page covers NAFlutomat, WIDesigner, transfer-matrix models, fipple-factor
optimization, inverse design, optimizer behavior, regression fixtures, model
cards, and governance boundaries.

## Agent Summary

Calculators and optimizers are useful design tools, not release proof.
NAFlutomat gives starting layouts and environmental corrections. WIDesigner is a
high-value modeling reference for NAF concept structure, toneholes, mouthpiece
modeling, fipple factor, transfer matrices, residuals, and optimization flow.
But the local authored `widesigner-2.6-docs` bundle is derived documentation,
and most load-bearing algorithm claims still need source/release roots or
golden numeric fixtures. Five narrow WIDesigner fixtures are now materialized:
fipple-factor lowest-note behavior, NAF tonehole-constructor behavior,
transfer-matrix impedance sampling, XML lengthType unit conversion, and weighted
residual routing. They do not validate optimizer convergence/output, CAD datum
compatibility, or measured flute
accuracy.

Every model should have a model card that names inputs, assumptions, training or
calibration data, unsupported regimes, objective function, optimizer settings,
version, and validation results.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/naflutomat.htm` | NAFlutomat calculator lead and warning context. |
| `docs/independent-agent-research/explore/2026-06-04-naflutomat-tested-code-extraction.md` | NAFlutomat code extraction and test gap. |
| `indexes/model-cards/naflutomat_model_card.v0.json` | Current NAFlutomat model-card artifact. |
| `widesigner-2.6-docs/STATUS.md` | Authored WIDesigner documentation-bundle state and caveats. |
| `widesigner-2.6-docs/docs/acoustic-model-reference.html` | WIDesigner acoustics, NAF, fipple, mouthpiece, and tonehole reference. |
| `widesigner-2.6-docs/docs/optimization-reference.html` | DIRECT, BOBYQA, Brent, CMA-ES, residuals, and optimization flow. |
| `widesigner-2.6-docs/docs/review-log.html` | Hostile review and residual-risk record. |
| `sources/widesigner_2_6_upstream_public_repo.json` | Separate upstream WIDesigner repository/release locator and source-boundary manifest. |
| `indexes/widesigner/widesigner_2_6_source_manifest.v0.json` | Source/release hash and ignored evidence-root state for WIDesigner fixture governance. |
| `indexes/widesigner/widesigner_fixture_status.v0.json` | Compact fixture readiness and static-only runtime boundary for agent routing. |
| `indexes/widesigner/widesigner_claim_limits.v0.json` | Machine-readable WIDesigner blocked/allowed claim map for docs, source, optimizer, XML, material, and tutorial-video claims. |
| `indexes/widesigner/golden_fixtures/fipple_factor_lowest_note.json` | Narrow source/release-bound WIDesigner fipple-factor fixture. |
| `indexes/widesigner/golden_fixtures/naf_tonehole_multiplier_no_finger_adjustment.json` | Narrow source/release-bound NAF tonehole constructor fixture. |
| `indexes/widesigner/golden_fixtures/transfer_matrix_impedance_sample.json` | Narrow source-test-bound transfer-matrix sample fixture. |
| `indexes/widesigner/golden_fixtures/unit_conversion_sample_xml.json` | Narrow source/release-bound XML lengthType unit-conversion fixture. |
| `indexes/widesigner/golden_fixtures/weighted_residual_optimizer_route.json` | Narrow source/release-bound weighted residual route fixture. |
| `docs/independent-agent-research/stress-tests/2026-06-06-boole-post-00d296e-model-workpack.md` | Post-workpack stress report confirming calculator agreement, optimizer residuals, and pressure-normalized rows remain release-hold evidence until model-card, split, and measured-corpus joins pass. |
| `docs/independent-agent-research/stress-tests/2026-06-06-boole-post-1d647df-model-validation-frontier.md` | Post-1d647df model-validation frontier stress confirming WIDesigner documentation and fixtures are not laundered into measured NAF validation while flagging stale transfer-matrix fixture wording for repair. |
| `docs/independent-agent-research/extend/2026-06-04-widesigner-2-6-model-card-source-intake-fixture-bridge.md` | WIDesigner intake/model-card bridge. |
| `docs/independent-agent-research/extend/2026-06-04-governed-inverse-design-assistant-packet.md` | Governed inverse-design assistant packet. |
| `docs/independent-agent-research/extend/2026-06-04-design-run-decision-replay-bundle.md` | Design-run decision replay bundle. |
| `docs/independent-agent-research/extend/2026-06-04-evidence-graph-local-synthesis.md` | Evidence-graph local synthesis for model/retrieval handoff. |
| `indexes/inverse-design/governed_inverse_design_assistant_packet.v0.json` | Machine-readable inverse-design handoff artifact. |

## Maker Workflow

1. Use calculators to propose dimensions, not to certify playability.
2. Record software version, model parameters, objective, optimizer settings,
   fipple factor, targets, weights, and constraints.
3. Keep predicted and measured values in the same record.
4. Use held-out fixtures when changing models or optimizer routing.
5. Refuse optimizer suggestions outside validated geometry, material, pressure,
   or player envelopes.
6. Preserve decision traces so a later return or failure can be replayed.

## Data Fields

Model-card fields should include model id, version, source paths, input schema,
output schema, assumptions, parameter bounds, unsupported cases, objective,
optimizer, random seed or multi-start settings, calibration data, validation
fixtures, known residual patterns, governance limits, and release state.

## Do Not Overclaim

Do not cite WIDesigner docs as primary upstream source code. Use
`widesigner-2.6-docs/PROVENANCE.md` and
`sources/widesigner_2_6_upstream_public_repo.json` when a claim needs deeper
re-verification.

Do not treat fipple-factor optimization, including the materialized lowest-note
fixture, as physical measurement.

Do not allow an inverse-design assistant to skip source governance, material
safety, cultural review, privacy, or measured validation.

## Open Gaps

Needed work includes WIDesigner runtime replay coverage,
NAFlutomat/CrossTune regression vectors,
executable parity tests, optimizer decision-trace validators, XML unit
crosswalks, and benchmark datasets that include failure/refusal cases rather
than only successful designs.
