# NAF Optimizers

[User Guide](../index.md) > [Optimizers] > NAF

The NAF (Native American Flute) study model provides eight optimizers. The first is a calibrator that adjusts the fipple factor mouthpiece parameter. The remaining seven are geometry optimizers that vary hole positions, hole sizes, and bore taper within constraint bounds.

All NAF optimizers appear in the Optimizers section of the Study Panel when the NAF study model is selected. NAF does not have any global (DIRECT-C) optimizers.


## Calibrator

### Fipple factor

**Display name:** Fipple factor
**Dimensions:** 1

Determines the fipple factor for a given instrument using measured frequencies. The fipple factor is a mouthpiece correction parameter that accounts for the acoustic behavior of the fipple window and windway.

This is a calibrator, not a geometry optimizer. When you select it, the toolbar button changes from "Optimize" to "Calibrate." It does not require a constraints document -- only an instrument and a tuning.

For best results, calibrate using a no-hole instrument (or a tuning with only the bell note) first. This isolates the mouthpiece behavior from hole interactions. Once calibrated, the fipple factor carries forward into subsequent geometry optimizations.

## Hole Optimizers

### Hole size only

**Display name:** Hole size only
**Dimensions:** N (where N = number of holes)

Varies all hole diameters while keeping hole positions and bore geometry fixed. Use this when hole positions are already drilled and you need to determine final hole sizes.

### Hole size & position

**Display name:** Hole size & position
**Dimensions:** 2N + 1

Varies bore length, spacing between holes (measured from the bore top), and all hole diameters. This is the most commonly used NAF optimizer for initial hole layout. The "+1" dimension is the total bore length.

### Grouped-hole position & size

**Display name:** Grouped-hole position & size
**Dimensions:** varies (fewer than 2N + 1)

Like "Hole size & position," but holes are organized into groups where the spacing within each group is constrained to be uniform. The grouping is defined in the constraints document's `holeGroups` field. Fewer independent parameters means faster convergence, at the cost of requiring all holes in a group to be evenly spaced.

## Taper Optimizers

Taper optimizers introduce a tapered bore section in addition to hole adjustments. A NAF single taper creates a three-section bore profile: a cylindrical head section, a tapered transition, and a cylindrical body. The taper is defined by its position along the bore, its length, and its diameter ratio.

### Taper, no hole grouping

**Display name:** Taper, no hole grouping
**Dimensions:** 2N + 4

Varies bore length, hole spacing (independent), hole sizes, plus three taper parameters: taper ratio, taper position, and taper length. Holes are not grouped -- each hole's spacing is an independent variable.

### Taper, grouped-hole

**Display name:** Taper, grouped-hole
**Dimensions:** varies (fewer than 2N + 4)

Same taper parameters as above, but hole spacing is grouped. Fewer dimensions for faster convergence.

### Taper, no hole grouping, hemispherical

**Display name:** Taper, no hole grouping, hemispherical
**Dimensions:** 2N + 4

Like "Taper, no hole grouping" but models a hemispherical head at the bore top. This is appropriate for NAF designs with a rounded bore termination. The dimension count is the same because the hemispherical shape replaces the cylindrical head -- it does not add a separate parameter.

### Taper, grouped-hole, hemispherical

**Display name:** Taper, grouped-hole, hemispherical
**Dimensions:** varies (fewer than 2N + 4)

Combines taper, hemispherical head, and grouped hole spacing. The fewest dimensions of any taper optimizer.

## Choosing an Optimizer

Start with **Fipple factor** calibration on a no-hole or bell-note-only instrument. Then use **Hole size & position** for initial hole layout. If you want to explore bore taper, use one of the taper optimizers.

Use grouped-hole variants when your design requires uniform hole spacing within groups (common in pentatonic NAF layouts). Use hemispherical variants when your bore has a rounded head termination.

## Prerequisites

- **Calibrator**: instrument + tuning. No constraints needed.
- **All others**: instrument + tuning + constraints. Use "+ Default" to generate constraints with pre-populated bounds for the selected optimizer.

## See Also

- [Optimizer Overview](overview.md) -- how optimization works and algorithm descriptions.
- [Constraints](constraints.md) -- creating and editing constraint bounds.
- [Optimization Workflow](workflow.md) -- step-by-step guide.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Optimizers-in-the-NAF-Study-Model).*
