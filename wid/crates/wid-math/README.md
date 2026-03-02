# wid-math

Core math types for acoustic transfer matrix calculations.

Provides `TransferMatrix` (2×2 complex matrix) and `StateVector` (pressure + volume-flow pair) used throughout the impedance pipeline. Both types are `Copy` — no heap allocation in the inner loop.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `TransferMatrix` | 2×2 complex matrix with `pp`, `pu`, `up`, `uu` components |
| `TransferMatrix::identity()` | No-op transformation |
| `TransferMatrix::multiply()` | Matrix × matrix |
| `TransferMatrix::multiply_sv()` | Matrix × state vector |
| `TransferMatrix::determinant()` | pp·uu − pu·up |
| `StateVector` | Pressure `p` + volume-flow `u` |
| `StateVector::open_end()` | P=0, U=1 |
| `StateVector::closed_end()` | P=1, U=0 |
| `StateVector::from_impedance()` | Dickens (2007) normalized construction |
| `StateVector::impedance()` | Z = P/U |
| `StateVector::reflectance()` | Pressure reflection coefficient |
| `StateVector::series()` / `parallel()` | Impedance combination |

## Dependencies

- `num-complex` — Complex64 arithmetic

## Tests

22 unit tests covering matrix algebra, state vector operations, boundary cases (open/closed ends, infinite impedance), and composition properties (associativity).
