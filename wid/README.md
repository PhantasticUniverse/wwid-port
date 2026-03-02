# wid — Rust core of the WIDesigner port

Acoustic modelling and evaluation engine for woodwind instrument design, ported from WIDesigner v2.6.0.

## Status

**M3 complete**: NAF evaluation, calibration, and optimization parity — 146 tests passing.

- All 36 NAF instrument×tuning combinations (540 fingerings) match oracle within 0.5 cents
- Fipple factor calibration (Brent 1D) matches oracle within 1e-6
- Hole geometry optimization (BOBYQA) matches oracle geometry within 5e-3 metres

## Crate dependency diagram

```
wid-optimize
├── bobyqa
├── wid-eval
│   ├── wid-acoustics
│   │   ├── wid-math
│   │   ├── wid-physics
│   │   └── wid-compile
│   │       └── wid-types
│   ├── wid-math
│   ├── wid-physics
│   ├── wid-types
│   └── wid-compile
├── wid-compile
├── wid-physics
└── wid-types
```

| Crate | Purpose | Tests |
|-------|---------|-------|
| `wid-math` | TransferMatrix (2x2 Complex64) + StateVector | 22 |
| `wid-physics` | CIPM-2007 air properties, wave parameters | 20 |
| `wid-types` | Serde structs for WIDesigner XML (instruments, tunings, constraints) | 17 |
| `wid-compile` | `compile(InstrumentRaw)` → component chain; geometry mutation API | 22 |
| `wid-acoustics` | Bore, tonehole, termination, mouthpiece TMs | 0 (via wid-eval) |
| `wid-eval` | Impedance pipeline, root finding, cents deviation | 8 + 4 integration |
| `wid-optimize` | Fipple calibration (Brent), hole optimization (BOBYQA) | 20 |
| `bobyqa` | Standalone BOBYQA optimizer (pure Rust, zero deps) | 32 + 1 doc |

## Quick reference

```bash
cargo build                      # Build all crates
cargo test                       # Run all 146 tests
cargo test -p wid-eval           # Test a single crate
cargo test zsample               # Run tests matching a name
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full data flow, per-crate design guide, and testing strategy.
