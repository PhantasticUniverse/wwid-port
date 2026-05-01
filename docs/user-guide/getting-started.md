# Getting Started

[User Guide](index.md) > Getting Started

This page walks you through your first session with WIDesigner Web: loading sample files, running an evaluation, editing an instrument, and launching an optimization. You can be up and running in about five minutes.

## Prerequisites

You need a modern web browser with WebAssembly support. Any of the following will work:

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

No installation, plugins, or accounts are required. The entire application runs locally in your browser.

## Step 1: Open the App

Navigate to the WIDesigner Web URL in your browser. The app loads a WebAssembly module on startup -- this may take a second or two on first visit.


## Step 2: Choose a Study Model

In the header bar at the top of the page, you will see a dropdown menu next to the "WIDesigner" title. Click it and select a study model:

- **NAF** -- Native American Flutes
- **Whistle** -- Tin Whistles
- **Flute** -- Transverse Flutes (keyless)
- **Reed** -- Reed instruments (chanters, didgeridoos)

For this walkthrough, select **NAF**.


## Step 3: Load a Sample Bundle

Click **Load Sample** in the header bar and choose **NAF F#4 Starter**. The app loads the instrument, tuning, and matching constraints files directly from the bundled `/samples/` directory.

If you prefer to download files manually, the NAF walkthrough uses:

- [`/samples/NafStudy/0.625-bore_6-hole_NAF_starter.xml`](/samples/NafStudy/0.625-bore_6-hole_NAF_starter.xml) -- a starter NAF instrument with 0.625" bore diameter and 6 holes
- [`/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_tuning.xml`](/samples/NafStudy/Fsharp4_ET_6-hole_NAF_chromatic_tuning.xml) -- an equal temperament tuning in F#4 for a 6-hole NAF
- [`/samples/NafStudy/NAF_HoleFromTop_constraints.xml`](/samples/NafStudy/NAF_HoleFromTop_constraints.xml) -- constraints for hole-position optimization

See [Sample Files](sample-files.md) for the full list of bundled files across all study models.

## Step 4: Load Your Files

If you used **Load Sample**, the files are already loaded and selected. To use your own files, click the **Open File** button in the header bar. A file picker dialog appears. Select XML files (you can select multiple files at once, or open them one at a time).

The files appear in the **Study Panel** on the left side of the screen. Instrument files appear under the "Instruments" section, and tuning files appear under the "Tunings" section.


## Step 5: Select an Instrument and Tuning

Single-click the instrument name (`0.625-bore_6-hole_NAF_starter`) in the Study Panel. It highlights in blue to indicate it is selected.

Single-click the tuning name (`F#4_ET_6-hole_NAF_chromatic_tuning`) in the Study Panel. It also highlights in blue.

With both selected, the **Evaluate** button in the toolbar becomes active. WIDesigner requires a matching pair -- the instrument and tuning must have the same number of holes.

## Step 6: Run an Evaluation

Click **Evaluate** in the toolbar row. The tuning comparison table opens in the app by default; you can switch tool output to popup windows in Settings. For each fingering in the tuning, the table displays:

- The note name
- The target frequency (from the tuning file)
- The predicted frequency (computed from the instrument geometry)
- The deviation in cents

This tells you how well the instrument's current geometry matches the desired tuning.


## Step 7: Edit an Instrument

Double-click the instrument name in the Study Panel to open it in an editor tab in the central workspace. The editor shows:

- **Mouthpiece parameters** -- bore position, fipple dimensions (window length, width, height, windway length, windway height), and fipple factor
- **Bore profile** -- a table of position/diameter pairs defining the bore shape
- **Hole table** -- position, diameter, and height for each tone hole

You can modify any value directly. Changes take effect immediately for subsequent evaluations. Use the **Form/XML** toggle in the workspace header to switch between the structured editor and raw XML view.


## Step 8: Optimize (Brief Overview)

To optimize an instrument's geometry against a tuning target:

1. Load a constraints file (e.g., `NAF_HoleFromTop_constraints.xml` from the NAF samples) using Open File.
2. Select an instrument, tuning, optimizer, and constraints in the Study Panel. The optimizer and constraints lists are populated with options appropriate for the current study model. You can click "+ Default" to add a default constraints file.
3. Click **Optimize** in the toolbar. The optimizer adjusts the instrument's dimensions within the constraint bounds to minimize tuning deviation.
4. Progress and results appear in the Console Panel at the bottom of the screen.
5. When optimization completes, the optimized instrument appears in the Study Panel. You can evaluate it to see the improved tuning.

For a detailed walkthrough, see [Optimization Workflow](optimizers/workflow.md).

## UI Layout Reference

The WIDesigner Web interface has four main areas:

```
+---------------------------------------------------------------+
| WIDesigner  [NAF v] [Load Sample] [Open File] [Save] [Settings] | Header
+---------------------------------------------------------------+
| Sketch | Compare || Evaluate | Suppl. || Graph | Spectrum     |  Toolbar Row
| || Optimize | Calibrate | Wizard                              |
+----------+----------------------------------------------------+
|          |                                                    |
| Study    |              Workspace                             |
| Panel    |         (tabbed editors)                           |
| (224px)  |                                                    |
|          |                                                    |
| Instr.   |                                                    |
| Tunings  |                                                    |
| Optim.   |                                                    |
| Constr.  |                                                    |
|          |                                                    |
+----------+----------------------------------------------------+
| Console Panel (optimization output, messages)                 |
+---------------------------------------------------------------+
```

- **Header bar**: Application title, study model dropdown, Load Sample, Open File, Save, and Settings.
- **Toolbar row**: Action buttons grouped by function -- visualization (Sketch, Compare), evaluation (Evaluate, Supplementary), analysis (Graph Tuning, Note Spectrum), and optimization (Optimize/Calibrate, Tuning Wizard).
- **Study Panel** (left sidebar, 224px wide): Lists loaded documents organized by type -- Instruments, Tunings, Optimizers, Constraints. The Constraints section has "+ Default" and "+ Blank" buttons to create new constraint documents when an optimizer is selected. Single-click to select; double-click to open in an editor.
- **Workspace** (center): Tabbed editor area for open documents. Each tab shows the document's name and a close button.
- **Console Panel** (bottom): Displays optimization progress, error messages, and other output.

## Next Steps

- Browse the [Sample Files](sample-files.md) for all four study models.
- Read about your study model: [NAF](study-models/naf.md), [Whistle](study-models/whistle.md), [Flute](study-models/flute.md), or [Reed](study-models/reed.md).
- Learn about [Optimization Workflow](optimizers/workflow.md) to start designing your own instruments.

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki).*
