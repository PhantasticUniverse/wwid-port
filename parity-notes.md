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
