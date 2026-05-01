# Note Spectrum

[User Guide](../index.md) > [Tools] > Note Spectrum

The Note Spectrum tool shows the impedance spectrum for a single fingering, letting you see where the instrument resonates and whether those resonances are strong enough to sustain a note.

## Prerequisites

- An instrument file must be selected in the Study Panel.
- A tuning file must be selected in the Study Panel.

If no evaluation has been run yet, clicking Spectrum will automatically run one first.

## Opening the Spectrum

Click the **Spectrum** button in the toolbar. A popup window opens showing the impedance spectrum chart for the first fingering in the tuning. Your browser may need to allow popups for this site.


## Selecting a Fingering

Use the **Fingering** dropdown at the top of the popup to switch between notes. Each entry shows the note name and target frequency. The chart recomputes when you select a different fingering.

## Reading the Chart

The chart has two Y-axes and plots two quantities against frequency:

### Impedance Ratio (left Y-axis)

The gray line shows the reactance-to-resistance ratio, Im(Z)/Re(Z), across the frequency range. This is the imaginary part of the input impedance divided by the real part.

- **Resonances** occur where the impedance ratio crosses zero with a positive slope (going from negative to positive). These are the frequencies at which the air column naturally vibrates.
- **Anti-resonances** occur at the peaks and troughs of the curve.

### Loop Gain (right Y-axis)

The gain curve is drawn in two colors:

- **Green** segments indicate loop gain >= 1. The note will sustain at these frequencies -- the energy fed back by the mouthpiece exceeds losses.
- **Red** segments indicate loop gain < 1. The note will not sustain -- losses exceed feedback.

A dashed horizontal line marks gain = 1 for reference.

Loop gain is most meaningful for **Whistle** and **Flute** study models. For **NAF** and **Reed** models, the gain is shown as a constant value of 1 across all frequencies.

## Interpretation

The instrument plays at frequencies where two conditions are met simultaneously:

1. The impedance ratio is at or near a zero crossing (a resonance exists).
2. The loop gain is >= 1 (the mouthpiece can sustain oscillation at that frequency).

A note with a strong resonance (sharp zero crossing) and high gain (well above 1) will speak easily. A note where the gain barely reaches 1 may be difficult to sustain or require more breath pressure.

## Frequency Range

The chart spans from near zero up to the target frequency multiplied by a spectrum multiplier. The default multiplier is 3.17, which shows approximately three harmonics above the fundamental. You can change this multiplier in the Settings dialog.

## Tonehole Lattice Cutoff

At higher frequencies, you may notice a sudden reduction in the amplitude of the impedance peaks and troughs. This corresponds to the **tonehole lattice cutoff frequency** -- the frequency above which sound propagates past the first open tonehole rather than being reflected back up the bore. Above this cutoff, the instrument behaves less like a resonant tube and more like an open pipe.

## See Also

- [Evaluation](evaluation.md) -- view the tuning table with predicted frequencies and deviations
- [Supplementary Info](supplementary.md) -- view gain and Q factor values in tabular form
- [Settings](../settings.md) -- change the spectrum multiplier

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Reading-the-Note-Spectrum-Graph).*
