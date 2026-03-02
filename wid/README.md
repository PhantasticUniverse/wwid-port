# wid — Rust core of the WIDesigner port

Acoustic modelling and evaluation engine for woodwind instrument design, ported from WIDesigner v2.6.0.

## Status

**M2 complete**: NAF evaluation parity — all 15 fingerings within 0.5 cents of oracle, 76 tests passing.

## Crate dependency diagram

```
wid-eval
├── wid-acoustics
│   ├── wid-math
│   ├── wid-physics
│   └── wid-compile
│       └── wid-types
├── wid-math
├── wid-physics
├── wid-types
└── wid-compile
```

| Crate | Purpose |
|-------|---------|
| `wid-math` | TransferMatrix (2x2 Complex64) + StateVector |
| `wid-physics` | CIPM-2007 air properties, wave parameters |
| `wid-types` | Serde structs for WIDesigner XML |
| `wid-compile` | `compile(InstrumentRaw)` → component chain |
| `wid-acoustics` | Bore, tonehole, termination, mouthpiece TMs |
| `wid-eval` | Impedance pipeline, root finding, cents deviation |

## Quick reference

```bash
cargo build                      # Build all crates
cargo test                       # Run all 76 tests
cargo test -p wid-eval           # Test a single crate
cargo test zsample               # Run tests matching a name
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full data flow, per-crate design guide, and testing strategy.
