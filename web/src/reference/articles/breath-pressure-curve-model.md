# Breath Pressure Curve Model

## Scope

This page covers pressure-to-pitch and pressure-to-response models for NAF
design, tuning, validation, and release. It routes questions about blow curve,
breath curve, pressure sweep, artificial mouth transfer, player envelope,
environment correction, and pressure-dependent tuning residuals.

It does not provide a validated universal curve form. No current local artifact
proves that a particular pressure model transfers across flute geometries,
players, wet states, blocks, materials, or fingering systems.

## Agent Summary

Treat breath pressure as an independent measurement axis, not as a footnote to
pitch. A tuning claim at one comfortable breath point is not enough to release
a design, diagnose a bad hole, compare algorithms, or decide whether a maker
should move a hole, alter the flue, shift the block, or change player
instructions.

The current corpus is strong at defining the needed variables and the refusal
logic. It is still weak on measured curve families, uncertainty budgets,
player-to-fixture transfer, and acceptance thresholds. Agents should answer
with a measurement protocol and gate unless a local pressure-curve dataset is
explicitly cited.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `docs/naf-encyclopedia/breath-pressure-environment-tuning.md` | Parent routing page for pressure, environment, and tuning claims. |
| `www.flutopedia.com/breath_pressure.htm` | Local breath-pressure measurement anchor; evidence for a sample, not a universal rule. |
| `www.flutopedia.com/naflutomat.htm` | Calculator/environment baseline that does not include per-fingering pressure curves. |
| `docs/independent-agent-research/explore/2026-06-04-breath-pressure-curve-tuning-algorithm.md` | Core pressure-curve algorithm gap and proposed research program. |
| `docs/independent-agent-research/explore/2026-06-04-artificial-mouth-player-transfer-standard.md` | Transfer gap between fixture curves and human playing. |
| `docs/independent-agent-research/explore/2026-06-04-bayesian-calibration-uncertainty-budget.md` | Uncertainty and calibration context for model claims. |
| `docs/independent-agent-research/explore/2026-06-04-automated-acoustic-quality-control.md` | Production acoustic QC context for pressure sweeps and state labels. |
| `docs/naf-encyclopedia/model-card-gates.md` | Refusal and release rules for model outputs. |
| `indexes/pressure-curves/pressure_pitch_curve.schema.json` | Executable per-fingering sweep contract: maker-action or scoring decisions force full sweeps, repeats, calibrated sensors, wet-state labels, uncertainty limits, domain wording, and pitch-corpus binding. |
| `indexes/pressure-curves/pressure_pitch_curve_records.jsonl` | Real-records container, intentionally empty: no real measured pressure sweep exists yet, so it holds zero rows. |
| `indexes/pressure-curves/pass_pressure_pitch_curve_protocol_calibration.synthetic-valid.json` | Synthetic protocol-calibration sweep fixture with labeled onset, stable, warble, and overblow points; not real measurement evidence. |
| `indexes/pressure-curves/fail_pressure_pitch_curve_single_point_maker_action.invalid.json` | Expected-invalid canary blocking maker-action wording from one comfortable-pressure reading. |
| `indexes/pressure-curves/fail_pressure_pitch_curve_artificial_mouth_player_claim_without_transfer.invalid.json` | Expected-invalid canary blocking artificial-mouth data from player-facing claims without a reviewed transfer standard. |
| `indexes/pressure-curves/artificial_mouth_player_transfer.schema.json` | Executable transfer-equivalence contract: per-metric, per-family equivalence requires a fully characterized rig, consented human baseline, minimum comparison counts, and a reviewer; raw physiological data is never public. |
| `indexes/pressure-curves/pass_artificial_mouth_transfer_insufficient_evidence_hold.synthetic-valid.json` | Synthetic insufficient-evidence hold fixture limited to fixture screening; not a transfer validation. |
| `indexes/pressure-curves/fail_artificial_mouth_transfer_equivalence_without_characterization.invalid.json` | Expected-invalid canary blocking player-equivalence claims from an uncharacterized rig without consented baselines. |

## Model Workflow

1. Declare the model purpose: design prediction, tuning diagnosis, release QC,
   player-fit guidance, algorithm benchmark, or research comparison.
2. Define the pressure variable and units. Record whether it is mouth pressure,
   SAC pressure, fixture pressure, pressure drop, flow proxy, or another
   measured signal.
3. Sweep pressure per fingering rather than sampling one point.
4. Record onset, stable-tone window, pitch slope, instability labels, overblow,
   squeak, weak fundamental, wet-out, and hysteresis.
5. Keep environment, instrument temperature, wet/dry state, block position,
   and player/fixture identity in the same record.
6. Fit only inside the measured domain. Refuse extrapolation to other keys,
   bore scales, fingering systems, materials, or player classes without
   transfer evidence.
7. Report confidence intervals and unresolved residuals.
8. Connect any maker recommendation to the measured cause class. A sharp note
   at high pressure may not imply the same fix as a sharp note at low pressure.

## Minimum Record

A pressure-curve record should include instrument id, geometry version, key or
root claim, fingering id, target frequency, pressure units, pressure sensor,
calibration date, player or fixture id, environment, wet-state label, block
state, measured frequency, cents residual, onset pressure, stable pressure
range, pitch slope, instability labels, repeated-trial count, uncertainty,
model version, and release decision.

## Do Not Overclaim

Do not convert a single pressure reading into a general statement about all
NAFs, all keys, all players, or all beginners.

Do not call an algorithm validated because it predicts dry, one-pressure pitch
for a small set of easy fingerings.

Do not use artificial-mouth data as player data unless a transfer standard
exists for the claim being made.

Do not average unstable states into a pitch curve. Route them through
`docs/naf-encyclopedia/warble-register-timbre.md` and pitch-state labeling.

## Open Gaps

The sweep-record schema, the transfer-equivalence schema, their synthetic
fixtures and expected-invalid canaries, and the intentionally empty
real-records container above are now materialized and enforced through the
schema-fixture manifest. The remaining gap is physical: real measured pressure
sweeps across notes, keys, wet states, blocks, materials, low flutes,
novice/expert players, and fixtures; curve-fit comparisons; uncertainty
budgets; measured transfer metrics with consented human baselines; and release
thresholds for when a pressure curve is good enough to support maker action.
Do not fabricate sweep rows or transfer baselines; zero real records is the
correct current state.
