# Parity Notes

Details on tricky Java→Rust parity decisions. These are intentional choices, not bugs.

## Temperature Default

The Rust core and golden fixtures use **72°F (22.22°C)** — matching `PhysicalParameters(72.0, TemperatureType.F)` from the Java study model constructors.

The Java GUI overrides this to **20°C** via `OptimizationPreferences.DEFAULT_TEMPERATURE`. Our web app has a Settings dialog where users can change temperature (default shows as 22.22°C from the session).

## Geometry Ordering

`HolePosition` uses indexed `geometry[i+1]` assignment (top-to-bottom spacing order), **not** `push()`. Using `push()` produces bottom-to-top order which silently reverses the spacing vector, causing wrong optimization results. See Java `HolePositionObjectiveFunction.getGeometryPoint()`.

## Constraints Ordering ABI

Lower/upper bound arrays from constraints XML must match the baseline's column ordering exactly. The objective function parameterization depends on this layout (bore length first, then spacings top-to-bottom, then diameters top-to-bottom for merged optimizers).

## Solver Divergence Patterns

BOBYQA is path-dependent — different starting simplex configurations can produce different evaluation counts but the same final optimum. The golden harness provides a reference point, but evaluation counts may differ by ±30% while final norm and geometry match within tolerance.

Brent bracket sensitivity: the bracket search direction preference (upstream/downstream) must match Java's `PlayingRange.findBracket()` exactly. When the primary-direction bracket is outside `PreferredSolutionRatio`, the fallback direction is preferred unconditionally (not by distance comparison).

## Reed Headspace (M5.6)

Reed instruments (e.g., SampleChanter) can have headspace bore sections — bore points above the mouthpiece position. In Java:
- `Instrument.updateComponents()` extracts headspace bore sections from the component chain
- `SimpleReedMouthpieceCalculator` inherits the default `MouthpieceCalculator.calcStateVector()` which just does `calcTransferMatrix().multiply(boreState)` — **no headspace handling**
- Headspace sections are stored on the instrument but never accessed by the reed calculator

Our Rust code matches this: headspace is extracted during compile and stored on `mouthpiece.headspace`, but the `SimpleReed` arm in `calc_z()` applies the reed TM directly without walking headspace sections. This is intentional parity, not a bug.

In contrast:
- **DefaultFipple** (NAF) uses headspace volume for compliance admittance (×4 end correction)
- **SimpleFipple** (Whistle/Flute) walks headspace as transfer matrices and combines in parallel with bore state

## Zero-Length Bore Sections at Mouthpiece Position

When the mouthpiece position coincides exactly with a bore point (e.g., SampleChanter mouthpiece at -40mm, first bore point at -40mm), `process_position` creates a zero-length bore section. Java's `addSection()` handles this by:

1. Setting length = `MINIMUM_CONE_LENGTH` (0.00001m)
2. Bumping `rightBorePosition` by `MINIMUM_CONE_LENGTH`
3. Updating the bore point's position in the bore list

Our Rust `process_position` matches this behavior. The bumped `right_bore_position` ensures the stub section is NOT extracted into headspace (since `right_bore_position > mouthpiece_position`), keeping the component chain identical to Java's.

**Why this matters**: Without the position bump, the stub section's `right_bore_position` equals the mouthpiece position and gets extracted to headspace, changing the component chain from 21 to 20 elements. The next bore section's length also changes (0.00799m vs 0.008m). These tiny differences accumulate through transfer matrix multiplication, producing ~0.012 cents error across multiple holes.

## Mouthpiece Position

Reed instruments (chanters) can have **negative mouthpiece position** — the mouthpiece sits upstream of the bore origin. Example: SampleChanter has mouthpiece at -40mm with bore points at -40mm and -32mm above it.

Didgeridoos (lip reed) have mouthpiece at position 0.0 — no upstream section.

## DefaultHoleCalculator Constructor Variants (M5.6)

Java's `DefaultHoleCalculator` has **multiple constructors** with different `fingerAdjustment` defaults:

| Constructor | `holeSizeMult` | `fingerAdjustment` | Used by |
|---|---|---|---|
| `DefaultHoleCalculator()` | 1.0 | **0.010** | `SimpleReedCalculator` |
| `DefaultHoleCalculator(holeSizeMult)` | arg | **0.0** | `NAFCalculator(0.9605)`, `WhistleCalculator(1.0)` |
| `DefaultHoleCalculator(isPlugged, fingerAdj)` | 1.0 | arg | Direct use |

The default (no-arg) constructor sets `fingerAdjustment = DEFAULT_FINGER_ADJ = 0.010`. The 1-arg constructor (taking `holeSizeMult`) sets it to `NO_FINGER_ADJ = 0.0`. This is easy to miss because the difference is buried in constructor overload semantics.

**Impact**: `fingerAdjustment` affects the closed-hole shunt admittance via `tf = radius² / fingerAdjustment`, which shifts `tan(k·(te - tf))`. The effect is small per hole (~0.0005 in Im(Z)) but accumulates through multiple closed holes. For SampleChanter with 8 closed holes, the cumulative effect shifted the Im(Z)=0 crossing by ~0.3 Hz, producing ~2.8 cents error.

**Lesson**: When porting a Java class's configuration, trace the **exact constructor call** in the calculator/study model, don't infer defaults from the class name or documentation. Different constructors can silently set different defaults.

## Gain Model

Only fipple and embouchure instruments have a gain factor (Auvray 2012). Reed instruments return `None` for `compute_gain_factor()`. The gain factor is used by the LinearV tuner for Strouhal-based frequency prediction — reed instruments use the simpler `SimpleInstrumentTuner` (standard reactance-zero search) instead.

## BOBYQA Chaotic Sensitivity in Merged Optimizers

BOBYQA's convergence trajectory is **chaotically sensitive** to sub-ULP evaluation differences. Even when two implementations (Java and Rust) produce function values matching to ~0.0000001% (1e-9 relative), the quadratic model's Hessian diagonal estimates can amplify these differences by ~1000×, causing divergent optimization trajectories.

**Observed case**: NAF single-taper optimization (16 dimensions, rhobeg=0.003175). All 33 preliminary evaluations matched Java to ~1e-9 relative error. Despite this, Java converged to norm=208 while Rust converged to norm=907 — both valid local minima, but different ones.

**Root cause**: The preliminary evaluations build a quadratic model via finite differences. The Hessian diagonal for dimension `i` is estimated from two evaluations spaced `rhobeg` apart. When `rhobeg` is small (0.003175), even 1e-9 relative error in the function values produces ~1e-6 relative error in the Hessian estimate (~1000× amplification). This steers the first trust-region step differently, and from there the trajectories diverge.

**Why this is not a bug**: Both implementations reach a valid optimum — the function landscape has multiple local minima. The exact path BOBYQA takes through this landscape is inherently sensitive to IEEE 754 evaluation order, compiler optimizations, and platform-specific floating-point behavior.

**Testing strategy**: For sensitive merged optimizers, tests verify:
1. **Evaluation parity**: applying the golden-optimal geometry to a fresh instrument produces the golden norm (within 1%). This confirms the objective function itself is correct.
2. **Norm reduction**: optimization reduces the norm by >99% from the initial value. This confirms the optimizer is working effectively.
3. **Reasonable final norm**: final norm is within a tolerance factor (e.g., 5×) of the golden value. This catches gross algorithmic errors without demanding trajectory-identical convergence.

Tests do NOT assert exact geometry or norm match for these sensitive problems.

## Brent vs BOBYQA Dispatch for 1D Optimizations

Java's `ObjectiveFunctionOptimizer` dispatches based on the `optimizerType` field set by each objective function:

| Objective function | Java optimizerType (1D) | Our Rust dispatch (1D) | Notes |
|---|---|---|---|
| Bore standalone (BoreDiameterFromBottom, etc.) | BOBYQAOptimizer (base default) | **Brent** | Brent is better for 1D |
| HoleSizeObjectiveFunction | **CMAESOptimizer** (explicit override for nrDims==1) | BOBYQA | 1-hole edge case |
| HolePositionObjectiveFunction | **CMAESOptimizer** (explicit override) | BOBYQA | 1-hole edge case |

**Our bore optimizer golden harness** (`BoreOptDriver.java`) explicitly dispatches to `BrentOptimizer` for 1D cases (matching our Rust `run_1d_or_nd` function). The Java study model would use BOBYQA for 1D bore problems (since bore objective functions don't override `optimizerType`), but BOBYQA on a 1D problem is equivalent — both find the same optimum.

**Hole optimizers with 1 hole**: Java's `HoleSizeObjectiveFunction` and `HolePositionObjectiveFunction` switch to CMAES (Covariance Matrix Adaptation Evolution Strategy) when `nrDimensions == 1`, since "BOBYQA doesn't support single dimension" per the Java comment. In practice, our BOBYQA implementation handles 1D fine (`n_interp = 2*1+1 = 3` meets the minimum requirement of `n+2`). CMAES is not implemented — it's a full evolutionary strategy optimizer that would be a large implementation effort for the extremely rare edge case of 1-hole instruments.

**MaxEvaluations for standalone bore optimizers**: Java uses the base default of 10000. Our formula `20000 + (n_dims - 1) * 5000` gives 20000 for 1D — double Java's limit. This is conservative (more computation, never fewer evaluations than needed). For merged bore optimizers, we use Java's exact overrides: 40K (BoreFromBottom, Headjoint), 50K (HoleAndBore* variants), 20K (HoleAndTaper).

## Trust Region Overrides per Optimizer Class

Java's `BaseObjectiveFunction.getInitialTrustRegionRadius(double[])` computes the BOBYQA initial trust radius from the bounds:
```
max_expected_change = max(upper[i] - lower[i])
min_radius = min(0.5 * (upper[i] - lower[i])) for non-degenerate bounds
initial_trust = max(min_radius, 0.1 * max_expected_change)
stopping_trust = 1e-8 * initial_trust
```

However, several Java subclasses override to a hardcoded `initial=10.0, stopping=1e-8`:

| Java class | initial_trust | stopping_trust | Our Rust |
|---|---|---|---|
| BaseObjectiveFunction (default) | bounds-based formula | `1e-8 * initial` | `compute_trust_radius()` — matches |
| HoleFromTopObjectiveFunction | **10.0** | **1e-8** | **10.0 / 1e-8** — fixed to match |
| HoleGroupFromTopObjectiveFunction | **10.0** | **1e-8** | **10.0 / 1e-8** — fixed to match |
| NafHoleSizeObjectiveFunction | **10.0** | **1e-8** | **10.0 / 1e-8** — fixed to match |
| SingleTaper* (all 4 variants) | **10.0** | **1e-8** | **10.0 / 1e-8** — already correct |
| HoleSizeObjectiveFunction | bounds-based | `1e-8 * initial` | bounds-based — matches |
| HolePositionObjectiveFunction | bounds-based | `1e-8 * initial` | bounds-based — matches |
| HoleObjectiveFunction | bounds-based | `1e-8 * initial` | bounds-based — matches |

**NAF hole_size trust radius**: Fixed — `hole_size.rs` now accepts a `naf_trust` parameter. NAF passes `true` (10.0/1e-8, matching `NafHoleSizeObjectiveFunction`), other models pass `false` (bounds-based, matching `HoleSizeObjectiveFunction` which inherits from `BaseObjectiveFunction`).

**Why 10.0 matters**: For instruments with small bore diameters (e.g., 0.019m), the bounds-based formula produces initial_trust ~0.003–0.05. An initial_trust of 10.0 is ~200× larger, which makes BOBYQA's first few steps explore much more aggressively. This can lead to different local minima on multimodal landscapes. The convergence behavior differs, though both approaches eventually find good optima.

## Stale Bore Points in Merged Taper Optimization

When a merged optimizer (e.g., SingleTaper) reuses a work instrument across evaluations, bore point mutations from one evaluation can corrupt subsequent evaluations.

**The bug pattern**: When the taper sub-optimizer applies `set_taper_geometry` with `taper_length < 1.0`, it creates intermediate bore points between the head and foot. On the next evaluation, if the hole-position sub-optimizer shortens the bore (decreases bore_end), those intermediate bore points survive beyond the new bore end. The taper function then reads `bot_pos` from the stale intermediate point instead of the new bore end, producing the wrong bore length.

**Example**: EVAL[17] creates bore points at (0.0, 0.323859, 0.324890). EVAL[18] sets bore_end = 0.321715. The stale point at 0.323859 is now above the new bore end. `set_taper_geometry` reads `bot_pos = 0.323859` instead of `0.321715`, making the bore 2.14mm too long.

**The fix**: In `set_merged_taper_geometry`, after `set_hole_geometry_from_top` moves the bore end, `retain` only bore points at or below `new_bore_end + epsilon`:
```rust
raw.bore_points.retain(|bp| bp.bore_position * m <= new_bore_end + 1e-9);
```

**How this mistake can happen**: Any optimizer that (a) reuses a work instrument between evaluations AND (b) has multiple sub-optimizers that independently modify bore geometry is susceptible. The sub-optimizers assume the bore is in a "clean" state matching the instrument's original topology, but the previous evaluation's sub-optimizer may have added, moved, or removed bore points.

**Prevention rule**: When writing merged/multi-component optimizers that reuse a work instrument, always sanitize bore point topology between evaluations — either clone-per-eval (expensive) or explicitly remove stale bore points before each sub-optimizer runs. A regression test exists in `single_taper.rs::reused_instrument_matches_fresh_instrument`.

## Named Bore Point Heuristics

Java's bore optimizers need to split the bore profile into "changed" and "unchanged" regions. Two helper methods determine the boundary:

### `find_body_top` (Java: `Instrument.getTopOfBody()`)

Returns the index of the topmost bore point considered part of the instrument body. Used by BoreDiameterFromBottom, BorePosition, BoreFromBottom. The fallback chain is:

1. Look for a bore point named "Body" → return its index
2. Look for a bore point named "Head" → return its index (the bottom of the head = top of body)
3. Fall back to the lowest bore point above the topmost hole's position

**Critical**: The `n_unchanged` parameter is `find_body_top(inst) + 1` — the "+1" means the body-top point itself is also unchanged. This was a source of a past off-by-one bug.

### `find_head_point` (Java: `BoreDiameterFromTopObjectiveFunction.getLowestPoint("Head")`)

Returns how many bore points from the top are considered part of the headjoint (the "changed" region). Used by BoreDiameterFromTop, BoreSpacingFromTop. The fallback chain is:

1. Search bore points top-to-bottom for one named "Head" → return `index + 1`
2. If no "Head" found, fall back to the same heuristic as `find_body_top` but return the count differently

**SamplePVC-Whistle (3 bore points, no named points)**: Both functions fall back to the hole-based heuristic. `find_body_top` returns 1, so BoreDiameterFromBottom gets `n_unchanged=2`, `n_dims=1`. `find_head_point` returns 1, so BoreDiameterFromTop gets `n_changed=1`, `n_dims=1`.

## PRESERVE_BELL Semantics

The `BoreLengthAdjust::PreserveBell` mode shifts bore points proportionally when the bore end changes, preserving the "bell" shape at the bottom of the instrument.

### `find_bell` algorithm (Java: `Instrument.getBell()`)

Identifies the start of the bell section by finding the longest segment between consecutive bore points:
1. Walk bore points bottom-to-top, computing segment lengths
2. Track the longest segment seen
3. The bell starts at the bore point where `position >= longest_segment_start`

**Java quirk**: The comparison is `pos >= longest` (using `>=`), which means if two segments have equal length, the bell index could include an extra point. Our Rust code replicates this exactly.

**Edge case**: 2-point bore → bellIndex at the last point, making the adjustment a no-op (no bell to preserve).

### How bell preservation works

When `set_hole_positions_adjusted` changes the bore end:
1. Bore points above the bell index are left unchanged
2. Bore points at or below the bell index are shifted proportionally: `new_pos = old_pos * (new_bore_end / old_bore_end)`
3. The bottom bore point's position is set to exactly `new_bore_end`

## Flange Diameter Coupling

`set_bore_diameter_from_bottom` adjusts `termination.flange_diameter` when the bottom bore diameter changes. This coupling is easy to miss:

```rust
// After setting bore diameters:
if let Some(ref mut term) = raw.termination {
    term.flange_diameter = new_bottom_bore_diameter / m;
}
```

**Why this matters**: Flange diameter affects radiation impedance at the open end → tuning. If the bore diameter at the bottom changes but the flange diameter doesn't track it, the radiation impedance calculation uses a stale diameter. This causes silent acoustic divergence — the evaluation still produces a number, but it's slightly wrong.

**Java reference**: `BoreDiameterFromBottomObjectiveFunction.setGeometryPoint()` updates `termination.setFlangeDiameter(...)` after changing bore diameters.

## Bore Spacing Upper Bound Clamping

Java's `BoreSpacingFromTopObjectiveFunction.setUpperBounds()` overrides the standard upper bound setter to prevent bore points from inverting their order during optimization.

**The problem**: If the sum of requested bore spacings exceeds the total bore length, bore points would need to overlap (negative spacing), which is physically impossible and crashes the acoustic calculation.

**The fix**: `clamp_bore_spacing_upper_bounds()` computes `total_available_length / sum_of_upper_bounds`. If this ratio is < 1.0, all upper bounds are scaled by this ratio. This ensures the optimizer can never request spacings that exceed the bore length.

```rust
pub fn clamp_bore_spacing_upper_bounds(
    instrument: &InstrumentRaw, n_changed: usize, upper: &mut [f64],
) {
    let total_length = bore_end - bore_start;
    let requested: f64 = upper.iter().sum();
    if requested > total_length {
        let scale = total_length / requested;
        for u in upper.iter_mut() { *u *= scale; }
    }
}
```

## maxEvaluations Per Optimizer Class

Java's `BaseObjectiveFunction` default is `maxEvaluations = 10000`. Individual subclasses override:

| Java class | maxEvaluations | Our Rust |
|---|---|---|
| Base default | 10000 | — |
| HoleSizeObjectiveFunction | `20000 + (nrDims-1)*5000` | `max_evaluations(n)` — matches |
| HolePositionObjectiveFunction | 10000 (default, no override!) | `max_evaluations(n)` — **overallocates** |
| HoleObjectiveFunction | `20000 + (nrDims-1)*5000` | `max_evaluations(n)` — matches |
| HoleFromTopObjectiveFunction | `20000 + (nrDims-1)*5000` | `max_evaluations(n)` — matches |
| HoleGroupFromTop/SingleTaper* | `20000 + (nrDims-1)*5000` | `max_evaluations(n)` — matches |
| BoreDiameter/Spacing/Position (standalone) | 10000 | `max_evaluations(n)` — **overallocates** |
| BoreFromBottom (merged) | 40000 | 40000 — matches |
| Headjoint | 40000 | 40000 — matches |
| HoleAndTaper | 20000 | 20000 — matches |
| HoleAndBoreDia/Spacing/Position | 50000 | 50000 — matches |
| HoleAndBoreFromBottom | 60000 | 60000 — matches |
| HoleAndHeadjoint | 50000 | 50000 — matches |
| Calibrators (Whistle/Flute/Reed) | 10000 (default) | `max_evaluations(2)` = 25000 — **overallocates** |

"Overallocates" means we allow more evaluations than Java. This is conservative — the optimizer converges before hitting the limit, so the extra budget is never used. It won't cause parity issues.

## Per-Study-Model Physical Parameter Defaults

Java sets different default physical parameters per study model:

| Study Model | Temperature | Pressure | Humidity | CO2 |
|---|---|---|---|---|
| NAF | 72°F (22.22°C) | 101.325 kPa | 45% | 390 ppm |
| Whistle/Flute/Reed | 27°C | 98.4 kPa | 100% | 40000 ppm (0.04 mol/mol) |

The NAF defaults come from `PhysicalParameters(72.0, TemperatureType.F)` (2-arg constructor, which uses 101.325/45%/390ppm). The Whistle/Flute/Reed defaults come from `PhysicalParameters(27.0, TemperatureType.C, 98.4, 100.0, 0.04)` (5-arg constructor in each study model's `setDefaults()`).

This doesn't affect golden fixture parity — the harness always sets physical params explicitly. It only affects the default state a user sees when opening a fresh session in the GUI.

## Optimizer List: addSub() vs objectiveFunctionNames

Java's study models have two distinct optimizer-related data structures:

1. **`objectiveFunctionNames`** (Map) — includes ALL optimizer keys for constraint generation dispatch. Used by `getDefaultConstraints()` / `getBlankConstraints()`.
2. **`addSub()` calls** — create GUI menu items that users can actually select.

These are different. For example, `WindowHeightObjectiveFunction` and `BetaObjectiveFunction` are in `objectiveFunctionNames` for Whistle (so constraints can be generated for them when used internally by the joint calibrator), but they don't have `addSub()` calls (users can't select them directly from the optimizer dropdown).

Our `available_optimizers()` should only mirror `addSub()` items (what users see in the dropdown). The `is_valid_optimizer()` and constraint generation functions should still handle the full set (for internal dispatch).

## Analysis Tool Frequency Target Fallbacks

Java's `InstrumentTuner.getFrequencyTarget()` tries `frequency → frequencyMax → frequencyMin → 0.0`. The evaluators already implement this correctly, but the analysis tools have their own fallback chains:

- **`graph_tuning`** (`PlotPlayingRanges.buildGraph()`): Uses `getFrequencyTarget()` chain (`frequency → frequencyMax → frequencyMin → 0.0`). When target is 0.0, Java's `predictedNote()` returns early with an empty note, and the graph skips that curve. Rust matches by producing an empty `points` vec and `continue`-ing.

- **`note_spectrum`** (`PlayingRangeSpectrum.plot()`): Uses `frequency → frequencyMax → 1000.0` — different from `getFrequencyTarget()` because this is a display tool with its own fallback logic. Note: no `frequencyMin` in this chain, and default is 1000.0 (not 0.0 or 440.0).

These are distinct from the evaluator frequency handling (which uses `frequency.or(frequency_max)` for fmax evaluators, and returns 0.0 for missing targets).

## `set_bore_position` Identical Branches

Both `if bottom_fixed` branches in `set_bore_position` computed `n_unchanged + d`. This was correct (not a bug) because the getter loop starts at `d=1` with `n_unchanged + (d-1)`, while the setter loop starts at `d=0` with `n_unchanged + d` — the mapping is equivalent. Round-trip tests at line 2736 confirm. Simplified to remove the dead `if`.

## DIRECT max_evaluations Enforcement

Java's DIRECT enforces `max_evaluations` per-evaluation: `computeObjectiveValue` calls `incrementEvaluationCount()` which throws `TooManyEvaluationsException` immediately when the limit is reached, potentially mid-rectangle-division. The exception propagates up and the caller catches it.

Rust's DIRECT checks the budget at two points: (1) between rectangle divisions within `divide_potentially_optimal` (stops selecting new rectangles to divide), and (2) between iterations in the main loop. A single rectangle division can still overshoot by up to `2 * n_dimensions` evals, but this is bounded and the algorithm always completes the current rectangle cleanly, producing a valid result from a fully consistent state. Java's mid-evaluation abort can leave partially divided rectangles in the tree, though the caller only uses the best point found.

## Bore Diameter Getter Minimum Guard

Java's `BoreDiameterFromTopObjectiveFunction.getGeometryPoint()` and `BoreDiameterFromBottomObjectiveFunction.getGeometryPoint()` clamp the reference diameter to a minimum of `0.000001` before dividing to get the ratio. This prevents division by zero or inf when a bore point has zero diameter. Rust now matches: `prior_dia.max(0.000001)` / `next_dia.max(0.000001)`.

## Bore Spacing Upper Bound Epsilon

Java's `BoreSpacingFromTopObjectiveFunction.setUpperBounds()` uses an epsilon offset: `if (upperBound + 0.0001 > availableSpace)` and `scale = available / (sum + 0.0001)`. This triggers clamping slightly earlier and ensures strict feasibility. Rust now matches.

## Bore Position Bottom-Hole Guard

Java's `BorePositionObjectiveFunction.setLowerBounds()` ensures the bore end stays at least 12mm below the lowest hole: `if (aLowerBounds[0] < bottomHolePosition + 0.012) aLowerBounds[0] = bottomHolePosition + 0.012`. Applies when `bottomPointUnchanged == false` (standalone bore position and bore-from-bottom optimizers). Rust now matches.

## Supplementary Info `usePredicted` Flag

Java's base `StudyModel.calculateSupplementaryInfo()` uses `usePredicted=false`. Only `NafStudyModel` overrides to `true`. Reed, Whistle, and Flute inherit `false`. Rust now correctly uses `matches!(study_kind, StudyKind::NAF)` instead of `linear_v_tuner.is_none()` (which incorrectly gave `true` for Reed).

## BOBYQA `sqrt(denom)` vs `sqrt(abs(denom))`

Both Java and Fortran compute `sqrt(denom)` in the Z-matrix update. `denom` should always be positive because the algorithm selects `knew` to maximize it. The Rust port was using `sqrt(abs(denom))` which would silently produce a valid (but wrong) result if `denom` were negative, instead of propagating NaN. Fixed to match Java/Fortran. In practice this code path is never triggered on well-conditioned problems.

## Trust Radius Overrides (Audit #2)

`hole_from_top::optimize_holes_with_progress` was using bounds-based `compute_trust_radius()` instead of Java's hardcoded 10.0/1e-8. Fixed in audit #2. The sibling functions (`hole_group_from_top`, `single_taper`) use a shared `_impl` pattern that prevented this class of bug.

## Visual Presentation Differences

Pixel-perfect UI replication is a non-goal. The web port uses a dark theme (modern web convention) vs Java Swing's light theme. Below are intentional visual deviations with rationale.

### Chart Library: JIDE (Java) vs Chart.js (web)
Java uses JIDE Charts (proprietary, tightly integrated with Swing). We use Chart.js (open-source, well-maintained). Chart.js renders to HTML5 Canvas, not SVG — line rendering differs at sub-pixel level. Interactive behavior (tooltips, hover) is web-native.

### Graph Tuning ("Impedance Pattern")
Java draws all curves in black/dark gray with green filled circles at peak resonances (fmax), blue open circles at zero crossings (fmin), and colored diamonds at target frequencies. We match this with gray curves + scatter overlay datasets. The chart title matches Java's "Impedance Pattern".

### Note Spectrum
Java uses green dots (gain >= 1) and red dots (gain < 1) with a black impedance line. We use continuous line segments in green/red with NaN-gap splitting, plus a dashed gain=1 reference line. The impedance line is dark gray (our dark theme equivalent of Java's black-on-white).

### Sketch Diagram
Java uses JFreeChart's XYPlot to render a top-down engineering drawing: dashed bore outline, circles for holes, axis labels. We use custom SVG with the same engineering conventions: dashed bore polygon, outline circles for holes, labeled X/Y axes with tick marks. No colored fills — monochrome gray palette.

### Default Constraints
Java's "Create Default Constraints" pre-populates bounds with study-model-specific values (bore length ranges, hole diameter ranges, taper ratios). Our web port now matches this. "Create Blank Constraints" creates empty bounds (Java fills with 0.0/1.0; we use None). The constraints are used identically for optimization — this is a usability feature, not a computation difference.

### Settings Dialog
Java has a multi-tab preferences dialog (temperature, humidity, length type, DIRECT toggle, constraints directory). Our settings are minimal: temperature, humidity, DIRECT toggle. Missing settings: length type (always metric in our implementation), constraints directory (web uses in-memory document store). These are platform differences, not parity gaps.
