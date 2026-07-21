# Tone Holes, Undercut, And Direction Holes

## Scope

This page covers tone-hole station and diameter, chimney/wall effects,
undercutting, cross-fingered states, direction holes, hand loading, mutual
radiation, and foot-end impedance.

## Agent Summary

Tone holes are not independent buttons on an otherwise simple tube. Hole size,
station, wall thickness, chimney shape, undercut, open/closed neighboring
holes, hand coverage, direction holes, foot-end geometry, and pressure all
interact. Simple end-correction formulas may be useful starting points, but the
research corpus explicitly rejects treating them as validated for low flutes,
direction holes, cross-fingerings, and hand-loaded states.

For design use, capture the full hole field and measurement state. For release
claims, require measured pitch and response data under the actual fingering and
player/environment conditions.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/dimensions.htm` | Finger-hole dimensions and geometry vocabulary. |
| `www.flutopedia.com/acoustic_length.htm` | Acoustic-length and correction context. |
| `www.flutopedia.com/naflutomat.htm` | Calculator starting point for tone-hole layout. |
| `docs/independent-agent-research/explore/2026-06-04-tone-hole-mutual-radiation-interaction-model.md` | Mutual radiation and interaction gap. |
| `docs/independent-agent-research/explore/2026-06-04-cross-fingering-sensitivity-model.md` | Cross-fingering sensitivity model needs. |
| `docs/independent-agent-research/explore/2026-06-04-direction-holes-foot-end-impedance.md` | Direction-hole and foot-end impedance context. |
| `docs/independent-agent-research/explore/2026-06-04-direction-hole-algorithm-validation.md` | Direction-hole algorithm validation gap. |
| `docs/independent-agent-research/explore/2026-06-04-hand-bore-radiation-loading.md` | Hand and bore radiation loading context. |
| `docs/independent-agent-research/explore/2026-06-04-joint-tuning-ergonomic-hole-optimization.md` | Joint tuning and ergonomic hole optimization. |
| `docs/independent-agent-research/explore/2026-06-04-tactile-landmark-hole-orientation-design.md` | Tactile landmark and hole-orientation design. |
| `docs/independent-agent-research/explore/2026-06-04-undercut-repeatability.md` | Undercut repeatability and measurement concerns. |
| `widesigner-2.6-docs/docs/acoustic-model-reference.html` | WIDesigner tonehole and transfer-matrix modeling lead. |
| `sources/nakai_art_of_native_american_flute_1996.json` | Historical/practitioner source guide for hole-count and body-derived layout topic discovery. |

## Maker Workflow

1. Record station from a declared datum, diameter, wall thickness, chimney
   shape, undercut direction, undercut depth, and angular offset for each hole.
2. Record whether direction holes and foot-end features are present.
3. Measure all target fingerings, including cross-fingerings and alternate
   rows, rather than inferring from open-hole states only.
4. Test with hands in playing position when hand loading can matter.
5. Preserve as-designed, as-drilled, as-undercut, as-finished, and as-tuned
   states separately.
6. Mark body-derived or hand-derived historical layout heuristics as descriptive
   practice, not equal-tempered or calculator-backed tuning evidence.

## Data Fields

Minimum fields include hole id, datum convention, station, station uncertainty,
diameter, diameter uncertainty, wall/chimney depth, undercut geometry, angular
offset, burr state, finish state, open/closed pattern, neighboring-hole state,
direction-hole geometry, hand-loading condition, pressure, environment, target
frequency, measured frequency, and cents residual.

## Do Not Overclaim

Do not assume independent tone-hole end corrections are validated for every NAF
geometry. The corpus marks low flutes, direction holes, cross-fingerings, and
hand-loaded states as especially risky.

Do not say undercut is only a tuning convenience. It changes the local geometry
and can hide drilling or layout errors.

Do not collapse direction holes into decorative features without acoustic and
source review. They may change foot-end impedance and public interpretation.

Do not use body-measure hole spacing or arbitrary/keyed maker variation as a
modern reproducible tuning recipe unless measured validation supports it.

## Open Gaps

Remaining work includes measured multi-hole interaction fixtures, direction-hole
datasets, hand-loading tests, undercut metrology, WIDesigner tonehole parity
fixtures, and a release validator that rejects pitch claims when the measured
state differs from the design state.
