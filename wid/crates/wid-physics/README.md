# wid-physics

Air properties and acoustic wave parameters from environmental conditions.

Implements the CIPM-2007 moist air model (density, viscosity, thermal conductivity, speed of sound) and derives wave number, attenuation constant, and bore impedance needed for acoustic modelling.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `PhysicalParameters` | Full CIPM-2007 model from temperature, pressure, humidity, CO₂ |
| `PhysicalParameters::calc_wave_number()` | k = 2πf/c |
| `PhysicalParameters::calc_z0()` | Bore characteristic impedance ρc/πr² |
| `PhysicalParameters::get_complex_wave_number()` | Lossy propagation constant |
| `PhysicalParameters::get_epsilon()` | Dimensionless loss factor |
| `SimplePhysicalParameters` | Polynomial approximation for fipple mouthpiece |
| `TemperatureType` | °C or °F input selector |

## Dependencies

- `num-complex` — Complex64 for lossy wave number

## Academic references

- CIPM-2007 (Picard et al. 2008) — density, compressibility, vapour pressure
- Tsilingiris (2008) — viscosity and thermal conductivity mixing rules
- Owen Cramer (JASA 1993) — speed of sound polynomial

## Tests

20 unit tests, all validated against golden `internals_0.json` to 12+ digit precision. Default conditions: 72 °F, 101.325 kPa, 45% RH, 390 ppm CO₂.
