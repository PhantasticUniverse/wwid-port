# Breath Pressure, Environment, And Tuning

## Scope

This page covers breath-pressure curves, environmental correction,
temperature/humidity state, player variability, tuner behavior, and release
rules for tuning claims.

## Agent Summary

NAF tuning is not a single number per note. Pitch and response depend on breath
pressure, environment, wetness, block state, player technique, fingering, and
measurement method. Flutopedia's breath-pressure page is evidence for a
measured sample, not a universal "all NAFs are low-pressure" rule. Calculators
and environmental corrections are starting points; measured curves are needed
before claiming a design is tuned across a player envelope.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/breath_pressure.htm` | Breath-pressure measurement sample and context. |
| `www.flutopedia.com/tuning_basic6.htm` | Basic six-hole tuning context. |
| `www.flutopedia.com/tuning_Grealish.htm` | Tuning and chromatic/cross-fingering context. |
| `www.flutopedia.com/naflutomat.htm` | Environmental correction and calculator lead. |
| `docs/independent-agent-research/explore/2026-06-04-breath-pressure-curve-tuning-algorithm.md` | Pressure-curve tuning algorithm research. |
| `docs/independent-agent-research/explore/2026-06-04-environmental-tuning-uncertainty.md` | Temperature/humidity uncertainty context. |
| `docs/independent-agent-research/explore/2026-06-04-outdoor-wind-crossflow-robustness.md` | Outdoor wind and crossflow robustness. |
| `docs/independent-agent-research/explore/2026-06-04-cold-weather-gloves-dry-lips-condensation.md` | Cold weather, gloves, dry lips, and condensation envelope. |
| `docs/independent-agent-research/explore/2026-06-04-thermal-travel-storage-damage-envelope.md` | Thermal travel, storage, and tuning-damage envelope. |
| `docs/independent-agent-research/explore/2026-06-04-predicted-measured-pitch-validation-corpus.md` | Needed validation corpus. |
| `docs/independent-agent-research/explore/2026-06-04-artificial-mouth-player-transfer-standard.md` | Artificial-mouth/player transfer gap. |
| `docs/independent-agent-research/explore/2026-06-04-warble-vibrato-tuner-pitch-tracking.md` | Tuner behavior under unstable pitch states. |

## Maker Workflow

1. Declare target pitch standard and temperament.
2. Measure environment: temperature, relative humidity, altitude or pressure if
   relevant, and instrument temperature state.
3. Record breath pressure or a calibrated pressure proxy.
4. Measure pitch across pressure, not only at one comfortable blow point.
5. Mark unstable states such as weak fundamental, warble, squeak, or wet-out.
6. Refuse a single tuning verdict when the measurement state is undefined.
7. Record whether a notation source is fingering tablature, concert pitch, or
   another pitch/notation convention.
8. For ensemble use, map actual sounded pitch for the specific flute rather
   than relying on tablature alone.

## Data Fields

Useful fields include note/fingering id, target frequency, cents target, pitch
standard, temperament, flute key, notation mode, tablature system, actual
sounded pitch map, alternate fingering, unplayable note flag, ensemble target,
pressure, pressure uncertainty, player/artificial-mouth id, temperature,
humidity, instrument warm/cold state, wet/dry state, block position, measured
frequency, cents residual, overblow state, stability label, tuner algorithm,
and refusal reason.

## Do Not Overclaim

Do not generalize one pressure measurement set to all NAFs or all players.

Do not publish "in tune" without pressure and environment context.

Do not treat tuner output as ground truth when the note has warble, vibrato,
weak fundamentals, multiphonics, squeak, wet-out, or unstable onset. The
correct output may be a refusal label.

## Open Gaps

Needed work includes pressure-curve datasets by fingering, environment, block
state, material, and player; artificial-mouth transfer validation; tuner
benchmark fixtures for unstable states; and a pitch-state label schema that
travels with every release claim.
