# Tuning Winds

[User Guide](../index.md) > [Reference] > Tuning Winds

Wind instruments inherently produce variable frequencies depending on playing technique. Understanding this variability is essential for setting realistic design goals in WIDesigner.

## Frequency Is Not Fixed

A single fingering on a wind instrument does not produce a single fixed frequency. Depending on air velocity, embouchure pressure, and other playing variables, the sounding pitch of any given note can vary by roughly 100 cents (a full semitone) or more. The target pitch for a note should fall somewhere within this achievable range under normal playing conditions.

## Fipple Flutes (NAFs and Whistles)

On fipple flutes, the sounding frequency generally increases with air velocity. As blowing pressure rises, the pitch sharpens until the instrument reaches a register break and jumps to a higher resonance mode. The player's primary pitch control is breath pressure, which means the instrument's geometry largely determines where in the playing range each note sits.

## Transverse Flutes and Reeds

Players of transverse flutes and reed instruments have additional techniques for controlling pitch beyond air velocity alone. Lip tension, embouchure shape, jaw position, and reed bite all affect the sounding frequency. This gives these players more latitude to correct tuning imperfections in an instrument's geometry, but it also means the instrument's design affects how much effort the player must exert to play in tune.

## Design Implications

Design choices directly affect the playing technique required for each note. If adjacent notes require very different blowing pressures to sound at the correct pitch, the instrument will feel uneven and difficult to play. A well-designed instrument allows consistent blowing pressure between neighboring notes, minimizing the physical adjustments the player must make.

Higher notes are particularly sensitive to bore geometry and hole placement. Small changes in dimensions can shift the playing range significantly for upper-register fingerings.

## Realistic Goals

Perfect pitch across all fingerings is impossible to guarantee through instrument geometry alone -- the player always contributes to the final sounding frequency. The realistic design goal is to make it as easy as possible for the player to sound each note in tune with normal technique. WIDesigner's optimization targets this goal by minimizing the deviation between predicted and target frequencies across all fingerings simultaneously.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Tuning-Winds).*
