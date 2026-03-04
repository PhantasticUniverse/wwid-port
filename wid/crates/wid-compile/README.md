# wid-compile

Converts `InstrumentRaw` (from XML) into `InstrumentCompiled` — a component chain ready for acoustic evaluation. This explicit compile step prevents the baseline's implicit "forgot to call updateComponents" bugs.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `compile()` | Main entry point: `InstrumentRaw` → `InstrumentCompiled` |
| `InstrumentCompiled` | Mouthpiece + component chain + termination |
| `Component` | Enum: `Bore(BoreSection)` or `Hole(CompiledHole)` |
| `BoreSection` | Length + left/right radius (metres) |
| `CompiledHole` | Position, diameter, height, interpolated bore diameter |
| `CompiledMouthpiece` | Position, bore diameter, headspace sections, type |
| `MouthpieceType` | `Fipple { ... }`, `EmbouchureHole { ... }`, or `SimpleReed { alpha, is_lip_reed }` |
| `CompiledTermination` | Flange diameter, bore diameter, position |
| `get_hole_geometry_from_top()` | Extract geometry vector (metres) for HoleFromTop optimizer |
| `set_hole_geometry_from_top()` | Apply geometry vector to `InstrumentRaw` |
| `get_fipple_factor()` | Read fipple factor from instrument |
| `set_fipple_factor()` | Write fipple factor on instrument |
| `get_beta()` / `set_beta()` | Read/write mouthpiece beta factor |
| `get_window_height()` / `set_window_height()` | Read/write fipple window height |
| `get_airstream_length()` / `set_airstream_length()` | Read/write embouchure hole airstream length |

## Dependencies

- `wid-types` — input `InstrumentRaw` type

## Key invariants

- Components alternate `Bore, Hole, Bore, Hole, ..., Bore`
- Holes are in ascending position order
- All bore sections have positive length (≥ `MINIMUM_CONE_LENGTH`)
- Headspace ends exactly at mouthpiece position
- All dimensions are in metres after compilation
- After geometry mutation, must re-compile before evaluation

## Geometry mutation

The `get_/set_hole_geometry_from_top()` functions work with the HoleFromTop parameterization:

| Index | Meaning | Unit |
|-------|---------|------|
| `0` | Bore end position | metres |
| `1` | Top hole position as fraction of bore | dimensionless |
| `2..N` | Inter-hole spacings (top→bottom) | metres |
| `N+1..2N` | Hole diameters (top→bottom) | metres |

Total dimensions: `2N + 1` (13 for a 6-hole NAF).

## Tests

22 tests validating component count (13 for 6-hole NAF), ordering, headspace extraction, bore diameter interpolation, geometry get/set round-trip, and fipple factor mutation. Component counts match golden `internals_0.json`.
