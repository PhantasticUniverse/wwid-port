# wid-acoustics

Transfer matrix and state vector calculations for the acoustic elements of a compiled instrument: bore sections, toneholes, termination, and mouthpiece.

## Modules

| Module | Model | Description |
|--------|-------|-------------|
| `tube` | Lossy cylinder/cone TMs | Viscothermal losses via complex propagation constant. Radiation impedance via Padé approximants (Silva et al. 2008). |
| `bore` | Bore section TM | Delegates to `tube::calc_cone_matrix()` with section geometry |
| `hole` | Tonehole T-network | Lefebvre & Scavone (2012): series impedance Za + shunt admittance Ys |
| `termination` | ThickFlanged / Unflanged open end | Reflection coefficient model. ThickFlanged (NAF) with flange correction; Unflanged (Whistle/Flute) with Levine & Schwinger model. |
| `mouthpiece` | Default fipple (NAF) | Headspace ×4, fipple factor scaling, window impedance |
| `simple_fipple` | Simple fipple / embouchure hole | Empirical Xw/Rw model for Whistle (Fipple) and Flute (EmbouchureHole). Same formulas, different parameter extraction. |

## Study model dispatch

The mouthpiece model is selected by `CalculatorParams` in `wid-eval`:

- **NAF**: `DefaultFipple` → `mouthpiece` module. Fipple factor scaling + headspace end correction.
- **Whistle**: `SimpleFipple` → `simple_fipple` module. Parameter extraction from `Fipple` fields: `eff_size = sqrt(windowLength × windowWidth)`.
- **Flute**: `SimpleFipple` → `simple_fipple` module. Parameter extraction from `EmbouchureHole` fields: `eff_size = sqrt(min(width, airstreamLength) × length)`.

## Dependencies

- `wid-math` — TransferMatrix and StateVector types
- `wid-physics` — PhysicalParameters for wave number, impedance, losses
- `wid-compile` — compiled instrument component types
- `num-complex` — Complex64 arithmetic

## Study model parameters

Constants used in `CalculatorParams` (defined in `wid-eval`):

| Constant | NAF | Whistle/Flute |
|----------|-----|---------------|
| `hole_size_mult` | 0.9605 | 1.0 |
| `finger_adjustment` | 0.0 | 0.010 |
| Termination | ThickFlanged | Unflanged |
| Mouthpiece | DefaultFipple | SimpleFipple |
| Blowing level | N/A | 5 |

Additional NAF constants: `AIR_GAMMA = 1.4018...`, headspace ×4 end correction, `DEFAULT_WINDWAY_HEIGHT = 0.00078740 m`.

## Tests

3 unit tests (termination module): closed-end behavior, unflanged vs thick-flanged divergence, unflanged finite result. Full validation through `wid-eval` integration tests against golden Z-samples (NAF, Whistle, Flute).
