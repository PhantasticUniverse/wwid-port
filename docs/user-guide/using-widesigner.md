# Using WIDesigner

[User Guide](index.md) > Using WIDesigner

This page describes the WIDesigner web interface and how to perform the core workflows: loading files, evaluating instruments, running optimizations, and using analysis tools. If you are new to WIDesigner, start here.

## The WIDesigner Window

The interface is divided into five areas:

**Header bar** -- The top row contains the "WIDesigner" title on the left, followed by a study model dropdown (NAF / Whistle / Flute / Reed). On the right side you will find the **Open File** button, a **Save** button, and a **Settings** gear icon.

**Toolbar row** -- Directly below the header. This row appears once the WASM engine has finished loading. It contains the following buttons, separated by dividers:

- Sketch | Compare
- Evaluate | Supplementary
- Graph | Spectrum
- Optimize (or Calibrate, depending on the selected optimizer) | Wizard

Buttons are disabled until the required documents are selected.

**Study Panel** (left sidebar) -- Lists all loaded documents in four sections: Instruments, Tunings, Optimizers, and Constraints. This is where you select which documents to work with.

**Workspace** (center) -- Tabbed editor area. When you open a document for editing, it appears here as a tab. You can have multiple tabs open at once.

**Console Panel** (bottom) -- Displays optimization progress messages, results, and any errors.


## Opening Files

WIDesigner works with XML files that describe instruments, tunings, and constraints. There are two ways to load files:

**Open File button** -- Click "Open File" in the header bar. The file picker accepts `.xml` files and allows multiple selection. Select one or more files and they will be loaded into the Study Panel.

**Drag and drop** -- Drag `.xml` files from your file manager and drop them anywhere on the application window. Each file is loaded automatically.

WIDesigner detects the file type from the XML content. Instrument files appear under "Instruments," tuning files appear under "Tunings," and constraints files appear under "Constraints." You do not need to specify the file type yourself.

When a file is loaded, it is automatically selected in the Study Panel.


## The Study Panel

The Study Panel on the left side of the window organizes your session into four sections:

**Instruments** -- Lists all loaded instrument definitions. Each entry shows the instrument name from the XML file.

**Tunings** -- Lists all loaded tuning definitions (fingering charts with target frequencies).

**Optimizers** -- Lists the available optimizers for the current study model. This list is populated automatically and changes when you switch study models. If "Use DIRECT optimizer" is enabled in Settings, additional "Global" optimizers appear in the list.

**Constraints** -- Lists all loaded constraint files. When an optimizer is selected, two buttons appear at the bottom of the Constraints section:
- **+ Default** creates a new constraints document with pre-populated bounds for the selected optimizer.
- **+ Blank** creates a new constraints document with empty bounds.

### Selection Model

**Single-click** an item to select it. The selected item is highlighted in the accent color. Selection determines which documents are used when you run Evaluate, Optimize, or the analysis tools.

**Double-click** an item to open it in an editor tab in the Workspace.

You can select one instrument, one tuning, one optimizer, and one constraints document at a time.


## Editing Documents

Double-click any document in the Study Panel to open it in an editor tab.

**Instrument editor** -- Shows the mouthpiece parameters at the top, followed by a bore profile table (position and diameter columns) and a tone holes table (position, diameter, and other hole parameters). You can add or remove bore points and holes using the row management controls.

**Tuning editor** -- Shows a list of notes with their fingering patterns and target frequencies.

**Constraints editor** -- Shows upper and lower bound arrays that define the search space for the optimizer.

Changes you make in an editor take effect immediately in the session. You do not need to save before running Evaluate or Optimize -- the current in-memory state is always used.

You can have multiple editor tabs open and switch between them by clicking the tab headers.


## Saving Files

Click the **Save** button in the header bar to download the currently active editor tab as an XML file. The browser will prompt you to save the file to your local filesystem.

The Save button is disabled when no editor tab is active. Only one document is saved at a time -- whichever tab is currently in the foreground.

## Study Models

The dropdown in the header bar lets you switch between four study models:

- **NAF** -- Native American Flute (end-blown, fipple)
- **Whistle** -- Tin whistle / pennywhistle (fipple)
- **Flute** -- Transverse flute (embouchure hole)
- **Reed** -- Single reed, double reed, or lip reed instruments

Switching the study model clears all loaded documents and resets the session. The optimizer list and available calibrators change to match the selected model. Physical parameter defaults (temperature, humidity) also differ between models -- see [Settings](settings.md) for details.

Choose the study model before loading your files.


## Evaluating Instruments

Evaluation calculates the predicted playing frequency for each fingering in the tuning, then compares it to the target frequency and reports the deviation in cents.

To evaluate:

1. Select an instrument in the Study Panel.
2. Select a tuning in the Study Panel. The tuning must have the same number of holes as the instrument.
3. Click the **Evaluate** button in the toolbar.

The results appear in a popup window showing a table with columns for note name, target frequency, predicted frequency, and deviation in cents.

If the Evaluate button is disabled, check that both an instrument and a compatible tuning are selected.


## Optimizing Instruments

Optimization adjusts instrument geometry (hole positions, hole sizes, bore dimensions, or combinations) to minimize tuning deviation. The optimizer searches within the bounds defined by the constraints document.

To optimize:

1. Select an instrument, tuning, optimizer, and constraints document.
2. Click the **Optimize** button in the toolbar.
3. A dialog appears showing progress as the optimizer runs. The Console Panel at the bottom of the main window also displays progress messages.
4. When optimization completes, the dialog shows the result. A new instrument document with the optimized geometry is added to the Instruments list in the Study Panel.

You can then evaluate the optimized instrument to verify the improvement, or save it as an XML file.

If you do not have a constraints document, select an optimizer first, then use the **+ Default** button in the Constraints section to generate one with reasonable default bounds.


## Calibration

The first optimizer listed for each study model is a calibrator. Calibrators adjust mouthpiece parameters (such as fipple factor for NAF, or embouchure parameters for Flute) rather than hole geometry.

When a calibrator is selected, the toolbar button changes from "Optimize" to **Calibrate**. Calibration does not require a constraints document -- only an instrument and a tuning are needed.

1. Select an instrument and a tuning.
2. Select the calibrator (the first optimizer in the list).
3. Click **Calibrate**.
4. The result is a new instrument with updated mouthpiece parameters. The original instrument is preserved.

Calibration is typically the first step before running a geometry optimizer. It ensures the acoustic model's mouthpiece parameters are tuned to your physical instrument.


## Analysis Tools

The toolbar provides several analysis tools. All tool results open in popup windows (your browser may ask you to allow popups from the application).

**Sketch** -- Draws a cross-section diagram of the selected instrument, showing the bore profile, tone holes, and mouthpiece geometry. Requires an instrument to be selected.

**Compare** -- Opens a dialog where you pick two instruments, then shows their geometries side by side in a popup window. Requires at least two instruments to be loaded.

**Evaluate** -- Described above. Shows predicted vs. target frequencies for all fingerings.

**Supplementary** -- Displays additional acoustic information for the selected instrument and tuning, including impedance characteristics and calculated dimensions.

**Graph** -- Plots impedance curves for each fingering, showing the playing ranges. Requires an instrument and tuning to be selected.

**Spectrum** -- Shows a note impedance spectrum chart. Runs an evaluation first (if one has not already been run) to determine the fingering frequencies, then plots the impedance response for each note.

**Wizard** -- Opens a dialog for generating a tuning document from a temperament and scale pattern. Does not require any documents to be loaded. See the Tuning Wizard documentation for details.

For detailed information on each tool, see the individual tool documentation pages.


---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Using-WIDesigner).*
