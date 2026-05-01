# Optimization Workflow

[User Guide](../index.md) > [Optimizers] > Workflow

This page walks through the practical workflow for optimizing a woodwind instrument, from initial calibration through iterative refinement. The examples use Whistle and Flute terminology, but the same principles apply to NAF and Reed instruments with their respective calibrators and optimizers.

## Overview

Instrument optimization is an iterative process that follows the construction phases of a real instrument. You do not run a single optimization and consider the design finished. Instead, you alternate between modeling, optimizing, building, measuring, and re-optimizing as the physical instrument takes shape.


## Phase 1: Planning Toneholes

**Situation:** You have a bore (tube) with no holes drilled yet. You want to determine where to place the holes and how large to make them.

### Step 1: Create an instrument file

Measure your bore and create an instrument XML file with:
- The bore profile (position and diameter pairs along the tube).
- The mouthpiece parameters (fipple dimensions for Whistle/NAF, embouchure hole dimensions for Flute, reed parameters for Reed).
- No tone holes (or placeholder holes at arbitrary positions).

Load this instrument into WIDesigner using the Open File button.

### Step 2: Set physical parameters

Open [Settings](../settings.md) and verify that the temperature matches your playing environment. If you are designing for outdoor use, consider a lower temperature than a heated room.

### Step 3: Calibrate

1. Create a tuning file with only the bell note (the note produced with all holes closed, or no holes at all). Measure this frequency on your physical instrument.
2. Load the tuning file and select both the instrument and the tuning in the Study Panel.
3. Select the calibrator for your study model (Whistle calibration, Flute calibration, Fipple factor, or Reed calibrator).
4. Click **Calibrate**.
5. The result is a new instrument with updated mouthpiece parameters. Select this calibrated instrument for subsequent steps.

If you do not have a physical instrument yet and are designing from scratch, you can skip calibration and use the default mouthpiece parameters. Calibration is most valuable when you have a real instrument to measure.

### Step 4: Optimize hole layout

1. Load a tuning file with your target scale (all notes, with desired frequencies).
2. Select the calibrated instrument, the target tuning, and a hole optimizer (such as "Hole position & size").
3. Generate constraints by clicking **+ Default** in the Constraints section. This creates bounds appropriate for the selected optimizer.
4. Review the constraints (double-click to open in the editor). Adjust bounds if needed -- for example, set a minimum bore length based on your physical tube.
5. Click **Optimize**.
6. When the optimization completes, a new instrument appears in the Study Panel with the optimized geometry.

### Step 5: Evaluate the result

1. Select the optimized instrument and the target tuning.
2. Click **Evaluate** to see the predicted tuning deviation for each note.
3. Use **Graph Tuning** to visualize the playing range and impedance curves.
4. If the result is satisfactory, save the optimized instrument (click Save in the header bar).

If the result is not satisfactory, try adjusting constraint bounds and re-optimizing, or try a different optimizer (e.g., a taper optimizer to explore bore modifications).

## Phase 2: Fine-Tuning the Model

**Situation:** You have drilled holes based on the Phase 1 optimization, but drilled them slightly undersized. You want to refine the model using actual measurements.

### Step 1: Re-measure

Measure all frequencies on the physical instrument with all fingering combinations. Update the tuning file with these measured values.

### Step 2: Re-calibrate

Calibrate again using the updated measurements. The mouthpiece parameters may shift slightly now that holes are present.

### Step 3: Optimize hole sizes only

1. Select the "Hole size only" optimizer. Since holes are already drilled, you do not want to change positions.
2. Create constraints that only allow enlargement. Set the lower bound for each hole diameter to its current measured size. Set the upper bound to the maximum acceptable size.
3. Click **Optimize**.

The optimizer will suggest how much to enlarge each hole to bring the tuning into alignment.

### Step 4: Evaluate and iterate

Evaluate the result. If the deviation is acceptable, enlarge the holes on the physical instrument. If not, adjust constraints and re-optimize.

## Phase 3: Fine-Tuning the Instrument

**Situation:** Holes are near their final size. You are making small physical adjustments.

At this stage, you rely primarily on hands-on tuning rather than modeling:
- Undercut hole edges to sharpen individual notes.
- Slightly enlarge holes that are flat.
- Use the Evaluate tool to track progress against the target tuning.

WIDesigner remains useful as a reference -- run evaluations to confirm that your physical adjustments match the model's predictions.

## Phase 4: Planning New Instruments

**Situation:** You want to explore bore modifications or design a new instrument from scratch.

### Bore optimization

Use bore optimizers to explore alternative bore profiles:
- **Basic taper** to introduce or refine a tapered bore section.
- **Bore diameter from top/bottom** to adjust bore diameters at specific points.
- **Bore spacing from top** to reposition bore transition points.

### Merged optimization

Use merged optimizers to simultaneously adjust holes and bore:
- **Holes + basic taper** to optimize hole layout and bore taper together.
- **Holes + headjoint** (Flute) to optimize the embouchure area and holes simultaneously.

### Global optimization

If you suspect the local optimizer is finding a suboptimal solution, enable DIRECT in Settings and use a global optimizer. This is most useful when:
- Starting from a rough design with no prior optimization.
- The design space is large (many holes, complex bore).
- Local optimization produces unsatisfactory results regardless of starting point.

## General Tips

**Drill holes undersized.** Always drill holes smaller than the optimizer suggests. You can enlarge a hole but you cannot shrink it. Re-measure, re-calibrate, and re-optimize with hole-size-only constraints.

**Use Evaluate and Graph Tuning often.** Run an evaluation after every optimization to verify the improvement. Graph Tuning shows the impedance curves and playing range, which can reveal issues that a simple cent deviation number might miss.

**Use Compare to see changes.** Load both the original and optimized instruments, then use the Compare tool to see the geometry differences side by side.

**Consistent blowing matters.** The acoustic model assumes a consistent blowing level. Physical measurements should be taken at a steady, reproducible blowing pressure for reliable calibration and optimization results.

**Save intermediate results.** Save the optimized instrument after each phase. If a later optimization produces worse results, you can go back to the previous version.

**Start simple, then add complexity.** Begin with a hole-only optimizer. If the results are not satisfactory, try adding bore optimization (merged optimizers). Only use global optimization when local optimization is clearly insufficient.

## Prerequisites

- An instrument, tuning, optimizer, and constraints document loaded and selected in the Study Panel.
- For calibration: only an instrument and tuning are needed (no constraints).
- For iterative refinement: measured frequencies from a physical instrument.

## See Also

- [Optimizer Overview](overview.md) -- algorithm descriptions and general concepts.
- [NAF Optimizers](naf.md), [Whistle & Flute Optimizers](whistle-flute.md), [Reed Optimizers](reed.md) -- optimizer details for each study model.
- [Constraints](constraints.md) -- creating and editing constraint bounds.
- [Settings](../settings.md) -- physical parameter configuration.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/Optimizing-a-Whistle-or-Flute-Design).*
