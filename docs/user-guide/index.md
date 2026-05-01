# WIDesigner Web App User Guide

This is the user guide for WIDesigner Web, a browser-based tool for designing and optimizing woodwind instruments. It is a port of [WIDesigner v2.6.0](https://github.com/edwardkort/WWIDesigner), originally a Java desktop application, now running entirely in your browser using WebAssembly.

WIDesigner supports four study models -- Native American Flutes (NAF), Tin Whistles, Transverse Flutes, and Reed instruments -- and provides acoustic evaluation, optimization, and analysis tools for each.

---

## Table of Contents

### Getting Started

- [Getting Started](getting-started.md) -- Quick start guide: open the app, load sample files, and run your first evaluation in five minutes.
- [Sample Files](sample-files.md) -- Bundled instrument, tuning, fingering, and constraints files for all four study models.

### Using WIDesigner

- [Using WIDesigner](using-widesigner.md) -- Overview of the web interface: layout, file handling, editing, evaluation, and optimization.
- [Settings](settings.md) -- Configure temperature, humidity, length units, DIRECT optimizer toggle, and spectrum multiplier.

### Study Models

- [NAF](study-models/naf.md) -- Native American Flute study model: fipple mouthpiece, open fingerings, pentatonic and chromatic scales.
- [Whistle](study-models/whistle.md) -- Tin Whistle study model: fipple mouthpiece, six-hole diatonic fingering, playing range analysis.
- [Flute](study-models/flute.md) -- Transverse Flute study model: embouchure hole mouthpiece, keyless six-hole design.
- [Reed](study-models/reed.md) -- Reed instrument study model: single, double, and lip reeds for chanters, didgeridoos, and similar instruments.

### Optimizers

- [Optimizer Overview](optimizers/overview.md) -- How optimization works: objective functions, constraints, BOBYQA, DIRECT, and multi-start search.
- [NAF Optimizers](optimizers/naf.md) -- Hole position, hole size, hole grouping, and taper optimizers for Native American Flutes.
- [Whistle & Flute Optimizers](optimizers/whistle-flute.md) -- Hole, bore, headjoint, stopper, taper, and merged optimizers for Whistles and Flutes.
- [Reed Optimizers](optimizers/reed.md) -- Hole position, bore diameter, and merged optimizers for Reed instruments.
- [Constraints](optimizers/constraints.md) -- How to create and edit constraints files that bound optimization dimensions.
- [Optimization Workflow](optimizers/workflow.md) -- Step-by-step guide to running an optimization from start to finish.

### Tools

- [Evaluation](tools/evaluation.md) -- Compare predicted tuning against target frequencies for each fingering.
- [Note Spectrum](tools/note-spectrum.md) -- View impedance magnitude across frequency for each fingering, with gain coloring.
- [Supplementary Info](tools/supplementary.md) -- View calculated acoustic properties: air speed, flow rate, gain, and Q factor.
- [Sketch & Compare](tools/sketch-and-compare.md) -- Visualize instrument geometry and compare two instruments side by side.
- [Tuning Wizard](tools/tuning-wizard.md) -- Generate tuning files from a scale, temperament, and fingering pattern.

### Reference

- [Glossary](reference/glossary.md) -- Definitions of acoustic and instrument design terms used throughout the app.
- [Bibliography](reference/bibliography.md) -- Published sources for the acoustic models and algorithms.
- [Tuning Winds](reference/tuning-winds.md) -- Background on tuning woodwind instruments.
- [Modelling Instruments](reference/modelling-instruments.md) -- How WIDesigner models bore acoustics, tone holes, and mouthpieces.
- [NAF Design](reference/naf-design.md) -- Design considerations specific to Native American Flutes.
- [Reed & Didgeridoos](reference/reed-didgeridoos.md) -- Design considerations for reed instruments and didgeridoos.
- [Differences from Java](reference/differences-from-java.md) -- Notable differences between this web app and the original Java desktop application.
