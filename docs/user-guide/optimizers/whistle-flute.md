# Whistle & Flute Optimizers

[User Guide](../index.md) > [Optimizers] > Whistle & Flute

The Whistle and Flute study models share most of their optimizers. Both provide calibrators, hole optimizers, bore optimizers, and merged (hole + bore) optimizers. The key differences are in calibration (Whistle adjusts window height; Flute adjusts airstream length) and in Flute-specific bore optimizers (stopper position and headjoint).

This page lists all optimizers for both models. Optimizers that are only available in one model are marked accordingly.


## Calibrators

Calibrators adjust mouthpiece parameters rather than hole or bore geometry. When you select a calibrator, the toolbar button changes from "Optimize" to "Calibrate." Calibrators do not require a constraints document.

The Whistle and Flute study models each have a joint calibrator that appears in the optimizer list. The sub-calibrators (window height / airstream length, and beta) are used internally by the joint calibrator and are not listed separately.

### Whistle calibration

**Display name:** Whistle calibration (Whistle only)
**Dimensions:** 2

Joint calibrator that adjusts both window height and beta simultaneously. Window height refines the maximum frequency prediction (fmax); beta refines the minimum frequency prediction (fmin). Uses the instrument's measured playing range to calibrate both parameters in a single pass.

### Flute calibration

**Display name:** Flute calibration (Flute only)
**Dimensions:** 2

Joint calibrator that adjusts both airstream length and beta simultaneously. Airstream length refines the maximum frequency prediction; beta refines the minimum frequency prediction. Functionally equivalent to the Whistle calibrator but uses the embouchure hole model instead of the fipple window model.

## Hole Optimizers

These optimizers vary tone hole geometry while keeping the bore fixed. They are identical between Whistle and Flute.

### Hole position & size

**Display name:** Hole position & size
**Dimensions:** 2N + 1 (where N = number of holes)

Varies bore length, spacing between holes (measured from the bore top), and all hole diameters. This is the most commonly used optimizer for initial hole layout.

### Hole position only

**Display name:** Hole position only
**Dimensions:** N + 1

Varies bore length and spacing between holes, keeping hole diameters fixed. Use this when hole sizes are already determined and you only need to adjust positions.

### Hole size only

**Display name:** Hole size only
**Dimensions:** N

Varies all hole diameters while keeping positions and bore geometry fixed. Use this when holes are already drilled and you need to determine final sizes.

## Bore Optimizers

These optimizers adjust bore geometry without changing hole positions or sizes.

### Basic taper

**Display name:** Basic taper
**Dimensions:** 2

Creates a two-section tapered bore by adjusting the taper position along the bore and the diameter ratio between the two sections. Available in both Whistle and Flute.

### Bore diameter from top

**Display name:** Bore diameter from top (Whistle only)
**Dimensions:** varies (depends on bore profile)

Adjusts diameter ratios at bore points near the top of the instrument. The number of dimensions depends on how many bore points lie above the tone holes.

### Bore diameter from bottom

**Display name:** Bore diameter from bottom
**Dimensions:** varies (depends on bore profile)

Adjusts diameter ratios at bore points near the bottom of the instrument. Available in both Whistle and Flute.

### Bore spacing from top

**Display name:** Bore spacing from top
**Dimensions:** varies (depends on bore profile)

Repositions bore points near the top of the instrument. Does not change diameters. Available in both Whistle and Flute.

### Stopper position

**Display name:** Stopper position (Flute only)
**Dimensions:** 1

Adjusts the position of the embouchure hole relative to the bore. This is a 1D optimization that uses the Brent algorithm. Only available for transverse flutes.

### Headjoint

**Display name:** Headjoint (Flute only)
**Dimensions:** varies

Adjusts the stopper position and the diameters of upper bore points simultaneously. Combines stopper positioning with headjoint bore shaping. Only available for transverse flutes.

## Merged Optimizers (Hole + Bore)

Merged optimizers combine hole optimization with bore optimization in a single pass. They solve a higher-dimensional problem but can find solutions that require coordinated changes to both holes and bore.

### Holes + basic taper

**Display name:** Holes + basic taper
**Dimensions:** 2N + 3

Combines hole position & size optimization with basic taper bore adjustment. Available in both Whistle and Flute.

### Holes + bore diameter from top

**Display name:** Holes + bore diameter from top (Whistle only)
**Dimensions:** varies

Combines hole optimization with upper bore diameter adjustment.

### Holes + bore diameter from bottom

**Display name:** Holes + bore diameter from bottom
**Dimensions:** varies

Combines hole optimization with lower bore diameter adjustment. Available in both Whistle and Flute.

### Holes + bore spacing

**Display name:** Holes + bore spacing
**Dimensions:** varies

Combines hole optimization with upper bore point repositioning. Available in both Whistle and Flute.

### Holes + headjoint

**Display name:** Holes + headjoint (Flute only)
**Dimensions:** varies

Combines hole optimization with headjoint adjustment (stopper position + upper bore diameters). Only available for transverse flutes.

## Global Optimizers

Global optimizers use the DIRECT-C algorithm for a thorough initial search of the parameter space, followed by BOBYQA refinement. They only appear in the optimizer list when **Use DIRECT optimizer** is enabled in [Settings](../settings.md).

Global optimizers take significantly longer to run but are more likely to find the best solution when the design space is large or when local optimization gets stuck in a poor local minimum.

### Hole spacing (global)

**Display name:** Hole spacing (global)
**Dimensions:** N + 1

DIRECT-C global search for hole positions. Available in both Whistle and Flute.

### Hole size+spacing (global)

**Display name:** Hole size+spacing (global)
**Dimensions:** 2N + 1

DIRECT-C global search for hole positions and sizes. Available in both Whistle and Flute.

### Holes + basic taper (global)

**Display name:** Holes + basic taper (global)
**Dimensions:** 2N + 3

DIRECT-C global search combining hole optimization with bore taper. Available in both Whistle and Flute.

### Holes + bore dia from bottom (global)

**Display name:** Holes + bore dia from bottom (global)
**Dimensions:** varies

DIRECT-C global search combining hole optimization with lower bore diameter adjustment. Available in both Whistle and Flute.

## Summary Table

| Optimizer | Whistle | Flute | Dimensions |
|---|:---:|:---:|---|
| Whistle calibration | yes | -- | 2 |
| Flute calibration | -- | yes | 2 |
| Hole position & size | yes | yes | 2N + 1 |
| Hole position only | yes | yes | N + 1 |
| Hole size only | yes | yes | N |
| Hole spacing (global) | yes | yes | N + 1 |
| Hole size+spacing (global) | yes | yes | 2N + 1 |
| Stopper position | -- | yes | 1 |
| Headjoint | -- | yes | varies |
| Basic taper | yes | yes | 2 |
| Bore diameter from top | yes | -- | varies |
| Bore diameter from bottom | yes | yes | varies |
| Bore spacing from top | yes | yes | varies |
| Holes + basic taper | yes | yes | 2N + 3 |
| Holes + bore diameter from top | yes | -- | varies |
| Holes + bore diameter from bottom | yes | yes | varies |
| Holes + bore spacing | yes | yes | varies |
| Holes + headjoint | -- | yes | varies |
| Holes + basic taper (global) | yes | yes | 2N + 3 |
| Holes + bore dia from bottom (global) | yes | yes | varies |

## Prerequisites

- **Calibrators**: instrument + tuning. No constraints needed.
- **All others**: instrument + tuning + constraints. Use "+ Default" to generate constraints with pre-populated bounds for the selected optimizer.

## See Also

- [Optimizer Overview](overview.md) -- how optimization works and algorithm descriptions.
- [NAF Optimizers](naf.md) -- optimizers in the NAF study model.
- [Reed Optimizers](reed.md) -- optimizers in the Reed study model.
- [Constraints](constraints.md) -- creating and editing constraint bounds.
- [Optimization Workflow](workflow.md) -- step-by-step guide.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Optimizers-in-the-Whistle-and-Flute-Study-Models).*
