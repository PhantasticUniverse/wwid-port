# Flute (Transverse Flute)

[User Guide](../index.md) > [Study Models] > Flute

The Flute study model is for designing and optimizing transverse flutes with embouchure holes -- simple-system flutes, fifes, and similar keyless instruments. The player blows across an embouchure hole rather than into a fipple, and the model accounts for this different sound production mechanism.

Like the Whistle model, the Flute model predicts **nominal**, **maximum**, and **minimum** playing frequencies for each fingering. The key difference is in the mouthpiece: the Flute model uses an embouchure hole with an airstream length parameter instead of a fipple window.


## Prerequisites

- [Getting Started](../getting-started.md) -- loading files and running your first evaluation
- [Sample Files](../sample-files.md) -- bundled Flute instruments, tunings, and constraints
- [Whistle](whistle.md) -- the Flute model shares many concepts with the Whistle model

## Frequency Types

The Flute model uses the same four frequency types as the Whistle model:

| Frequency | Description |
|-----------|-------------|
| **Target** | The desired pitch from the tuning file. |
| **Predicted (nominal)** | The pitch the model calculates under normal playing. |
| **Maximum (fmax)** | The highest sustainable pitch before overblowing. |
| **Minimum (fmin)** | The lowest sustainable pitch before the note drops out. |

The evaluation table shows all four values with cents deviations. Tuning files can include `frequencyMax` and `frequencyMin` fields for calibration.

## Embouchure Hole Geometry

The instrument definition includes an embouchure hole section with these fields:

| Parameter | Description |
|-----------|-------------|
| **Length** | Length of the embouchure hole along the bore axis (metres). |
| **Width** | Width of the embouchure hole perpendicular to the bore axis (metres). |
| **Height** | Depth of the embouchure hole from the outer surface to the bore wall (metres). |

These are physical dimensions you measure directly from the instrument. They determine how the model computes the embouchure's acoustic impedance.

## Calibration Parameters

The Flute model has two calibration parameters:

| Parameter | Description | Starting value |
|-----------|-------------|----------------|
| **Airstream length** | The effective distance from the player's lips to the far edge of the embouchure hole (metres). This is difficult to measure directly because it depends on the player's embouchure. Refined by calibration to improve fmax prediction. | Estimate based on embouchure hole width plus lip offset. |
| **Beta factor** | A dimensionless parameter (same concept as in the Whistle model) that controls fmin prediction. Represents the effective fraction of the embouchure opening. | ~0.4 is a reasonable starting point. |

Both parameters appear in the instrument editor's Mouthpiece section.

## Calibrators

The Flute model has three calibrators. Select the joint calibrator from the Optimizer dropdown; the Optimize button changes to **Calibrate**. No constraints are needed.

### Flute Calibration (Joint)

The recommended calibrator. Adjusts **both** airstream length and beta factor simultaneously using a 2D optimizer (BOBYQA). Requires tuning data with measured `frequencyMax` and `frequencyMin` values.

This is the first entry in the Optimizer dropdown.

### Airstream Length Calibrator

A sub-calibrator that adjusts only airstream length. Uses a 1D optimizer (Brent's method). Improves fmax prediction. Not listed in the Optimizer dropdown; the joint calibrator calls it internally.

### Beta Calibrator

A sub-calibrator that adjusts only the beta factor. Uses a 1D optimizer (Brent's method). Improves fmin prediction. Not listed in the Optimizer dropdown; the joint calibrator calls it internally.

## Available Optimizers

These appear in the Optimizer dropdown. The first is the calibrator; the rest require constraints.

### Hole optimizers

| Optimizer | Description |
|-----------|-------------|
| **Flute calibration** | Joint calibrator. Adjusts airstream length + beta factor. No constraints needed. |
| **Hole position & size** | Optimizes both hole diameters and spacings. |
| **Hole position only** | Optimizes only hole spacings, keeping diameters fixed. |
| **Hole size only** | Optimizes only hole diameters, keeping positions fixed. |

### Global hole optimizers

Global optimizers use DIRECT-C for broad search followed by BOBYQA for local refinement. Enable DIRECT in Settings to make these available.

| Optimizer | Description |
|-----------|-------------|
| **Hole spacing (global)** | Global search over hole positions. |
| **Hole size+spacing (global)** | Global search over both hole positions and diameters. |

### Bore and stopper optimizers

| Optimizer | Description |
|-----------|-------------|
| **Stopper position** | Flute-specific. Adjusts the position of the stopper (cork) that seals the bore above the embouchure hole. A 1D optimizer. This is unique to the Flute model. |
| **Headjoint** | Optimizes stopper position plus bore diameters in the headjoint region (from the stopper to the first tone hole). |
| **Basic taper** | Optimizes a linear taper in the bore profile. |
| **Bore diameter from bottom** | Adjusts bore diameters at internal bore points, working from the bottom up. |
| **Bore spacing from top** | Adjusts the axial spacing of bore points from the top down. |

### Merged optimizers

Merged optimizers combine hole and bore optimization in a single run.

| Optimizer | Description |
|-----------|-------------|
| **Holes + bore diameter from bottom** | Hole optimization combined with bore diameter from bottom. |
| **Holes + bore spacing** | Hole optimization combined with bore spacing from top. |
| **Holes + basic taper** | Hole optimization combined with basic taper. |
| **Holes + headjoint** | Hole optimization combined with headjoint (stopper + bore diameters). |

### Global merged optimizers

| Optimizer | Description |
|-----------|-------------|
| **Holes + basic taper (global)** | Global search over holes and taper. |
| **Holes + bore dia from bottom (global)** | Global search over holes and bore diameters. |

## Stopper Position

The **stopper** (or cork) is a plug that seals the bore above the embouchure hole. Its position relative to the embouchure hole center affects tuning, particularly in the upper register. The Stopper Position optimizer adjusts this single parameter to improve tuning across all fingerings.

On a real instrument, you adjust the stopper by pushing or pulling the cork inside the headjoint. The optimizer tells you where to place it.

## Sample Files

- **Instruments**: `SamplePVC-Flute`, `fife`
- **Tunings**: `D4-Equal` and other tunings with optional measured min/max frequencies
- **Constraints**: Pre-configured bounds for six-hole flute optimization

## Calibration Workflow

1. Measure the physical dimensions of your flute: bore profile, hole positions and diameters, embouchure hole dimensions, and stopper position.

2. Create an instrument XML with these measurements. Set airstream length to an initial estimate (roughly the embouchure hole width) and beta factor to 0.4.

3. Measure actual playing frequencies. For calibration, also measure the maximum and minimum sustainable frequencies per fingering.

4. Create a tuning file with target, maximum, and minimum frequencies.

5. Select the instrument and tuning. Choose **Flute calibration** from the Optimizer dropdown. Click **Calibrate**.

6. The calibrator produces a new instrument with updated airstream length and beta factor. Evaluate to verify improvement.

## Typical Design Workflow

1. **Select the Flute study model** from the header dropdown.

2. **Open instrument and tuning files**. Select both.

3. **Evaluate** to see current accuracy.

4. **Calibrate** using the joint calibrator if you have measured min/max frequencies.

5. **Optimize the stopper** first if headjoint tuning is off. The stopper position optimizer is fast (1D) and can improve overall tuning before hole optimization.

6. **Optimize holes**. Select "Hole position & size" and appropriate constraints. Run the optimizer.

7. **Refine bore shape** if needed. Use a merged optimizer (e.g., "Holes + headjoint") to jointly optimize hole placement and bore profile.

8. **Save** the optimized instrument.


## See Also

- [Whistle](whistle.md) -- the Whistle model shares the same frequency prediction framework
- [Optimizer Overview](../optimizers/overview.md) -- how optimization algorithms work
- [Whistle & Flute Optimizers](../optimizers/whistle-flute.md) -- detailed guide to each optimizer
- [Constraints](../optimizers/constraints.md) -- creating and editing constraints files
- [Evaluation](../tools/evaluation.md) -- understanding evaluation results
- [Note Spectrum](../tools/note-spectrum.md) -- impedance spectrum analysis

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Working-with-the-Flute-Study-Model).*
