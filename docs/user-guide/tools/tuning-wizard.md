# Tuning Wizard

[User Guide](../index.md) > [Tools] > Tuning Wizard

The Tuning Wizard generates a new tuning file by combining a temperament, scale parameters, and a fingering pattern. This saves you from manually calculating frequencies for each note when you want to create a tuning in a different key, temperament, or fingering layout.

## Prerequisites

None. The Wizard works independently of any loaded instruments, tunings, or other documents. You can open it at any time.

## Opening the Wizard

Click the **Wizard** button in the toolbar. A dialog opens with three steps, shown as a step indicator at the top.


---

## Step 1: Temperament

Choose the temperament that defines the interval ratios between notes.

**Built-in options:**

- **Equal Temperament (12-tone)** -- divides the octave into 12 equal semitones of 100 cents each. The standard tuning system for most modern Western instruments.
- **Just Intonation (12-tone)** -- uses pure frequency ratios based on the harmonic series. Produces purer intervals in specific keys but does not transpose equally to all keys.

**Loading a custom temperament:**

If neither built-in option suits your needs, you can load a temperament XML file. Click **Load temperament/pattern from file** and select your file. Once loaded, it appears as a third option in the temperament list.

Click **Next** to proceed to Step 2.

## Step 2: Scale

Configure the scale that maps the temperament intervals to specific note names and frequencies.

| Field | Description |
|-------|-------------|
| **Note symbols** | Choose **Sharps** (C, C#, D, ...) or **Flats** (C, Db, D, ...) for note naming. |
| **Reference note** | The note name used as the tuning anchor (default: A4). |
| **Reference freq (Hz)** | The frequency of the reference note (default: 440 Hz). All other note frequencies are calculated relative to this. |
| **Scale name** | A descriptive name for the generated scale (default: "Generated Scale"). |

Click **Generate Scale** to compute the full scale. The wizard calculates the absolute frequency for every note in the temperament, anchored to your reference note and frequency. On success, you advance to Step 3.

## Step 3: Tuning

Map the generated scale to a fingering pattern to produce a complete tuning file.

| Field | Description |
|-------|-------------|
| **Fingering pattern** | Select from loaded fingering pattern files. Each pattern defines which holes are open or closed for each note. If no patterns are loaded, use the **Load pattern from file** button to load one. |
| **Tuning name** | A descriptive name for the generated tuning (default: "Generated Tuning"). |

Click **Generate Tuning** to create the tuning. The wizard combines the scale frequencies with the fingering pattern to produce a complete tuning file. The dialog closes, and the new tuning appears in the Study Panel, ready for use with evaluation and optimization.

---

## Fingering Patterns

A fingering pattern defines the relationship between notes and hole configurations. Each entry in the pattern specifies:

- A note name (which maps to the scale to determine frequency)
- Which tone holes are open (1) or closed (0)

Sample fingering pattern files are included with the WIDesigner sample files. You can also create custom patterns as XML files.

## Custom Temperaments

A temperament file defines the interval ratios for each degree of the scale. The built-in Equal Temperament and Just Intonation cover most use cases, but you can load custom temperament XML files for historical tunings, microtonal systems, or ethnic scales.

## Tips

- To create a tuning in a different key, keep the same temperament and fingering pattern but change the reference note and frequency. For example, change from A4 = 440 Hz to D4 = 293.66 Hz.
- To compare temperaments, generate two tunings with the same reference note but different temperaments, then evaluate each against the same instrument to see how the cent deviations differ.
- The wizard does not modify the instrument. It only creates a tuning file that you can then use for evaluation or optimization.

## See Also

- [Evaluation](evaluation.md) -- evaluate the instrument against the generated tuning
- [Constraints](../optimizers/constraints.md) -- create constraints files for optimization
- [Sample Files](../sample-files.md) -- bundled fingering patterns and temperament files

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Using-the-Tuning-File-Wizard).*
