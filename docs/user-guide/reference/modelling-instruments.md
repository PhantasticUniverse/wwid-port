# Modelling Instruments

[User Guide](../index.md) > [Reference] > Modelling Instruments

WIDesigner represents woodwind instruments at three levels of abstraction, each adding more specific information. Understanding these levels clarifies how the software organizes instrument data, tuning targets, and acoustic predictions.

## Level 1: Instrument Family

An instrument family defines a list of acoustic components and the distinct playing states they can assume. For example, a "whistle" family consists of a fipple mouthpiece, a cylindrical or tapered bore, and a set of tone holes. The playing states are typically discrete -- each hole is either open or closed. The family is represented as an ordered list of all possible fingering configurations.

At this level, no specific dimensions or target pitches are assigned. The family defines only the topology: what components exist and how many distinct states the instrument can produce.

## Level 2: Instrument Version

An instrument version is a specific member of a family with a defined target range. It is identified by its lowest note. For example, a "high-D whistle" is a version of the whistle family whose lowest note is D5.

At this level, each fingering configuration is mapped to a target note name within a tuning system. The version defines what notes the instrument should play, but not yet the physical dimensions that produce those notes.

## Level 3: Instrument Instance

An instrument instance is a specific instrument with known physical dimensions. It maps each fingering configuration to an actual playing frequency, computed from the geometry of the bore, tone holes, and mouthpiece.

At this level, WIDesigner can calculate the acoustic impedance of the bore for each fingering and predict the sounding frequency. It can also compute loop gain functions that characterize the playing range -- the span of frequencies the instrument can sustain for each fingering under varying blowing conditions.

An instrument instance may provide:

- **Predicted frequencies** for each fingering, derived from the impedance spectrum
- **Impedance functions** showing the resonator's response across a range of frequencies
- **Loop gain functions** indicating which frequencies are sustainable under normal playing conditions

These three levels correspond to the data files used in WIDesigner: the study model defines the family, the tuning file defines the version, and the instrument file defines the instance.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Modelling-Instruments).*
