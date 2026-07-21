# Flue, TSH, Fipple, And Voicing

## Scope

This page covers NAF sound-mechanism geometry: flue dimensions, true sound hole
dimensions, splitting edge, bevels, block/nest state, voicing adjustments, and
WIDesigner's fipple factor.

## Agent Summary

The sound mechanism is a coupled fluid-acoustic system. Small changes in flue
depth, flue width, flue taper, TSH length/width, edge lift, bevel condition,
block seal, block position, surface finish, wetness, and breath pressure can
change attack, noise, pitch, octave behavior, and stability. Treat WIDesigner's
fipple factor as a calibration parameter in the model, not a physical
measurement of the exact flue/TSH/block geometry.

Flutopedia supplies useful terminology and maker measurement conventions. The
research corpus says the current missing layer is measured response surfaces
and validated fixtures.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/dimensions.htm` | Feature vocabulary and sound-mechanism dimensions. |
| `www.flutopedia.com/fipple.htm` | Fipple-region measurement language and diagrams. |
| `docs/independent-agent-research/explore/2026-06-04-fipple-bevel-response-surface.md` | Research program for bevel/edge response surfaces. |
| `docs/independent-agent-research/explore/2026-06-04-jet-edge-cfd-voicing-map.md` | CFD and jet-edge voicing research lead. |
| `docs/independent-agent-research/explore/2026-06-04-nest-microgeometry-surface-finish.md` | Nest and surface-finish sensitivity context. |
| `docs/independent-agent-research/explore/2026-06-04-block-position-leakage-acoustic-map.md` | Block position and leakage mapping. |
| `docs/independent-agent-research/explore/2026-06-04-block-tying-force-repeatability.md` | Tying force and repeatability context. |
| `docs/independent-agent-research/extend/2026-06-04-interchangeable-block-configuration-release-binder.md` | Interchangeable block configuration release binder. |
| `widesigner-2.6-docs/docs/acoustic-model-reference.html` | WIDesigner NAF mouthpiece/fipple and transfer-matrix reference. |
| `widesigner-2.6-docs/docs/optimization-reference.html` | Fipple-factor optimization and caveats. |
| `sources/nakai_art_of_native_american_flute_1996.json` | Book source guide for practitioner sound-apparatus vocabulary and care-state topic discovery. |

## Maker Workflow

For voicing and design capture:

1. Record flue length, width, depth, taper, and surface condition at defined
   measurement locations.
2. Record TSH length, width, edge lift, bevel geometry, and burr/roundover
   state.
3. Record block position, block footprint, tying method, tying force proxy, and
   leak-test result.
4. Measure response across breath pressure and wet/dry state before and after
   voicing changes.
5. Distinguish intentional voicing compensation from upstream manufacturing
   defects such as drill wander, cutter wear, burrs, or datum shift.
6. Record maker vocabulary such as bird/saddle, spacer plate, block, nest, flue,
   edge, and related local synonyms before mapping it to encyclopedia terms.
7. Record disassembly, drying, oiling, refit, and leak-check state when care
   actions affect the sound mechanism.

## Data Fields

Useful fields include flue depth at mouth end and TSH end, flue width, flue
length, flue taper, TSH length, TSH width, splitting-edge lift, bevel angle,
edge radius, block material, block position, block force proxy, leak result,
surface finish, moisture state, breath pressure, onset threshold, noise metric,
pitch residual, and player notes.

## Do Not Overclaim

Do not equate "fipple factor" with a measured physical feature. It calibrates
model behavior in WIDesigner and needs fixture-backed interpretation before it
can carry physical design claims.

Do not assume final voicing erases process problems. A flute can be made to
sound acceptable while retaining hidden burrs, asymmetrical chimneys, unstable
block seating, or wet-state fragility.

Do not claim one flue, bevel, finish, or block strategy is universally best
without measured response surfaces across material, moisture, pressure, and
player envelope.

## Open Gaps

Needed work includes WIDesigner runtime replay coverage,
measured bevel/flue/TSH response surfaces, wet-state voicing replay, and an
inspection protocol that links manufacturing defects to later acoustic
symptoms. The materialized WIDesigner fipple-factor lowest-note fixture is a narrow model
regression, not a measured voicing response surface.
