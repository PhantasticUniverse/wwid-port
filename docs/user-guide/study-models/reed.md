# Reed

[User Guide](../index.md) > [Study Models] > Reed

The Reed study model is for designing and optimizing instruments driven by vibrating reeds -- single reeds (chanters, chalumeau-type instruments), double reeds (practice chanters, shawms, crumhorns), and lip reeds (didgeridoos, cornetts, serpents). The common thread is a tube with tone holes where the sound source is a reed or the player's lips vibrating against or inside a mouthpiece.

WIDesigner's Reed model does **not** handle valved or slide instruments (trumpets, trombones). It is designed for instruments where pitch is controlled by opening and closing tone holes along the bore.

Like the NAF model, the Reed model predicts a single **nominal playing frequency** per fingering (not min/max ranges).


## Prerequisites

- [Getting Started](../getting-started.md) -- loading files and running your first evaluation
- [Sample Files](../sample-files.md) -- bundled Reed instruments, tunings, and constraints

## Instrument Types

The Reed model can simulate:

- **Single reed instruments** -- chanters, practice chanters, chalumeau-type instruments
- **Double reed instruments** -- practice chanters with double reeds, shawms, crumhorns. For the instrument definition, describe the staple (the metal tube the reed sits on) and omit the reed itself.
- **Lip reed instruments** -- didgeridoos, cornetts, serpents. The player's lips function as the reed.

## Mouthpiece Geometry

Reed instrument definitions have a critical constraint: the **reed position must equal the position of the uppermost bore point**. The model treats the reed as sitting exactly at the top of the bore. If these values do not match, the instrument will fail validation.

For double reed instruments, enter the staple dimensions (the metal tube between the reed and the bore) as the mouthpiece geometry. Omit the reed blades -- they are accounted for by the calibration parameters.

For lip reed instruments (didgeridoos), there is no separate mouthpiece piece. The mouthpiece position still must match the top bore point.

## Calibration Parameters

The Reed model has two calibration parameters:

| Parameter | Description |
|-----------|-------------|
| **Alpha** | A millisecond-based parameter that models the reed's contribution to the effective bore length. The acoustic model computes the reed's length correction as `alpha * 0.001 * frequency + beta`. |
| **Beta** | A dimensionless offset in the reed length correction formula. |

Together, alpha and beta characterize how the reed (or lip buzz) shifts the effective resonant frequencies of the bore. Once calibrated for a specific mouthpiece, these values stay constant across different instruments that use the same mouthpiece.

Both parameters appear in the instrument editor's Mouthpiece section.

## Calibrator

The Reed model has one calibrator.

### Reed Calibrator

Select **Reed calibrator** from the Optimizer dropdown. The Optimize button changes to **Calibrate**. No constraints are needed.

The calibrator adjusts **both** alpha and beta simultaneously using a 2D optimizer (BOBYQA). It minimizes the cents deviation between predicted and target frequencies across all fingerings.

Unlike the Whistle and Flute calibrators (which use min/max frequency analysis), the Reed calibrator works only with nominal target frequencies. This is because the Reed model does not predict playing ranges.

For best results:

- Calibrate using an instrument with known, accurately measured playing frequencies.
- Use a tuning file where every fingering has a target frequency.
- After calibration, the console shows the initial and final alpha, beta, and optimization norm values.
- Transfer the calibrated alpha and beta to other instruments that share the same mouthpiece.

## Available Optimizers

These appear in the Optimizer dropdown. The first is the calibrator; the rest require constraints.

### Hole optimizers

| Optimizer | Description |
|-----------|-------------|
| **Reed calibrator** | Calibrator. Adjusts alpha + beta to match measured frequencies. No constraints needed. |
| **Hole position & size** | Optimizes both hole diameters and spacings. |
| **Hole position only** | Optimizes only hole spacings, keeping diameters fixed. |
| **Hole size only** | Optimizes only hole diameters, keeping positions fixed. |

### Global hole optimizer

Global optimizers use DIRECT-C for broad search followed by BOBYQA for local refinement. Enable DIRECT in Settings to make this available.

| Optimizer | Description |
|-----------|-------------|
| **Hole size+spacing (global)** | Global search over both hole positions and diameters. |

### Bore optimizers

| Optimizer | Description |
|-----------|-------------|
| **Bore diameter from bottom** | Adjusts bore diameters at internal bore points, working from the bottom up. |
| **Bore position** | Adjusts the axial positions of bore points. Uses a mixed parameterization: the first dimension is an absolute position, the rest are fractional spacings. |
| **Bore from bottom** | Adjusts both bore positions and diameters, working from the bottom up. A combined bore optimizer. |

### Merged optimizers

Merged optimizers combine hole and bore optimization in a single run.

| Optimizer | Description |
|-----------|-------------|
| **Holes + bore diameter from bottom** | Hole optimization combined with bore diameter from bottom. |
| **Holes + bore position** | Hole optimization combined with bore position adjustment. |
| **Holes + bore from bottom** | Hole optimization combined with bore from bottom (positions + diameters). |

### Global merged optimizer

| Optimizer | Description |
|-----------|-------------|
| **Holes + bore dia from bottom (global)** | Global search over holes and bore diameters. |

## Sample Files

- **Instruments**: `SampleChanter` (practice chanter with double reed), `Didgeridoo-2stage` (two-stage didgeridoo)
- **Tunings**: Chanter and didgeridoo tunings with target frequencies
- **Constraints**: Pre-configured bounds for reed instrument optimization

## Typical Workflow

1. **Select the Reed study model** from the header dropdown.

2. **Open an instrument file**. Verify that the mouthpiece position matches the top bore point (the app validates this and reports an error if they differ).

3. **Open a tuning file**. Select both the instrument and tuning.

4. **Evaluate** to see current tuning accuracy.

5. **Calibrate alpha and beta**. Select "Reed calibrator" from the Optimizer dropdown. Click Calibrate. The result is a new instrument with updated alpha and beta. Select it and re-evaluate.

6. **Select an optimizer + constraints**. Choose a hole or bore optimizer. Generate default constraints or load a constraints file.

7. **Optimize**. Review the result by evaluating the new instrument.

8. **Save** the optimized instrument.

## Didgeridoo Example

Didgeridoos are lip reed instruments with no finger holes. They use bore shape rather than tone holes to determine pitch. To work with a didgeridoo in WIDesigner:

- Set the number of holes to 0 in both the instrument and tuning files.
- The tuning file contains the target drone frequency (and optionally overtone targets).
- Calibrate alpha and beta for the lip-reed mouthpiece.
- Use the bore optimizers (Bore diameter from bottom, Bore position, or Bore from bottom) to shape the bore profile for your target tuning.

See the [Reed & Didgeridoos](../reference/reed-didgeridoos.md) reference page for more detail on didgeridoo modelling.


## See Also

- [Optimizer Overview](../optimizers/overview.md) -- how optimization algorithms work
- [Reed Optimizers](../optimizers/reed.md) -- detailed guide to each Reed optimizer
- [Constraints](../optimizers/constraints.md) -- creating and editing constraints files
- [Evaluation](../tools/evaluation.md) -- understanding evaluation results
- [Reed & Didgeridoos](../reference/reed-didgeridoos.md) -- design considerations for reed instruments

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Working-with-the-Reed-Study-Model).*
