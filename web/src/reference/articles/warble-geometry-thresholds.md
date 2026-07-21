# Warble Geometry Thresholds

## Scope

This page covers quantitative thresholds and validation gates for steady-state
warble, vibrato-induced warble, register transition, overblow, squeak, weak
fundamental, timbre change, and related geometry controls.

It does not give final threshold values. The local corpus has strong
qualitative and protocol evidence, but not a validated response surface that
predicts warble from geometry across flute families.

## Agent Summary

Warble is a controlled instability only when the measurement state and intent
support that label. Narrow flues, block position, chimney/depth, TSH geometry,
edge offset, bore response, pressure, wet state, and player input can all
affect warble-like behavior. The current encyclopedia can help an agent avoid
bad claims; it cannot yet tell a maker "set this dimension to get warble" with
engineering confidence.

The correct answer to most design questions is a gate: define the desired
state, measure onset and extinction pressure, classify the audio, record
geometry and block state, and compare against held-out examples before
publishing maker rules.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `docs/naf-encyclopedia/warble-register-timbre.md` | Parent routing page for warble and pitch-state labels. |
| `docs/naf-encyclopedia/flue-tsh-fipple-voicing.md` | Sound-mechanism geometry context. |
| `docs/naf-encyclopedia/breath-pressure-curve-model.md` | Pressure-curve route for onset and hysteresis. |
| `www.flutopedia.com/warble.htm` | Local warble terminology, examples, and spectrogram context. |
| `www.flutopedia.com/dimensions.htm` | Local flue/TSH/chimney/splitting-edge geometry context. |
| `docs/independent-agent-research/explore/2026-06-04-warble-geometry-control.md` | Core warble geometry research program. |
| `docs/independent-agent-research/explore/2026-06-04-warble-vibrato-tuner-pitch-tracking.md` | Tuner and pitch-tracking stress. |
| `docs/independent-agent-research/explore/2026-06-04-first-register-onset-hysteresis-map.md` | Onset and hysteresis research report. |
| `docs/independent-agent-research/explore/2026-06-04-second-register-squeak-risk.md` | Register and squeak-risk context. |
| `docs/independent-agent-research/extend/2026-06-04-pitch-state-validation-label-contract.md` | Pitch-state label contract. |

## Threshold Workflow

1. Decide whether warble is desired, undesired, diagnostic, or merely observed.
2. Record geometry: bore, SAC, flue depth/width/length/taper, TSH size, edge
   lift, bevel, chimney, block position, block mass, and leak state.
3. Sweep pressure upward and downward to capture onset, extinction, and
   hysteresis.
4. Label audio state before pitch scoring.
5. Distinguish steady-state warble from player vibrato, leakage, wet-out,
   overblow, squeak, and weak fundamental.
6. Validate any threshold on held-out flutes or adjustable test bodies.
7. Publish only directional guidance until measured thresholds survive
   repeatability and transfer checks.

## Minimum Record

A threshold record should include geometry id, measured geometry, block state,
pressure trace, environment, wet state, player/fixture id, audio features,
pitch-state labels, warble onset pressure, warble extinction pressure,
overblow pressure, squeak label, weak-tone label, repeat count, uncertainty,
desired-design state, and release decision.

## Do Not Overclaim

Do not call any modulation warble without checking pressure, player intent,
audio features, and source context.

Do not use averaged tuner pitch as proof of a stable warbling design.

Do not promote a single maker anecdote or one adjustable-block trial into a
geometry rule.

Do not attach cultural, traditional, or authenticity meaning to a warble
example unless the source-governance evidence supports that specific claim.

## Open Gaps

Future output: `indexes/audio-validation/pitch_state_label.schema.json`.

Future output: `indexes/warble/warble_geometry_matrix.schema.json`.

Future output: `indexes/warble/warble_trials.jsonl`.

Needed work includes rights-cleared audio examples, adjustable test bodies,
pressure-sweep protocols, audio-feature classifiers, wet/dry repeatability,
geometry sensitivity matrices, and maker-facing threshold bands that remain
honest about uncertainty.
