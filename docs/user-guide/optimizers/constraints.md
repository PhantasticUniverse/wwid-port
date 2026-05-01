# Optimization Constraints

[User Guide](../index.md) > [Optimizers] > Constraints

Constraints define the upper and lower bounds on each parameter that an optimizer varies. Every non-calibrator optimizer requires a constraints document before it can run. This page explains what constraints are, how they relate to optimizer parameters, and how to create and edit them in the web app.

## What Constraints Are

A constraints document is a list of parameter bounds. Each entry has:

- **Display name** -- a human-readable description of the parameter (e.g., "Bore length," "Hole 1 diameter," "Taper ratio").
- **Type** -- the unit of measurement. Common types are Dimensional (meters), and Dimensionless (ratios, factors). The display unit depends on your Length Type setting (inches, mm, etc.).
- **Lower bound** -- the minimum allowed value for the parameter.
- **Upper bound** -- the maximum allowed value for the parameter.

The optimizer will never set a parameter outside its bounds. If both bounds are equal, that parameter is effectively fixed.

## Parameters vs Physical Dimensions

Constraint parameters do not always correspond directly to physical measurements you can take with a ruler. Some examples:

- **Hole spacing** is the distance between adjacent holes (or from the bore top to the first hole), not the absolute position of each hole along the bore.
- **Bore diameter ratio** is the ratio of a bore point's diameter to a reference diameter, not the absolute diameter itself.
- **Taper ratio** is the ratio of diameters at two bore sections, not either diameter individually.
- **Grouped hole spacing** controls the uniform spacing within a group of holes. A single parameter affects the position of every hole in the group.

A single constraint parameter can control multiple geometry values. For example, in a grouped-hole optimizer, changing the group spacing parameter moves all holes in that group simultaneously.

## Constraints Must Match the Optimizer

Each optimizer has its own parameter layout. A constraints document created for "Hole size only" (N parameters) is not compatible with "Hole position & size" (2N + 1 parameters). If you select a constraints document that does not match the selected optimizer, the optimization will fail.

The safest approach is to generate constraints for each optimizer you intend to use, using the "+ Default" or "+ Blank" buttons described below.

## Creating Constraints in the Web UI

To create a constraints document:

1. Select an optimizer in the Optimizers section of the Study Panel.
2. At the bottom of the Constraints section, two buttons appear:
   - **+ Default** -- creates a new constraints document with pre-populated bounds based on the study model's defaults. These defaults are reasonable starting bounds derived from the original WIDesigner application.
   - **+ Blank** -- creates a new constraints document with zero bounds for all parameters. You fill in every bound manually.
3. The new constraints document appears in the Constraints list and is automatically selected.


## Editing Constraints

Double-click a constraints document in the Study Panel to open it in an editor tab. The editor displays a table with columns for the parameter description, type, lower bound, and upper bound.

Edit the bound values directly in the table cells. Changes take effect immediately in the session -- you do not need to save before running an optimization.


## Tips for Setting Bounds

- **Physical realities**: Set bounds that reflect what is physically achievable. Minimum hole diameter should be large enough for a fingertip to cover. Maximum hole spacing should be comfortable for hand reach.
- **Bore length**: The lower bound on bore length should be the shortest playable tube for your target pitch. The upper bound should be the longest tube you are willing to build.
- **Taper ratio**: A ratio of 1.0 means no taper (cylindrical). Values below 1.0 mean the bore narrows. Keep the range narrow unless you want to explore extreme profiles.
- **Start with defaults**: The "+ Default" bounds are a good starting point. Run an optimization with defaults first, then tighten bounds based on the results.
- **Constraining hole growth**: If holes are already drilled, set the lower bound for each hole diameter to its current size. This ensures the optimizer only suggests enlargement, never shrinkage (you cannot un-drill a hole).

## Constraint Structure in XML

If you save a constraints document, the resulting XML file contains:

- The optimizer name it was created for.
- The number of holes it applies to.
- A list of constraint entries, each with a display name, type, lower bound, and upper bound.
- Optional hole group definitions (for grouped-hole optimizers).

You can load previously saved constraints files using the Open File button, just like instrument and tuning files.

## Prerequisites

- An optimizer must be selected in the Study Panel before the "+ Default" and "+ Blank" buttons appear.
- An instrument should be loaded if you want default bounds to reflect the instrument's current geometry (some defaults depend on the number of bore points).

## See Also

- [Optimizer Overview](overview.md) -- how optimization works and algorithm descriptions.
- [NAF Optimizers](naf.md) -- parameter layouts for NAF optimizers.
- [Whistle & Flute Optimizers](whistle-flute.md) -- parameter layouts for Whistle and Flute optimizers.
- [Reed Optimizers](reed.md) -- parameter layouts for Reed optimizers.
- [Optimization Workflow](workflow.md) -- step-by-step guide to running an optimization.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Optimization-Constraints).*
