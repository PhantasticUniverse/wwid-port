# wid-acoustics

Transfer matrix and state vector calculations for the acoustic elements of a compiled instrument: bore sections, toneholes, termination, and mouthpiece.

## Modules

| Module | Model | Description |
|--------|-------|-------------|
| `tube` | Lossy cylinder/cone TMs | Viscothermal losses via complex propagation constant. Radiation impedance via Padé approximants (Silva et al. 2008). |
| `bore` | Bore section TM | Delegates to `tube::calc_cone_matrix()` with section geometry |
| `hole` | Tonehole T-network | Lefebvre & Scavone (2012): series impedance Za + shunt admittance Ys |
| `termination` | Thick-flanged open end | Reflection coefficient model with flange correction |
| `mouthpiece` | Fipple (NAF) mouthpiece | Headspace ×4, fipple factor scaling, window impedance |

## Dependencies

- `wid-math` — TransferMatrix and StateVector types
- `wid-physics` — PhysicalParameters for wave number, impedance, losses
- `wid-compile` — compiled instrument component types
- `num-complex` — Complex64 arithmetic

## NAF-specific constants

- `NAF_HOLE_SIZE_MULT = 0.9605` — empirical hole size correction
- `AIR_GAMMA = 1.4018...` — hardcoded adiabatic index
- Headspace ×4 end correction factor
- `DEFAULT_WINDWAY_HEIGHT = 0.00078740 m`

## Tests

No unit tests in this crate. Validated entirely through `wid-eval` integration tests against golden Z-samples and evaluation results.
