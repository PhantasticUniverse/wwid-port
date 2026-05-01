# Reed Instruments: Didgeridoos

[User Guide](../index.md) > [Reference] > Reed & Didgeridoos

A didgeridoo is a lip reed instrument with no tone holes. WIDesigner can tune both the fundamental pitch and overtones (sometimes called "trumpet" notes) by optimizing the bore profile. This page walks through the sample files and optimization approach.

## Overview

Unlike most instruments in WIDesigner, a didgeridoo has no tone holes to adjust. Tuning is controlled entirely through the bore profile -- the sequence of diameters along the instrument's length. The Reed study model in WIDesigner handles this case using the Bore Position optimizer, which adjusts bore point positions and diameters within constraint bounds.

## Sample Files

Two didgeridoo sample files are bundled with the web app at `/samples/ReedStudy/`:

- **Didgeridoo-2stage-D2-D3.xml** -- A two-section didgeridoo with cylindrical bore segments of 1.312" and 2.125" diameter. A simple starting geometry for optimization.
- **Didgeridoo-D2-D3-tuning.xml** -- Defines two target notes: D2 as the fundamental and D3 as the trumpet note (first overtone, one octave above).

Additional files are available in the [original WIDesigner release package](https://github.com/edwardkort/WWIDesigner) under `ReedStudy/`:

- **Didgeridoo-3stage-D2-D3.xml** -- A three-section didgeridoo with three cylindrical bore segments. The additional section gives the optimizer more flexibility.
- **DidgeridooConstraints-2stage.xml** -- Constraints for the two-section design (5 bore points).
- **DidgeridooConstraints-3stage.xml** -- Constraints for the three-section design (7 bore points).

## Optimization Approach

1. In the web app, select the **Reed** study model from the dropdown in the header bar.
2. Open the tuning file, an instrument file, and the matching constraint file.
3. Select all three in the Study Panel.
4. Choose the **Bore Position** optimizer from the optimizer list.
5. Click **Optimize**.

The optimizer adjusts the bore point positions and diameters to minimize the deviation between the predicted and target frequencies for both the fundamental and the trumpet note.

A redundant bore point placed 1" from the open end is included in the sample instrument files. This point gives the optimizer an additional degree of freedom to fine-tune the radiation behavior at the bell without moving the instrument's physical endpoint.

## Design Notes

Didgeridoo design is primarily about controlling the relationship between the fundamental and overtones through bore geometry. A simple cylindrical bore produces overtones that are not harmonically related to the fundamental. By introducing diameter changes along the length, the bore profile can be shaped so that the trumpet note falls at a musically useful interval (typically one octave above the fundamental).

The two-stage design is a good starting point. If the optimizer cannot achieve acceptable tuning with two sections, the three-stage design provides additional flexibility at the cost of a more complex bore to construct.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Reed-Instrument-Example:-Didgeridoos).*
