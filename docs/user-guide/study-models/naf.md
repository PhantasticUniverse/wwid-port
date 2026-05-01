# NAF (Native American Flute)

[User Guide](../index.md) > [Study Models] > NAF

The NAF study model is for designing and optimizing Native American Flutes. These are fipple flutes with a unique voicing mechanism: air from the slow air chamber passes through an external windway and strikes a splitting edge at the true sound hole (TSH), producing the tone. Unlike most Western flutes, the player does not blow directly into the sound hole -- the bird or block directs the airstream.

WIDesigner models the NAF mouthpiece as a set of measurable physical dimensions plus a single calibration parameter called the **fipple factor**. The acoustic model predicts one playing frequency per fingering (not min/max ranges as in the Whistle and Flute models).


## Prerequisites

- [Getting Started](../getting-started.md) -- loading files and running your first evaluation
- [Sample Files](../sample-files.md) -- bundled NAF instruments, tunings, and constraints

## Mouthpiece Parameters

The NAF instrument definition includes these mouthpiece fields:

| Parameter | Description |
|-----------|-------------|
| **Fipple factor** | Dimensionless calibration parameter. Accounts for the effective length added by the fipple voicing mechanism. Determined by calibration against measured frequencies. |
| **Windway height** | Height of the windway channel (metres). If the instrument has a windway height, the fipple factor is scaled by it internally. |
| **Flue depth** | Depth of the flue opening where air enters the TSH area (metres). |
| **TSH length** | Length of the true sound hole along the bore axis (metres). |
| **TSH width** | Width of the true sound hole perpendicular to the bore axis (metres). |
| **Window length** | Length of the fipple window (metres). |
| **Window width** | Width of the fipple window (metres). |
| **Splitting-edge position** | Position of the splitting edge relative to the bore, measured from the bore top (metres). |

You enter these in the instrument editor's Mouthpiece section. The only parameter you typically calibrate (rather than measure directly) is the fipple factor.

## Frequency Prediction

The NAF model predicts a single **nominal playing frequency** for each fingering. The evaluation table shows:

- **Target frequency** -- the desired pitch from your tuning file
- **Predicted frequency** -- what the model calculates for the current geometry
- **Cents deviation** -- difference between predicted and target, in cents

There are no minimum/maximum frequency predictions. If you need playing range analysis, use the Whistle or Flute model instead.

## Calibration

Before optimizing hole placement, you should calibrate the fipple factor so the model accurately predicts your instrument's existing tuning.

### Fipple Factor Calibrator

Select **Fipple factor** from the Optimizer dropdown. The Optimize button changes to **Calibrate**.

The calibrator works as follows:

1. It uses only the **lowest note** in the tuning (the first fingering by frequency).
2. It runs a 1D optimizer (Brent's method) to find the fipple factor value that minimizes the cents deviation for that note.
3. It produces a new instrument with the updated fipple factor. The original instrument is unchanged.

For best results:

- Calibrate on a **no-hole instrument** first (a simple tube with no finger holes). This isolates the mouthpiece behavior from hole interactions.
- Measure the actual playing frequency of the lowest note carefully. Enter it as the target frequency in your tuning file.
- After calibration, evaluate the full tuning to verify the model's predictions across all fingerings.
- The calibrated fipple factor value is shown in the console output (initial and final values, plus the optimization norm).

You do not need constraints for calibration -- the calibrator uses built-in bounds.

## Available Optimizers

Select an optimizer from the Optimizer dropdown in the Study Panel. The first entry is the calibrator; the rest require a constraints file.

| Optimizer | Description |
|-----------|-------------|
| **Fipple factor** | Calibrator. Adjusts fipple factor to match measured frequency. No constraints needed. |
| **Grouped-hole position & size** | Optimizes hole positions and sizes, treating adjacent holes as groups with equal spacing. |
| **Hole size & position** | Optimizes both hole diameters and positions (from the top of the bore). |
| **Hole size only** | Optimizes only hole diameters, keeping positions fixed. |
| **Taper, grouped-hole** | Optimizes a single-taper bore profile together with grouped hole positions and sizes. |
| **Taper, grouped-hole, hemispherical** | Same as above but assumes hemispherical head geometry. |
| **Taper, no hole grouping** | Optimizes a single-taper bore profile with individual (ungrouped) hole positions and sizes. |
| **Taper, no hole grouping, hemispherical** | Same as above but assumes hemispherical head geometry. |

Taper optimizers adjust the bore shape in addition to hole geometry. The "hemispherical" variants model the head of the bore as a hemisphere rather than a flat termination, which is appropriate for flutes with rounded or carved head chambers.

## Sample Files

WIDesigner ships with NAF sample files you can use as starting points:

- **Instruments**: `0.625-bore` and `1.00-bore` starters (simple NAF geometries at two common bore sizes)
- **Tunings**: F#4 and A4 pentatonic tunings with target frequencies
- **Fingering patterns**: Standard NAF fingering patterns
- **Constraints**: Pre-configured bounds for hole and taper optimizers

Open these from your local files using the Open button or drag and drop.

## Typical Workflow

1. **Select the NAF study model** from the dropdown in the header bar. This clears any previously loaded documents.

2. **Open an instrument file** (XML). It appears in the Instruments list in the Study Panel. Select it.

3. **Open a tuning file** (XML). It appears in the Tunings list. Select it. The tuning must have the same number of holes as the instrument.

4. **Evaluate** to see the current tuning accuracy. Click Evaluate in the toolbar. A popup window shows predicted vs. target frequencies and cents deviation for each fingering.

5. **Calibrate the fipple factor** if the model's predictions are inaccurate. Select "Fipple factor" from the Optimizer dropdown, then click Calibrate. The result is a new instrument with the adjusted fipple factor. Select the new instrument and re-evaluate.

6. **Select an optimizer and constraints**. Choose a hole or taper optimizer from the Optimizer dropdown. Open or create a constraints file and select it. You can generate default constraints (with pre-populated bounds) using the "Default Constraints" button.

7. **Optimize**. Click Optimize. Progress appears in the console. When complete, a new instrument is added to the Instruments list. Select it and evaluate to verify the result.

8. **Save** the optimized instrument. Click Save to download the instrument XML.


## See Also

- [Optimizer Overview](../optimizers/overview.md) -- how optimization algorithms work
- [NAF Optimizers](../optimizers/naf.md) -- detailed guide to each NAF optimizer
- [Constraints](../optimizers/constraints.md) -- creating and editing constraints files
- [NAF Design](../reference/naf-design.md) -- design principles for Native American Flutes
- [Evaluation](../tools/evaluation.md) -- understanding evaluation results
- [Sketch & Compare](../tools/sketch-and-compare.md) -- visualizing instrument geometry

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/NAF-GUI---First-Steps).*
