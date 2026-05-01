# Native American Flute Design and Construction

[User Guide](../index.md) > [Reference] > NAF Design

This page describes Edward Kort's five-stage process for designing and building a Native American Flute using WIDesigner. The process moves from initial specification through optimization, construction, measurement, and final tuning.

## Stage 1: Initial Specification

Before opening WIDesigner, several design decisions must be made:

- **Key selection.** NAFs are commonly built in keys from A2 to A5. The sweet spot for ease of construction and playing is F4 to Bb4. Lower keys require longer bores and larger diameters; higher keys demand tighter tolerances.

- **Sound mechanism and flue geometry.** The fipple (sound mechanism) design -- windway dimensions, window shape, splitting edge profile -- affects tone quality and response. These dimensions are typically determined by the builder's preferred construction technique.

- **Head space size.** The slow air chamber (head space) above the block affects back-pressure and response. Its volume is part of the bore profile.

- **Fingering pattern.** Most NAFs use a six-hole pentatonic pattern. Chromatic fingering patterns with additional holes are also possible. The fingering pattern determines the tuning file.

- **Bore diameter.** The bore diameter is a primary determinant of the instrument's voice and volume. Common starting diameters range from 0.625" to 1.0" depending on the target key.

## Stage 2: Pre-Construction Design

With the initial specification in hand, use WIDesigner to optimize the instrument's bore profile and hole layout:

1. Load a starter instrument file and the appropriate tuning file.
2. Load or create a constraints file that bounds the optimization dimensions.
3. Run the hole position optimizer to find hole locations that minimize tuning deviation.
4. Review the results, checking that the hole layout meets playability constraints.

**Hole layout guidelines:**

- Maximum spacing between holes within a triplet (top three or bottom three): 1.1" to 1.4"
- Minimum spacing between any two adjacent holes: 0.8"
- Maximum hole diameter: 0.45"

**Taper guidelines:**

- Minimize bore taper when possible. If taper is needed to achieve tuning targets, restrict it to the region near the bottom three holes.

**Target deviation:** Aim for no more than 5 cents deviation on any individual note after optimization.

If the optimizer cannot achieve acceptable results within playability constraints, revisit the bore diameter or key selection.

## Stage 3: Flute Blank Construction

Build the instrument body without drilling tone holes:

1. Construct the bore to the designed profile.
2. Install the fipple mechanism (block, windway, splitting edge).
3. Voice the instrument by adjusting the fipple until the fundamental tone speaks cleanly.
4. Measure the actual bore profile at multiple points along the length.
5. Determine the fipple parameters by calibrating the blank instrument. In WIDesigner, load the blank instrument (no holes) with a single-note tuning for the fundamental, and run calibration to find the fipple factor.

## Stage 4: As-Built Calculations

Update the instrument file with the actual measured dimensions:

1. Replace the designed bore profile with the measured bore profile.
2. Enter the calibrated fipple factor.
3. Re-run the hole position optimizer with the updated geometry.

The optimizer now works from the real instrument's bore rather than the idealized design, producing hole locations that account for any construction variations.

## Stage 5: Final Tuning

Drill and tune the tone holes:

1. Drill each hole undersized (smaller diameter than the optimizer's recommendation).
2. Run an evaluation to compare predicted and actual tuning.
3. Gradually enlarge each hole, checking tuning after each adjustment, until the desired pitch is reached.
4. Use WIDesigner's deviation calculations as a guide -- the predicted frequency for each fingering updates as you change hole dimensions in the instrument file.

Drilling undersized and enlarging incrementally prevents overshooting. Once a hole is too large, the only remedy is to fill and re-drill, which is difficult to do cleanly.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Native-American-Flute-Design-and-Construction).*
