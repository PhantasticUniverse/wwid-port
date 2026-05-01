# Settings

[User Guide](index.md) > Settings

The Settings dialog controls physical parameters, display preferences, and optimizer behavior. Open it by clicking the gear icon in the top-right corner of the header bar. Press Escape or click outside the dialog to close it without saving.


## Length Type

**Dropdown** -- IN (default), mm, cm, m, ft.

Controls the units used for displaying dimensions in instrument editors and tool outputs (bore positions, hole diameters, etc.). This is a display preference only -- internally, all dimensions are stored in the instrument XML's native units.

## Temperature, C

**Number input** -- Default varies by study model.

The air temperature in degrees Celsius, used to calculate the speed of sound. The default depends on the active study model:

| Study Model | Default Temperature |
|---|---|
| NAF | 22.22 C (72 F) |
| Whistle | 27 C |
| Flute | 27 C |
| Reed | 27 C |

The speed of sound increases with temperature, so this setting directly affects predicted playing frequencies. If you are designing for a specific playing environment, set this to the expected ambient temperature.

## Relative Humidity, %

**Number input** (0--100) -- Default varies by study model.

The relative humidity percentage, used in air density calculations that affect the speed of sound.

| Study Model | Default Humidity |
|---|---|
| NAF | 45% |
| Whistle | 100% |
| Flute | 100% |
| Reed | 100% |

Whistle, Flute, and Reed models default to 100% because the air column inside a played instrument is nearly saturated with moisture from the player's breath. NAF uses a lower default reflecting typical ambient conditions for end-blown playing.

## Use DIRECT Optimizer

**Checkbox** -- Default: on.

When enabled, "Global" optimizers appear in the optimizer list in the Study Panel. Global optimizers use the DIRECT-C algorithm for an initial thorough search of the entire parameter space before refining with BOBYQA. This finds better solutions on complex landscapes but takes significantly longer to run.

When disabled, only the standard (local) optimizers are listed. Local optimizers use BOBYQA alone, starting from the current instrument geometry. They are faster but may miss the global optimum if the starting point is far from the best solution.

If you are unsure, leave this enabled. You can always choose a local optimizer from the list when you want a faster run.

## Max Note Spectrum Freq (Multiplier)

**Number input** -- Default: 3.17.

Controls the upper frequency bound when displaying Note Spectrum charts. The value is a multiplier applied to each note's fundamental frequency. The default of 3.17 captures approximately up to the third harmonic.

Increase this value to see higher harmonics in the spectrum plots. Decrease it to focus on the fundamental and first overtone.

## Tool Output

**Dropdown** -- In app (default), Popup.

Controls where analysis tools display their results. **In app** opens results in a docked modal panel, which works even when browser popup blockers are enabled. **Popup** uses Java-style separate browser windows.

## Confirm Study Switch

**Checkbox** -- Default: on.

When enabled, WIDesigner asks before switching study model if documents are loaded. Switching study model clears the current session because each model has different optimizers and physical defaults.

## Parameters Not Exposed in Settings

Two physical parameters are used internally but are not shown in the Settings dialog:

- **Pressure** -- NAF uses 101.325 kPa (standard atmosphere). Whistle, Flute, and Reed use 98.4 kPa.
- **CO2 concentration** -- NAF uses 390 ppm (ambient). Whistle, Flute, and Reed use 40,000 ppm (reflecting elevated CO2 in the breath stream).

These values are set per study model and cannot be changed through the UI. They match the defaults used in the original WIDesigner desktop application.

## Saving and Canceling

Click **Apply** to save your changes and close the dialog. Settings are stored in your browser's localStorage and persist across sessions -- you do not need to re-enter them each time you open the application.

Click **Cancel** (or press Escape) to discard any changes and close the dialog.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/WIDesigner-Options).*
