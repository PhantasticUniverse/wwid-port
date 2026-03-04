# wid-acoustics

Transfer matrix and state vector calculations for the acoustic elements of a compiled instrument: bore sections, toneholes, termination, and mouthpiece.

## How It Fits In

```
wid-compile                    wid-acoustics                   wid-eval
┌──────────────┐    ┌────────────────────────────────┐    ┌─────────────┐
│ Component    │    │                                │    │             │
│ chain:       │───▶│  tube    bore    hole          │───▶│  calc_z()   │
│ [Bore, Hole, │    │    ↓       ↓       ↓           │    │  walks the  │
│  Bore, Hole, │    │  Transfer matrices              │    │  chain to   │
│  ..., Bore]  │    │                                │    │  compute    │
│              │    │  termination   mouthpiece       │    │  impedance  │
│ Mouthpiece   │───▶│    ↓              ↓            │───▶│             │
│ Termination  │    │  Boundary conditions            │    │             │
└──────────────┘    └────────────────────────────────┘    └─────────────┘
```

Each acoustic element produces a 2×2 complex transfer matrix. The evaluation pipeline in `wid-eval` multiplies these matrices in sequence (from termination to mouthpiece) to compute the input impedance at any frequency.

## Modules

| Module | Physical Element | Model |
|--------|-----------------|-------|
| `tube` | Cylindrical/conical tube | Viscothermal losses via complex propagation constant. Radiation impedance via Padé approximants (Silva et al. 2008). |
| `bore` | Bore section | Delegates to `tube::calc_cone_matrix()` with section geometry (left/right radius, length) |
| `hole` | Tonehole | Lefebvre & Scavone (2012) T-network: series impedance Za + shunt admittance Ys. Open/closed controlled by fingering. |
| `termination` | Open end | Radiation impedance. **ThickFlanged** (NAF): infinite flange correction. **Unflanged** (Whistle/Flute/Reed): Levine & Schwinger model. |
| `mouthpiece` | Default fipple (NAF) | Headspace ×4 end correction, fipple factor scaling, window impedance |
| `simple_fipple` | Simple fipple / embouchure hole | Empirical Xw/Rw model for Whistle (Fipple) and Flute (EmbouchureHole). Same formulas, different parameter extraction. |
| `simple_reed` | Reed mouthpiece | Linear reactance model for single/double/lip reeds |

## Mouthpiece Models

The mouthpiece model is the key differentiator between study models. Each uses a fundamentally different acoustic model:

### DefaultFipple (NAF)

Traditional fipple mouthpiece with explicit headspace. The headspace bore sections above the mouthpiece position are treated as resonating chambers — each contributes a transfer matrix, with an end correction scaled by a factor of 4. The fipple factor (typically 0.2–0.4) scales the window impedance, controlling the effective open area.

### SimpleFipple (Whistle, Flute)

Empirical model using measured window impedance correlations. The effective size depends on the mouthpiece type:
- **Fipple** (Whistle): `eff_size = √(windowLength × windowWidth)`
- **EmbouchureHole** (Flute): `eff_size = √(min(width, airstreamLength) × length)`

Same Xw/Rw formulas for both — only the parameter extraction differs.

### SimpleReed (Reed instruments)

Linear reactance model: `X = α × 10⁻³ × freq + β`. The transfer matrix uses a pressure-node boundary condition: `[[0+iX, z₀], [1, 0]]`. Lip reeds negate the beta sign. Headspace bore sections are extracted during compilation but are intentionally not used (matching Java `SimpleReedMouthpieceCalculator` parity).

## Study Model Parameters

| Constant | NAF | Whistle/Flute | Reed |
|----------|-----|---------------|------|
| `hole_size_mult` | 0.9605 | 1.0 | 1.0 |
| `finger_adjustment` | 0.0 | 0.010 | 0.010 |
| Termination | ThickFlanged | Unflanged | Unflanged |
| Mouthpiece | DefaultFipple | SimpleFipple | SimpleReed |
| Blowing level | N/A | 5 | N/A |

NAF-specific constants: `AIR_GAMMA = 1.4018...`, headspace ×4 end correction, `DEFAULT_WINDWAY_HEIGHT = 0.00078740 m`.

## Dependencies

- `wid-math` — TransferMatrix and StateVector types
- `wid-physics` — PhysicalParameters for wave number, impedance, losses
- `wid-compile` — compiled instrument component types
- `num-complex` — Complex64 arithmetic

## Tests

3 unit tests (termination module): closed-end behavior, unflanged vs thick-flanged divergence, unflanged finite result. Full validation through `wid-eval` integration tests against golden Z-samples (NAF, Whistle, Flute, Reed).
