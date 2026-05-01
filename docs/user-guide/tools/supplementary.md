# Supplementary Information

[User Guide](../index.md) > [Tools] > Supplementary Information

The Supplementary tool shows a table of calculated acoustic properties for each fingering. These values provide insight into how the instrument behaves at each note -- resonance quality, airflow requirements, and loop gain.

## Prerequisites

- An instrument file must be selected in the Study Panel.
- A tuning file must be selected in the Study Panel.

If these conditions are not met, the Supplementary button in the toolbar is disabled.

## Opening the Table

Click the **Supplementary** button in the toolbar. A popup window opens showing a table with one row per note in the tuning. Your browser may need to allow popups for this site.


## Table Columns

| Column | Description |
|--------|-------------|
| **Note** | The note name from the tuning file. |
| **Freq (Hz)** | The frequency used for calculations. For NAF models, this is the predicted frequency; for other models, it is the target frequency from the tuning file. |
| **Im(Z) Corr** | Impedance correction factor. This measures the imaginary impedance at the playing frequency. Useful for reed mouthpiece calibration -- if the value drifts significantly across the range, the reed model parameters may need adjustment. |
| **Air Speed (m/s)** | Estimated average air speed leaving the windway. This column appears only for study models that calculate air speed (Whistle and Flute). The value should increase smoothly from low notes to high notes. Sudden jumps may indicate a problematic bore design. |
| **Air Flow** | Estimated air flow rate in relative units. This indicates how much breath is needed to sustain each note. Higher values mean more air consumption, affecting how long a phrase can be sustained. This column appears only for study models that calculate air flow. |
| **Gain** | Loop gain at the note frequency. Notes with gain >= 1 (shown in green) will sustain; notes with gain < 1 (shown in red) will not speak or will be difficult to play. For NAF and Reed models, gain is always 1. |
| **Q Factor** | Resonance quality factor. Higher Q values indicate a sharper, more defined resonance peak. A note with high Q speaks clearly and has good pitch stability. Low Q values suggest a broad, weak resonance that may be hard to center. |

## Interpreting the Values

These quantities come from the acoustic model and are approximations, not direct physical measurements. Use them as relative guides:

- **Compare within a single instrument** rather than between different instruments. The absolute values depend on the model's assumptions.
- **Look for smooth trends** across the note range. Abrupt changes in air speed or gain often indicate a design issue worth investigating.
- **Gain below 1** for a note you expect to play is a warning. Consider adjusting bore geometry or hole placement.
- **Q factor** helps distinguish notes that will speak easily (high Q) from notes that will be unstable or breathy (low Q).

## See Also

- [Evaluation](evaluation.md) -- view predicted frequencies and cent deviations
- [Note Spectrum](note-spectrum.md) -- visualize the full impedance spectrum for a single fingering
- [Sketch & Compare](sketch-and-compare.md) -- view the instrument geometry

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/The-Supplementary-Information-Table).*
