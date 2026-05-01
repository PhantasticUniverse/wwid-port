# Glossary

[User Guide](../index.md) > [Reference] > Glossary

This glossary defines the acoustic, instrument design, and software terms used throughout WIDesigner. Terms are organized by category.

## Instrument Terms

**Airstream Height (Flutes)** -- The thickness of the air stream from the player's lips. Typically about 1.5 mm.

**Airstream Length (Flutes)** -- The distance from the player's lips to the far edge of the embouchure hole. This parameter is adjusted during calibration to match the flute's measured tuning behavior.

**Alpha (Reeds)** -- A calibration parameter for reed instruments, expressed in milliseconds. Alpha characterizes the reed's response time and is determined through calibration against a measured instrument.

**Beta Factor (Whistles, Flutes, Reeds)** -- A dimensionless loop gain parameter that models the energy feedback in the sound-production mechanism. Typically starts at 0.4. Adjusted during calibration.

**Bore Point** -- A location along the instrument's bore where the interior diameter is measured. WIDesigner assumes a uniform taper (linear interpolation) between adjacent bore points.

**Bore Point Diameter** -- The inside diameter of the bore at a bore point.

**Crow Frequency (Reeds)** -- The frequency produced by the bare reed and staple assembly, without the instrument body attached. Not used by WIDesigner's acoustic models; included here for reference since it appears in reed instrument literature.

**Embouchure Hole Height (Flutes)** -- The body wall thickness at the embouchure hole.

**Embouchure Hole Length (Flutes)** -- The dimension of the embouchure hole measured along the bore direction (toward the foot of the instrument).

**Embouchure Hole Width (Flutes)** -- The dimension of the embouchure hole measured perpendicular to the bore direction (side to side).

**Fipple Factor (NAFs)** -- A dimensionless calibration parameter specific to Native American Flutes. It accounts for end-correction effects at the fipple window that are difficult to measure directly. Determined through calibration against a real instrument.

**Flue Depth (NAFs)** -- The body wall thickness at the window (True Sound Hole) location.

**Hole Height** -- The body wall thickness at a tone hole, including any external flange or chimney.

**Mouthpiece Position** -- The reference location of the sound-production mechanism along the bore. For whistles, this is the sound blade (labium). For flutes, it is the center of the embouchure hole. For reeds, it is the bottom of the staple.

**Position** -- The distance along the bore measured from the mouthpiece reference point toward the far (open) end of the instrument.

**Splitting Edge Position (NAFs)** -- The location of the sound blade at the lower end of the True Sound Hole.

**Termination Flange Diameter** -- The outer diameter of the instrument body at the open end (bore bottom). Affects the radiation impedance and end correction.

**TSH (NAFs)** -- True Sound Hole. The opening in a Native American Flute where the air stream from the flue strikes the splitting edge to produce sound.

**TSH Length (NAFs)** -- The distance from the windway exit to the splitting edge, measured along the bore direction.

**TSH Width (NAFs)** -- The side-to-side dimension of the True Sound Hole window.

**Window Height (Whistles)** -- The body wall thickness at the window opening.

**Window Length (Whistles)** -- The distance from the windway exit to the labium (sound blade), measured along the bore direction.

**Window Width (Whistles)** -- The side-to-side dimension of the window opening.

**Windway Length (Whistles)** -- The distance from the blowing end to the window. Not used by WIDesigner's acoustic models; included for completeness.

**Windway Height (NAFs, Whistles)** -- The height of the windway channel through which air flows toward the splitting edge.

## Tuning and Acoustics Terms

**Cents** -- A unit of frequency difference. 100 cents equals one semitone (one half step). Calculated as 1200 * log2(f1 / f2). A difference of less than about 5 cents is typically inaudible to most listeners.

**Fingering** -- The pattern of open and closed tone holes that produces a particular note. A fingering may also specify overblowing (exciting a higher resonance mode of the bore).

**Frequency** -- The number of vibration cycles per second, measured in Hertz (Hz). Frequency doubles with each octave. The standard concert pitch reference is A4 = 440 Hz.

**Impedance (Z)** -- The acoustic impedance of the resonator, describing how the air column responds to sound waves. Impedance has two components: reactance (X), representing stored energy, and resistance (R), representing dissipated energy. WIDesigner commonly displays the ratio X/R (reactance over resistance) in analysis tools like Note Spectrum and Graph Tuning.

## Software Terms

**XML** -- eXtended Markup Language. A structured text format used by WIDesigner for instrument data, tuning definitions, fingering patterns, and optimization constraints. All WIDesigner data files are XML documents that can be opened in any text editor.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Glossary).*
