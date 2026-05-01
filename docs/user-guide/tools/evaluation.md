# Evaluation

[User Guide](../index.md) > [Tools] > Evaluation

The Evaluate tool compares the predicted playing frequencies of your instrument against the target frequencies defined in a tuning file. This is the primary way to assess how well an instrument matches a desired scale.

## Prerequisites

- An instrument file must be selected in the Study Panel.
- A tuning file must be selected in the Study Panel.
- The number of fingerings in the tuning must match the number of tone holes in the instrument.

If these conditions are not met, the Evaluate button in the toolbar is disabled.

## Running an Evaluation

Click the **Evaluate** button in the toolbar. The acoustic model compiles the instrument geometry, applies the physical parameters (temperature, humidity, pressure), and calculates the playing frequency for each fingering in the tuning.

Results open in an in-app tool panel by default. You can choose Java-style popup windows in Settings.


## Reading the Tuning Table

The result table shows one row per fingering. The columns are:

| Column | Description |
|--------|-------------|
| **Note** | The note name from the tuning file. |
| **Target (Hz)** | The desired frequency for this note, as specified in the tuning file. |
| **Predicted (Hz)** | The frequency the acoustic model predicts the instrument will produce with this fingering. |
| **Deviation (cents)** | The difference between predicted and target frequency, expressed in cents. Positive values mean the note is sharp; negative values mean it is flat. |
| **Weight** | The note's weight from the tuning file. Weighted notes contribute more to optimization and appear in the summary statistics. A weight of 0 means the note is excluded from summary calculations. |

Deviation values are color-coded:

- Green: less than 5 cents (typically inaudible)
- Yellow: 5 to 15 cents (noticeable to trained ears)
- Red: more than 15 cents (clearly out of tune)

## Summary Statistics

Below the table, two summary values appear:

- **Net Error**: The weighted mean of cent deviations across all notes with nonzero weight. A positive value means the instrument is generally sharp; negative means generally flat.
- **Mean Deviation**: The weighted mean of the absolute cent deviations. This measures overall tuning accuracy regardless of direction.

## Understanding Cents

Cents are a logarithmic unit for measuring pitch intervals. There are 100 cents in one equal-tempered semitone and 1200 cents in one octave. For practical instrument design:

- Differences under 5 cents are typically inaudible to most listeners.
- Differences of 5 to 15 cents are noticeable to trained musicians.
- Differences over 15 cents are clearly audible.

## Study Model Differences

- **NAF** and **Reed** models show only the nominal playing frequency for each fingering.
- **Whistle** and **Flute** playing-range data is available through **Graph** and **Supplementary**. The Evaluate table itself reports the nominal predicted frequency and cents deviation for each fingering.

## Note Weights

Notes can be assigned different weights in the tuning file. A higher weight means the note has more influence during optimization. Notes with a weight of 0 are evaluated but excluded from the summary statistics. This is useful for including reference fingerings that should not drive optimization.

## Graph Tuning

Click the **Graph** button in the toolbar to open an impedance pattern chart for all fingerings at once. This complements the tuning table by showing the acoustic behavior visually.

The chart plots the reactance ratio (X/R) against frequency, with one gray curve per fingering. Markers indicate key frequencies:

- **Green filled diamonds** -- target frequencies that fall within the predicted playing range (between fmin and fmax). These notes should play at the desired pitch under normal conditions.
- **Red filled diamonds** -- target frequencies that fall outside the predicted playing range. These notes may require extra effort to play in tune.
- **Green filled circles** -- fmax markers (the upper edge of each note's playing range).
- **Blue open circles** -- fmin markers (the lower edge of each note's playing range).

The Y-axis range is determined by the fmin and fmax marker values, with 10% padding. Curves extend beyond this range but are clipped to keep the markers visible.

Use Graph Tuning to spot notes where the target frequency sits far from the nearest resonance, or where the playing range is unusually narrow.


## See Also

- [Note Spectrum](note-spectrum.md) -- view the impedance spectrum for individual fingerings
- [Supplementary Info](supplementary.md) -- view additional acoustic properties per note
- [Optimization Workflow](../optimizers/workflow.md) -- use evaluation results to guide optimization

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Evaluating-Instruments-with-the-Tuning-Table-and-Tuning-Graph).*
