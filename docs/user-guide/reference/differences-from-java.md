# Differences from the Java Desktop Application

[User Guide](../index.md) > [Reference] > Differences from Java

This page summarizes the notable differences between the WIDesigner web app and the original Java desktop application (WIDesigner v2.6.0). If you are migrating from the desktop version, this is a quick reference for what has changed, what is simplified, and what is identical.

## No Installation Required

The web app runs entirely in your browser. There is no Java runtime, JVM, or desktop installation needed. Any modern browser with WebAssembly support works (Chrome 57+, Firefox 52+, Safari 11+, Edge 16+). All computation runs locally in the browser -- no data is sent to a server.

## Interface Changes

| Java Desktop | Web App |
|---|---|
| Edit > Options to change study model | Dropdown in header bar |
| File > Open menu | Open File button + drag-and-drop |
| Tool menu | Toolbar row of buttons |
| Dockable/undockable panels (Study, Console) | Fixed layout panels |
| Internal JFrame windows for tool results | Browser popup windows |

The overall layout is the same -- Study Panel on the left, workspace in the center, console at the bottom -- but the web app uses a fixed layout rather than the Java dockable panel system.

## Simplified Settings

The Settings dialog (gear icon in the header bar) exposes:

- Temperature (degrees C)
- Relative humidity (%)
- Length display units
- DIRECT optimizer toggle
- Note Spectrum frequency multiplier

Parameters not exposed in the web app:

- **Pressure and CO2 concentration** -- set automatically per study model (matching Java defaults)
- **Blowing level** -- a Java-specific display setting, not applicable to the web UI
- **Constraints directory** -- not needed; create constraints via "+ Default" or "+ Blank" buttons in the Study Panel

## Tuning Wizard

The Tuning Wizard is simplified from 7 dialog steps in Java to 3 steps in the web app. The functionality is the same: select a scale, temperament, and fingering pattern to generate a tuning file. The streamlined interface removes intermediate confirmation screens.

## File Handling

The web app has no direct filesystem access. Key differences:

- **Opening files:** Use the Open File button or drag-and-drop files onto the app. There is no recent files list.
- **Saving files:** Save downloads the file to your browser's default download folder. There is no "Save As" dialog with directory navigation.
- **Constraints:** No constraints directory setting. Use the "+ Default" button in the Study Panel's Constraints section to create a default constraints file pre-populated with bounds for the selected optimizer, or "+ Blank" for an empty one.
- **Sample files:** Bundled at `/samples/` in the web app (was in the release package on disk in Java). See [Sample Files](../sample-files.md).

## Settings Persistence

Settings are stored in your browser's localStorage and persist across sessions. In the Java app, settings were stored in Java Preferences (OS-specific location).

## DIRECT Optimizer Toggle

The web app adds a "Use DIRECT Optimizer" checkbox in Settings. When enabled, Global optimizers (DIRECT + BOBYQA multi-start) appear in the optimizer list. When disabled, only local (BOBYQA-only) optimizers are shown. The Java app always showed all optimizers.

## Identical Acoustic Engine

The acoustic models, optimization algorithms, and evaluation precision are identical to the Java version:

- Same impedance calculations, transfer matrix formulations, and mouthpiece models
- Same optimizers: BOBYQA (local), DIRECT-C + BOBYQA (global), Brent (1D)
- Same evaluation precision: within 0.5 cents of the Java oracle per fingering
- Deterministic: identical inputs produce identical outputs
- 457 automated tests verify parity with the Java oracle across all four study models

If you have instrument and tuning files from the Java version, they will produce the same evaluation results in the web app (assuming the same temperature and humidity settings).

## Features Not Ported

The following Java desktop features are not available in the web app:

- **Dockable/undockable panels** -- The Study Panel and Console Panel have fixed positions.
- **Multi-tab constraint management** -- Java allowed switching between constraint sets per optimizer in a tabbed interface. The web app manages constraints as separate documents in the Study Panel.
- **Calibration spreadsheet (Reed)** -- Java's Reed study model included a spreadsheet view for bulk calibration data entry. Not ported.
