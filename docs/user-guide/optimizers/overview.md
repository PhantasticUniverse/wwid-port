# Optimizer Overview

[User Guide](../index.md) > [Optimizers] > Overview

WIDesigner optimizers adjust instrument geometry to bring predicted playing frequencies closer to the target tuning. This page explains what optimization does, how the algorithms work, and what to expect from the output.

## What Optimization Does

An optimizer varies selected instrument dimensions -- hole sizes, hole positions, bore diameters, bore length, or combinations of these -- within the bounds you specify in a constraints document. At each iteration, the optimizer evaluates the instrument's predicted tuning across all fingerings and computes the sum of squared cent deviations from the target frequencies. The goal is to minimize this error.

Notes in the tuning file can be weighted to prioritize certain frequencies. A note with weight 2.0 contributes four times as much to the error as a note with weight 1.0 (because the weight is squared). Notes with weight 0.0 are excluded from the optimization entirely.

## Parameters and Dimensions

The parameters that an optimizer adjusts may not correspond directly to physical measurements. Some examples:

- **Hole spacing** is measured as the distance between adjacent holes (or from the bore top to the first hole), not as an absolute position along the bore.
- **Bore diameter ratios** express the diameter at a bore point relative to a reference diameter, not as an absolute value.
- **Grouped hole spacing** uses a single parameter to control the uniform spacing within a group of holes.

The number of parameters an optimizer varies is called its **dimension count**. Each study model's optimizer page lists the dimension count for each optimizer. Avoid under-determined problems where the dimension count exceeds the number of target notes -- the optimizer will have too many degrees of freedom and may produce unreliable results.

## Console Output

When you run an optimization, the Console Panel at the bottom of the window displays progress information:

- **Initial error** -- the starting sum of squared cent deviations before any adjustments.
- **Iteration progress** -- periodic updates as the optimizer evaluates candidate solutions.
- **Final error** -- the optimized error value after convergence.
- **Elapsed time** -- how long the optimization took.

After the optimizer finishes, a new instrument document named `Untitled_N_` appears in the Instruments list in the Study Panel. This document contains the optimized geometry. You can evaluate it, save it, or use it as the starting point for further optimization.


## Algorithms

WIDesigner uses three optimization algorithms, selected automatically based on the problem:

### Brent

Used for one-dimensional problems (a single variable). Brent's method is a root-bracketing algorithm that efficiently finds the minimum of a univariate function. It is fast and exact for 1D problems such as stopper position adjustment.

### BOBYQA

**B**ound **O**ptimization **BY** **Q**uadratic **A**pproximation. This is the primary algorithm for multi-dimensional problems. BOBYQA builds a quadratic model of the objective function and iteratively refines it within the constraint bounds. It is a local optimizer -- it starts from the current instrument geometry and converges to the nearest local minimum.

BOBYQA is deterministic: the same instrument, tuning, and constraints always produce the same result. It is fast (typically a few seconds) and works well when the starting design is already close to optimal.

### DIRECT-C

**DI**viding **RECT**angles (C variant). This is a global optimization algorithm that systematically explores the entire parameter space by subdividing it into hyper-rectangles. It is thorough but slower than BOBYQA.

In WIDesigner, DIRECT-C is used as the first phase of a two-phase approach: DIRECT-C explores broadly to find a promising region, then BOBYQA refines locally within that region. This combination is more likely to find the global optimum on difficult landscapes with multiple local minima.

Global optimizers (those using DIRECT-C) only appear in the optimizer list when **Use DIRECT optimizer** is enabled in [Settings](../settings.md). They are labeled with "(global)" in the optimizer list.

## Determinism

Optimization is deterministic. Given the same instrument, tuning, constraints, and physical parameters (temperature, humidity, etc.), the optimizer will always produce the same result. If you change any input -- even a single constraint bound or physical parameter -- the result may differ.

## Prerequisites

Before running an optimization, you need:

- An **instrument** loaded and selected in the Study Panel.
- A **tuning** loaded and selected, with the same number of holes as the instrument.
- An **optimizer** selected from the optimizer list.
- A **constraints** document selected (not required for calibrators). Use "+ Default" to generate one with pre-populated bounds, or "+ Blank" to create one you fill in manually.

## See Also

- [NAF Optimizers](naf.md) -- optimizers available in the NAF study model.
- [Whistle & Flute Optimizers](whistle-flute.md) -- optimizers available in the Whistle and Flute study models.
- [Reed Optimizers](reed.md) -- optimizers available in the Reed study model.
- [Constraints](constraints.md) -- how to create and edit constraint bounds.
- [Optimization Workflow](workflow.md) -- step-by-step guide to running an optimization.

---

*Adapted from the [original WIDesigner wiki](https://github.com/edwardkort/WWIDesigner/wiki/WIDesigner-Optimizers).*
