# wid-compile

Converts `InstrumentRaw` (from XML) into `InstrumentCompiled` — a component chain ready for acoustic evaluation. This explicit compile step prevents the baseline's implicit "forgot to call updateComponents" bugs.

## What Compilation Does

```
                    InstrumentRaw (XML geometry)
                            │
          ┌─────────────────┼─────────────────┐
          ▼                 ▼                 ▼
   Sort bore points   Extract holes     Compute mouthpiece
   by position        with bore dia     headspace + type
          │                 │                 │
          └────────┬────────┘                 │
                   ▼                          │
        Interleave into component chain       │
        [Bore, Hole, Bore, Hole, ..., Bore]   │
                   │                          │
                   └──────────┬───────────────┘
                              ▼
                     InstrumentCompiled
                   {mouthpiece, components, termination}
```

1. **Sort** bore points by position (ascending)
2. **Extract** headspace (bore segments above the mouthpiece position)
3. **Interleave** bore sections with holes in position order
4. **Interpolate** bore diameter at each hole position
5. **Compute** termination (flange diameter from last bore point)
6. **Package** mouthpiece with type-specific parameters

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

### Geometry Mutation API

These functions read and write geometry vectors on `InstrumentRaw` — used by optimizers to iteratively adjust the instrument.

| Function | Description |
|----------|-------------|
| `get_hole_geometry_from_top()` | Extract HoleFromTop geometry vector (NAF) |
| `set_hole_geometry_from_top()` | Apply HoleFromTop geometry vector |
| `get_hole_geometry_position()` | Extract position geometry: bore end + inter-hole spacings |
| `set_hole_geometry_position()` | Apply position geometry |
| `get_hole_diameters()` | Extract hole diameter vector |
| `set_hole_diameters()` | Apply hole diameters |
| `get_fipple_factor()` / `set_fipple_factor()` | Read/write fipple factor |
| `get_beta()` / `set_beta()` | Read/write mouthpiece beta factor |
| `get_window_height()` / `set_window_height()` | Read/write fipple window height |
| `get_airstream_length()` / `set_airstream_length()` | Read/write embouchure airstream length |
| `get_alpha()` / `set_alpha()` | Read/write reed alpha parameter |

**Important**: After any geometry mutation, you must `compile()` again before evaluating. The compiled representation caches interpolated bore diameters and component positions.

## Geometry Parameterizations

### HoleFromTop (NAF)

| Index | Meaning | Unit |
|-------|---------|------|
| `0` | Bore end position | metres |
| `1` | Top hole position as fraction of bore | dimensionless |
| `2..N` | Inter-hole spacings (top→bottom) | metres |
| `N+1..2N` | Hole diameters (top→bottom) | metres |

Total dimensions: `2N + 1` (13 for a 6-hole NAF).

### Position (Whistle/Flute/Reed)

| Index | Meaning | Unit |
|-------|---------|------|
| `0` | Bore end position | metres |
| `1..N` | Inter-hole spacings (top→bottom) | metres |

Total dimensions: `N + 1` (7 for a 6-hole whistle).

### Merged Position + Size (Whistle/Flute/Reed)

| Index | Meaning | Unit |
|-------|---------|------|
| `0..N` | Position geometry (bore end + spacings) | metres |
| `N+1..2N` | Hole diameters (top→bottom) | metres |

Total dimensions: `2N + 1` (13 for a 6-hole whistle).

## Key Invariants

- Components alternate `Bore, Hole, Bore, Hole, ..., Bore`
- Holes are in ascending position order
- All bore sections have positive length (≥ `MINIMUM_CONE_LENGTH`)
- Headspace ends exactly at mouthpiece position
- All dimensions are in metres after compilation
- After geometry mutation, must re-compile before evaluation

## Dependencies

- `wid-types` — input `InstrumentRaw` type

## Tests

22 tests validating component count (13 for 6-hole NAF), ordering, headspace extraction, bore diameter interpolation, geometry get/set round-trip, fipple factor mutation, and mouthpiece type extraction. Component counts match golden `internals_0.json`.
