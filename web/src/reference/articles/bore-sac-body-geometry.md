# Bore, SAC, And Body Geometry

## Scope

This page covers sound-chamber length, acoustic length, bore diameter and
shape, taper, ovality, wall thickness, slow air chamber geometry, body scale,
and body-derived design claims.

## Agent Summary

Bore and body geometry set the acoustic frame within which the sound mechanism,
tone holes, breath pressure, and player loading operate. Flutopedia and
NAFlutomat provide useful starting vocabulary for sound-chamber dimensions,
aspect ratio, acoustic length, wall thickness, and bore diameter, while the
research reports identify the missing measured layer: tolerance stacks, bore
shape and roughness, SAC transient behavior, and pressure-dependent validation.

Do not collapse body length, physical bore length, effective acoustic length,
or target key into one field. A private CAD exemplar or calculator output can
suggest design intent, but release claims still need measured pitch, pressure,
environment, fingering, and wet-state records.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/dimensions.htm` | Design-dimension vocabulary for body, sound chamber, bore, SAC, wall, and TSH fields. |
| `www.flutopedia.com/acoustic_length.htm` | Acoustic-length and correction context. |
| `www.flutopedia.com/bore_diameter.htm` | Bore-diameter discussion and maker context. |
| `www.flutopedia.com/naflutomat.htm` | Calculator starting point for bore, wall, hole, and environment fields. |
| `widesigner-2.6-docs/docs/acoustic-model-reference.html` | Transfer-matrix and NAF concept reference for bore/tonehole modeling. |
| `docs/independent-agent-research/explore/2026-06-04-bore-shape-impedance-timbre.md` | Bore shape, impedance, roughness, taper, and timbre research report. |
| `docs/independent-agent-research/explore/2026-06-04-bore-tolerance-stack.md` | Bore tolerance stack and release-risk report. |
| `docs/independent-agent-research/explore/2026-06-04-amplitude-dependent-end-correction.md` | Amplitude-dependent end-correction and pressure-domain limits. |
| `docs/independent-agent-research/explore/2026-06-04-extreme-pitch-size-scaling-envelope.md` | Extreme pitch-size scaling and low/high flute envelope. |
| `docs/independent-agent-research/explore/2026-06-04-full-low-d-six-hole-response-prediction.md` | Full low-D six-hole response-prediction stress case. |
| `docs/independent-agent-research/explore/2026-06-04-acoustic-conditioning-aging-response.md` | Acoustic conditioning and aging-response route. |
| `docs/independent-agent-research/explore/2026-06-04-dynamic-mechanical-vibration-sideband-coupling.md` | Dynamic mechanical vibration and sideband coupling. |
| `docs/independent-agent-research/explore/2026-06-04-sac-compliance-pressure-smoothing.md` | Slow air chamber pressure-smoothing and transient-behavior report. |
| `indexes/design-exemplars/user_flute_taper_parameter_discovery.v0.json` | Provisional CAD parameter discovery; useful for schema design, not ground truth. |

## Maker Workflow

1. Record coordinate datum, source units, and axis direction before geometry
   values are used.
2. Separate physical length, acoustic length, end correction, and measured
   pitch state.
3. Capture bore profile as a station table, not only a single nominal diameter.
4. Record ovality, taper, roughness, wall thickness, seam/glue-line state, and
   any local discontinuities that could affect impedance.
5. Record SAC volume, breath-hole geometry, plug/seal geometry, and any
   compliance or leakage evidence.
6. Tie geometry to measured pressure, environment, fingering, wet/dry state,
   and tuning residuals before release.

## Data Fields

Useful fields include body length, sound-chamber physical length, effective
acoustic length, bore station, bore diameter, ovality, taper, roughness class,
wall thickness, seam state, glue-line state, SAC length, SAC volume,
breath-hole diameter, breath-hole length, plug geometry, material, moisture
state, target key, pitch standard, temperament, fingering set, pressure curve,
environment, and measurement uncertainty.

## Do Not Overclaim

Do not infer final key from body length alone, CAD filename, or a related
tuning XML.

Do not treat a single bore diameter as enough for taper, ovality, roughness,
or impedance claims.

Do not treat SAC size as a universal comfort or pitch-stability fix without
pressure-transient measurements and leak checks.

Do not use body-derived historical or practitioner heuristics as modern
equal-tempered design rules.

## Open Gaps

Needed work includes a bore/SAC schema, measured bore-profile fixtures,
low-cost bore inspection procedures, SAC pressure-transient datasets,
WIDesigner numeric parity fixtures, and release examples linking predicted
acoustic length to measured pitch under declared pressure and environment.
