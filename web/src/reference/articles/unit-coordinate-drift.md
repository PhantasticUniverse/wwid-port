# Unit, Coordinate, And Drift Gates

## Scope

This page covers canonical units, geometry datums, hole-order conventions,
coordinate transforms, unit conversion, residual drift, model drift, and stale
validation alerts.

## Agent Summary

Agents must not ingest unitless geometry or compare residuals from incompatible
coordinate systems. A flute design needs a declared datum, axis direction,
station origin, unit system, hole numbering convention, angular convention, and
uncertainty. Model outputs also need drift monitoring: a good residual from an
old calibration set may be stale after a new source, material, player envelope,
or measurement system enters the corpus.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `indexes/unit-coordinate/unit_coordinate_contract.v0.json` | Machine-readable unit and coordinate contract. |
| `indexes/drift-monitoring/monitoring_signals.v0.json` | Drift signal definitions. |
| `docs/independent-agent-research/extend/2026-06-04-unit-coordinate-validation-local-synthesis.md` | Unit-coordinate synthesis. |
| `docs/independent-agent-research/extend/2026-06-04-algorithm-drift-monitoring-synthesis.md` | Drift-monitoring synthesis. |
| `docs/independent-agent-research/explore/2026-06-04-missing-data-imputation-artifact-geometry.md` | Geometry imputation and missing-data risk. |
| `docs/independent-agent-research/explore/2026-06-04-measurement-system-analysis-calibration-drift.md` | Measurement-system drift context. |
| `indexes/design-exemplars/user_flute_taper_parameter_discovery.v0.json` | Provisional CAD example requiring authoritative units/datum export. |

## Maker Workflow

1. Require explicit units for every geometry field.
2. Define mouth-to-foot or foot-to-mouth station convention before comparing
   hole positions.
3. Preserve serialized XML hole order until physical mapping is verified.
4. Record measurement uncertainty and instrument/device calibration state.
5. Re-run validation when source data, material families, model code, tuning
   targets, or measurement systems change.
6. Block model release when residuals drift outside predeclared thresholds.

## Data Fields

Useful fields include unit system, datum, axis direction, station origin,
hole-order convention, angular zero, handedness, coordinate transform, source
uncertainty, measurement method, calibration device, fixture id, model version,
residual metric, drift signal, threshold, trigger date, and re-review owner.

## Do Not Overclaim

Do not compare CAD, WIDesigner XML, Flutopedia chart rows, and measured
instruments until their units, hole order, and coordinate conventions are made
explicit.

Do not fill missing geometry with plausible values and call it measured.

Do not treat `indexes/design-exemplars/user_flute_taper_parameter_discovery.v0.json`
as public or release-ready geometry. It is a private, source-governed CAD
exemplar for unit/datum discovery until export manifests, release cards, and
measured validation make a narrower claim available.

Do not assume a model remains validated after a new design family, player
envelope, material, wet-state condition, or measurement system is added.

## Open Gaps

Needed work includes executable unit-coordinate validation for every geometry
artifact, WIDesigner XML unit fixtures, CAD export crosswalks, drift dashboard
instances, and revalidation triggers tied to model cards and release binders.
