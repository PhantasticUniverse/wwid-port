# Reed Optimizers

[User Guide](../index.md) > [Optimizers] > Reed

The Reed study model provides twelve optimizers for single reed, double reed, and lip reed instruments. The first is a calibrator that adjusts mouthpiece parameters. The rest are hole optimizers, bore optimizers, and merged (hole + bore) optimizers. One global optimizer is available when DIRECT-C is enabled in Settings.


## Calibrator

### Reed calibrator

**Display name:** Reed calibrator
**Dimensions:** 2

Adjusts the alpha and beta mouthpiece parameters simultaneously using 2D BOBYQA optimization. Alpha controls the overall reed impedance scaling; beta controls the playing frequency prediction.

Unlike the Whistle and Flute calibrators (which use the fmin/fmax evaluator), the Reed calibrator uses the cent deviation evaluator. It minimizes the sum of squared cent deviations between predicted and target frequencies directly.

When you select this calibrator, the toolbar button changes from "Optimize" to "Calibrate." It does not require a constraints document -- only an instrument and a tuning.

## Hole Optimizers

### Hole position & size

**Display name:** Hole position & size
**Dimensions:** 2N + 1 (where N = number of holes)

Varies bore length, spacing between holes (measured from the bore top), and all hole diameters. This is the standard optimizer for initial hole layout on reed instruments.

### Hole position only

**Display name:** Hole position only
**Dimensions:** N + 1

Varies bore length and hole spacing while keeping hole diameters fixed.

### Hole size only

**Display name:** Hole size only
**Dimensions:** N

Varies all hole diameters while keeping positions and bore geometry fixed.

## Bore Optimizers

Reed instruments have distinctive bore profiles -- conical, stepped, or reverse-conical depending on the instrument type. These optimizers adjust bore geometry without changing hole positions or sizes.

### Bore diameter from bottom

**Display name:** Bore diameter from bottom
**Dimensions:** varies (depends on bore profile)

Adjusts diameter ratios at bore points near the bottom (bell end) of the instrument. The number of dimensions depends on how many bore points lie below the body top.

### Bore position

**Display name:** Bore position
**Dimensions:** varies

Changes the absolute position of the bottom bore point and the relative spacing between bore points. This optimizer uses a mix of absolute and relative parameters, making it useful for adjusting the overall bore length while preserving proportional relationships between bore points.

### Bore from bottom

**Display name:** Bore from bottom
**Dimensions:** varies

Adjusts both position and diameter at bore points near the bottom of the instrument. Combines the effects of "Bore diameter from bottom" and "Bore position" into a single optimization.

## Merged Optimizers (Hole + Bore)

Merged optimizers combine hole optimization with bore optimization in a single pass. They solve a higher-dimensional problem but can find coordinated solutions that neither optimizer alone would discover.

### Holes + bore diameter from bottom

**Display name:** Holes + bore diameter from bottom
**Dimensions:** varies

Combines hole position & size optimization with lower bore diameter adjustment.

### Holes + bore position

**Display name:** Holes + bore position
**Dimensions:** varies

Combines hole position & size optimization with bore point positioning.

### Holes + bore from bottom

**Display name:** Holes + bore from bottom
**Dimensions:** varies

Combines hole optimization with bore position and diameter adjustment at lower bore points. The highest-dimensional merged optimizer for reed instruments.

## Global Optimizer

Global optimizers use the DIRECT-C algorithm for an initial thorough search of the parameter space, followed by BOBYQA refinement. They only appear in the optimizer list when **Use DIRECT optimizer** is enabled in [Settings](../settings.md).

### Holes + bore dia from bottom (global)

**Display name:** Holes + bore dia from bottom (global)
**Dimensions:** varies

DIRECT-C global search combining hole optimization with lower bore diameter adjustment. Takes significantly longer than the local variant but is more likely to find the global optimum on instruments with complex bore profiles.

## Local vs Global Optimization

Local optimizers (BOBYQA) are fast -- typically finishing in seconds -- and refine the design starting from the current instrument geometry. They converge to the nearest local minimum, which may or may not be the best solution overall.

Global optimizers (DIRECT-C followed by BOBYQA) search more thoroughly by exploring the entire parameter space. This takes longer (minutes rather than seconds) but may find better solutions, especially when:

- The current instrument geometry is far from optimal.
- The design space has multiple local minima (common with complex bore profiles).
- You are exploring a new design from scratch rather than refining an existing one.

Use local optimizers for iterative refinement. Use the global optimizer when you suspect the local optimum is not the best solution or when starting from an unoptimized design.

## Prerequisites

- **Calibrator**: instrument + tuning. No constraints needed.
- **All others**: instrument + tuning + constraints. Use "+ Default" to generate constraints with pre-populated bounds for the selected optimizer.

## See Also

- [Optimizer Overview](overview.md) -- how optimization works and algorithm descriptions.
- [NAF Optimizers](naf.md) -- optimizers in the NAF study model.
- [Whistle & Flute Optimizers](whistle-flute.md) -- optimizers in the Whistle and Flute study models.
- [Constraints](constraints.md) -- creating and editing constraint bounds.
- [Optimization Workflow](workflow.md) -- step-by-step guide.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Optimizers-in-the-Reed-Study-Model).*
