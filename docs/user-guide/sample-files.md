# Sample Files

[User Guide](index.md) > Sample Files

This page documents the bundled sample files included with WIDesigner Web. These files provide working examples for all four study models and are a good starting point for learning the application and experimenting with instrument design.

## How to Access the Sample Files

The sample files are served at `/samples/` when the app is running. You can download them by navigating to the file URL directly in your browser (e.g., `/samples/NafStudy/0.625-bore_6-hole_NAF_starter.xml`), or by right-clicking the links below and saving.

To load a sample file into WIDesigner, click the **Open File** button in the header bar and select the downloaded XML file from your computer. You can also use your own XML files -- the format is the same.

## File Types

WIDesigner uses four types of XML files. Each type serves a distinct role in the evaluation and optimization workflow.

- **Instrument files** (`.xml`, `<instrument>` root): Describe the physical geometry of a woodwind instrument -- bore profile (position/diameter pairs), tone hole positions, sizes, and heights, and mouthpiece parameters (fipple, embouchure hole, or reed dimensions). This is what the optimizer modifies.

- **Tuning files** (`.xml`, `<tuning>` root): Define target frequencies and fingerings for each note. Each entry specifies a note name, the target frequency in Hz, the fingering pattern (which holes are open or closed), and an optional optimization weight. This is what the optimizer tunes toward.

- **Fingering files** (`.xml`, `<fingeringPattern>` root): Define just the fingering patterns (open/closed holes for each note) without target frequencies. These are used with the Tuning Wizard to generate tuning files from a scale and temperament.

- **Constraints files** (`.xml`, `<constraints>` root): Set upper and lower bounds on instrument dimensions for optimization. Each constraint specifies a dimension category (e.g., hole position, hole size, bore diameter), its current value, and the allowed range. The optimizer will not move dimensions outside these bounds.

---

## NafStudy -- Native American Flutes

Sample files for the NAF study model, located at `/samples/NafStudy/`.

### Instruments

| File | Description |
|------|-------------|
| `0.625-bore_6-hole_NAF_starter.xml` | Starter NAF with 0.625" bore diameter and 6 tone holes. A good baseline for learning hole position optimization. |
| `1.00-bore_6-hole_NAF_starter.xml` | Larger bore NAF with 1.00" bore diameter and 6 tone holes. Produces a richer, deeper tone than the 0.625" bore. |

### Tunings

| File | Description |
|------|-------------|
| `F#4_ET_6-hole_NAF_chromatic_tuning.xml` | Equal temperament tuning in F#4 for a 6-hole NAF. Includes chromatic fingerings and second-octave notes. |
| `A4_ET_6-hole_NAF_chromatic_tuning.xml` | Equal temperament tuning in A4 for a 6-hole NAF. Higher key, same chromatic fingering layout. |

### Fingerings

| File | Description |
|------|-------------|
| `Wood_Wind_NAF_6-hole_fingering.xml` | Standard 6-hole NAF fingering pattern from Edward Kort's Wood Wind system. Includes chromatic notes and three second-octave notes. Use with the Tuning Wizard to generate tunings in any key. |

### Constraints

| File | Description |
|------|-------------|
| `NAF_HoleFromTop_constraints.xml` | Constraints for hole-from-top position optimization. Sets maximum hole spacing of 1.25" and bounds on bore length. Designed for 6-hole NAFs. |

---

## WhistleStudy -- Tin Whistles

Sample files for the Whistle study model, located at `/samples/WhistleStudy/`.

### Instruments

| File | Description |
|------|-------------|
| `SamplePVC-Whistle.xml` | Simple PVC whistle design. Good starter instrument for learning whistle optimization. |
| `FeadogMk1.xml` | Measurement of a commercial Feadog Mk 1 high D whistle (6 holes, 12mm bore). Useful as a reference for comparing predicted vs. measured tuning. |

### Tunings

| File | Description |
|------|-------------|
| `D5-Equal.xml` | High D whistle tuning in equal temperament. Standard 6-hole diatonic scale with second-octave notes. |
| `FeadogMk1-tuning.xml` | Measured tuning for the Feadog Mk 1 whistle in Just Intonation, including measured minimum and maximum frequencies for playing range analysis. |

### Fingerings

| File | Description |
|------|-------------|
| `diatonic_whistle_fingering.xml` | Standard diatonic whistle fingering pattern. Two-octave major scale with cross-fingered C-natural. Use with the Tuning Wizard. |

### Constraints

| File | Description |
|------|-------------|
| `Whistle_Hole_constraints.xml` | Default hole position and size constraints for 6-hole whistle optimization. |

---

## FluteStudy -- Transverse Flutes

Sample files for the Flute study model, located at `/samples/FluteStudy/`.

### Instruments

| File | Description |
|------|-------------|
| `SamplePVC-Flute.xml` | PVC transverse flute design. Simple cylindrical bore with embouchure hole. Good starter for flute optimization. |
| `fife.xml` | Bb fife: a 6-hole cylindrical bore maple fife with embouchure hole. Measurements in millimeters. |

### Tunings

| File | Description |
|------|-------------|
| `D4-Equal.xml` | Generic 6-hole keyless flute tuning in D4, equal temperament. |
| `fife-tuning.xml` | Measured tuning for the Bb fife, including measured frequencies. Note that embouchure and mouth position were not necessarily constant during measurement. |

### Constraints

| File | Description |
|------|-------------|
| `Flute_Hole_constraints.xml` | Hole position and size constraints for 6-hole flute optimization. Limits modelled on a Pratten flute with large hole sizes and spacing. |

---

## ReedStudy -- Reed Instruments

Sample files for the Reed study model, located at `/samples/ReedStudy/`.

### Instruments

| File | Description |
|------|-------------|
| `SampleChanter.xml` | CPVC and brass smallpipe chanter in A, based on the Eric Reiswig design. Double reed, 8 holes (including thumbhole), closed fingering. |
| `Didgeridoo-2stage-D2-D3.xml` | Two-stage stepped bore didgeridoo with a lip reed mouthpiece. Tuned with a D2 fundamental and D3 first trumpet note. Zero tone holes. |

### Tunings

| File | Description |
|------|-------------|
| `A3-ClosedFingering.xml` | Bagpipe chanter tuning in A3 with closed fingering and thumbhole. Just Intonation tuning (A4=440 Hz), 8 holes. Tonic D3, lowest note G3. |
| `Didgeridoo-D2-D3-tuning.xml` | Didgeridoo tuning targeting D2 fundamental (73.42 Hz) and D3 first trumpet note. Zero holes. |

### Fingerings

| File | Description |
|------|-------------|
| `SmallpipeClosedFingering.xml` | Smallpipe closed fingering pattern with thumbhole, 8 holes. Use with the Tuning Wizard to generate chanter tunings in different keys. |

### Constraints

| File | Description |
|------|-------------|
| `Reed_Hole_constraints.xml` | Hole position and size constraints for reed chanter optimization. |

---

## Tips

- You can load multiple files at once using the Open File button. WIDesigner detects the file type automatically from the XML root element.
- Instrument and tuning files must have the same number of holes to be used together for evaluation or optimization.
- Constraints files are tied to a specific optimizer. The `objectiveFunctionName` element in the constraints XML must match the optimizer you select. Use the "+ Default" button in the Study Panel to generate constraints that match the currently selected optimizer.
- You can create your own XML files by editing the samples or by clicking "+ Blank" in the Study Panel and filling in values in the editor.

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki).*
