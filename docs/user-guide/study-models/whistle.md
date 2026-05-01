# Whistle (Tin Whistle)

[User Guide](../index.md) > [Study Models] > Whistle

The Whistle study model is for designing and optimizing tin whistles and similar fipple-driven instruments with six finger holes. Unlike the NAF model, the Whistle model predicts not just a nominal playing frequency but also the **maximum** and **minimum** sustainable pitches for each fingering. This playing range analysis helps you design instruments that are stable across different blowing pressures.


## Prerequisites

- [Getting Started](../getting-started.md) -- loading files and running your first evaluation
- [Sample Files](../sample-files.md) -- bundled Whistle instruments, tunings, and constraints

## Frequency Types

The Whistle model works with four frequency values per fingering:

| Frequency | Description |
|-----------|-------------|
| **Target** | The desired pitch, entered in the tuning file. |
| **Predicted (nominal)** | The pitch the model predicts under normal playing conditions. |
| **Maximum (fmax)** | The highest pitch the player can sustain before the note jumps to a higher register (overblows). |
| **Minimum (fmin)** | The lowest pitch the player can hold before the note drops out or jumps to a lower register. |

The evaluation table shows all four values along with cents deviations. A well-designed whistle has the target frequency comfortably between fmin and fmax for every fingering.

Tuning files for Whistle instruments can include `frequencyMax` and `frequencyMin` fields in addition to the standard `frequency` field. If you have measured these on a physical instrument, enter them so the calibrators can refine the model.

## Calibration Parameters

The Whistle mouthpiece has two calibration parameters beyond its physical dimensions:

| Parameter | Description | Starting value |
|-----------|-------------|----------------|
| **Window height** | The physical height of the fipple window opening (metres). Refined by the Window Height calibrator to improve fmax prediction. | Measure from the instrument. |
| **Beta factor** | A dimensionless parameter that controls fmin prediction. Represents the fraction of the window area effectively open to airflow. | ~0.4 is a reasonable starting point. |

Both parameters appear in the instrument editor's Mouthpiece section.

## Calibrators

The Whistle model has three calibrators. Select one from the Optimizer dropdown; the Optimize button changes to **Calibrate**. No constraints are needed.

### Whistle Calibration (Joint)

The recommended calibrator. Adjusts **both** window height and beta factor simultaneously using a 2D optimizer (BOBYQA). Requires tuning data with measured `frequencyMax` and `frequencyMin` values. Minimizes combined fmax and fmin deviations.

This is the first entry in the Optimizer dropdown.

### Window Height Calibrator

A sub-calibrator that adjusts only window height. Uses a 1D optimizer (Brent's method). Improves fmax prediction. Useful when you have measured maximum frequencies but not minimum frequencies.

This calibrator is not listed in the Optimizer dropdown. The joint calibrator calls it internally when needed.

### Beta Calibrator

A sub-calibrator that adjusts only the beta factor. Uses a 1D optimizer (Brent's method). Improves fmin prediction. Useful when you have measured minimum frequencies but not maximum frequencies.

This calibrator is not listed in the Optimizer dropdown. The joint calibrator calls it internally when needed.

## Available Optimizers

These appear in the Optimizer dropdown. The first is the calibrator; the rest require constraints.

### Hole optimizers

| Optimizer | Description |
|-----------|-------------|
| **Whistle calibration** | Joint calibrator. Adjusts window height + beta factor. No constraints needed. |
| **Hole position & size** | Optimizes both hole diameters and spacings. The most common starting optimizer. |
| **Hole position only** | Optimizes only hole spacings, keeping diameters fixed. |
| **Hole size only** | Optimizes only hole diameters, keeping positions fixed. |

### Global hole optimizers

Global optimizers use the DIRECT-C algorithm for broad search followed by BOBYQA for local refinement. They explore a wider solution space but take longer. Enable DIRECT in Settings to make these available.

| Optimizer | Description |
|-----------|-------------|
| **Hole spacing (global)** | Global search over hole positions. |
| **Hole size+spacing (global)** | Global search over both hole positions and diameters. |

### Bore optimizers

| Optimizer | Description |
|-----------|-------------|
| **Basic taper** | Optimizes a linear taper in the bore profile (2 parameters). |
| **Bore diameter from top** | Adjusts bore diameters at internal bore points, working from the top down. |
| **Bore diameter from bottom** | Adjusts bore diameters at internal bore points, working from the bottom up. |
| **Bore spacing from top** | Adjusts the axial spacing of bore points from the top down. |

### Merged optimizers

Merged optimizers combine hole and bore optimization in a single run. They alternate between hole and bore sub-optimizers, converging on a joint solution.

| Optimizer | Description |
|-----------|-------------|
| **Holes + basic taper** | Hole position & size combined with basic taper. |
| **Holes + bore diameter from top** | Hole optimization combined with bore diameter from top. |
| **Holes + bore diameter from bottom** | Hole optimization combined with bore diameter from bottom. |
| **Holes + bore spacing** | Hole optimization combined with bore spacing from top. |

### Global merged optimizers

| Optimizer | Description |
|-----------|-------------|
| **Holes + basic taper (global)** | Global search over holes and taper. |
| **Holes + bore dia from bottom (global)** | Global search over holes and bore diameters. |

## Sample Files

- **Instruments**: `SamplePVC-Whistle`, `FeadogMk1`
- **Tunings**: `D5-Equal` and other diatonic tunings with optional measured min/max frequencies
- **Constraints**: Pre-configured bounds for six-hole whistle optimization

## Calibration Workflow

1. Build or obtain a physical whistle. Measure its bore dimensions, hole positions, hole diameters, and mouthpiece geometry.

2. Create an instrument XML with these measurements. Enter the fipple window dimensions. Set beta factor to 0.4 as an initial estimate.

3. Measure the actual playing frequencies for each fingering. For best calibration, also measure:
   - **Maximum frequency**: blow harder until the note just overblows, record the pitch just before the break.
   - **Minimum frequency**: blow softer until the note just drops out, record the pitch just before it dies.

4. Create a tuning file with the target, maximum, and minimum frequencies.

5. Select the instrument and tuning. Choose **Whistle calibration** from the Optimizer dropdown. Click **Calibrate**.

6. The calibrator produces a new instrument with updated window height and beta factor. Select the new instrument and evaluate to verify that predicted, fmax, and fmin values are closer to the measured values.

7. Transfer the calibrated parameters to your working instrument file before proceeding to hole optimization.

## Typical Design Workflow

1. **Select the Whistle study model** from the header dropdown.

2. **Open instrument and tuning files**. Select both.

3. **Evaluate** to see current accuracy across all fingerings.

4. **Calibrate** if the instrument has measured frequencies. Use the joint calibrator to refine window height and beta.

5. **Select an optimizer + constraints**. For a first optimization, "Hole position & size" is the most common choice. Generate default constraints or load a constraints file.

6. **Optimize**. Review the new instrument's evaluation. If bore shape needs adjustment, follow up with a merged optimizer (holes + taper or holes + bore diameter).

7. **Save** the optimized instrument.


## See Also

- [Optimizer Overview](../optimizers/overview.md) -- how optimization algorithms work
- [Whistle & Flute Optimizers](../optimizers/whistle-flute.md) -- detailed guide to each optimizer
- [Constraints](../optimizers/constraints.md) -- creating and editing constraints files
- [Evaluation](../tools/evaluation.md) -- understanding evaluation results
- [Graph Tuning](../tools/evaluation.md#graph-tuning) -- visualizing impedance curves and playing ranges
- [Note Spectrum](../tools/note-spectrum.md) -- impedance spectrum analysis

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Working-With-the-Whistle-Study-Model).*
