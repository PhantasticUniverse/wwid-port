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
| `MouthpieceType` | `Fipple { ... }` or `EmbouchureHole { ... }` |
| `CompiledTermination` | Flange diameter, bore diameter, position |

## Dependencies

- `wid-types` — input `InstrumentRaw` type

## Key invariants

- Components alternate `Bore, Hole, Bore, Hole, ..., Bore`
- Holes are in ascending position order
- All bore sections have positive length (≥ `MINIMUM_CONE_LENGTH`)
- Headspace ends exactly at mouthpiece position
- All dimensions are in metres after compilation

## Tests

17 tests validating component count (13 for 6-hole NAF), ordering, headspace extraction, bore diameter interpolation, and geometry validation. Component counts match golden `internals_0.json`.
