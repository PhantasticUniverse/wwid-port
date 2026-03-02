# wid-types

Serde structs mapping directly to the WIDesigner XML schema.

All dimensional values are stored in the units specified by `length_type` (typically inches). Conversion to metres happens during compilation in `wid-compile`.

## Public API

| Type / Function | Description |
|----------------|-------------|
| `InstrumentRaw` | Top-level instrument (bore points, holes, mouthpiece, termination) |
| `Tuning` | Note targets + fingering patterns |
| `Fingering` | Note + open-hole boolean vector + optimization weight |
| `Note` | Name + optional frequency/min/max |
| `LengthType` | Unit system with `to_metres()` conversion factor |
| `MouthpieceRaw` | Fipple, embouchure hole, or reed mouthpiece |
| `Constraints` | Optimization bounds (name, objective function, constraint list) |
| `Constraint` | Display name, category, type, lower/upper bound |
| `parse_instrument_xml()` | Deserialize instrument from WIDesigner XML |
| `parse_tuning_xml()` | Deserialize tuning from WIDesigner XML |
| `parse_constraints_xml()` | Deserialize constraints from WIDesigner XML |
| `strip_xml_namespaces()` | Remove `ns2:` prefix before deserialization |

## Dependencies

- `serde` — derive deserialization
- `quick-xml` — XML parsing with serde integration

## Namespace handling

WIDesigner XML uses a namespace prefix on the root element (`<ns2:instrument xmlns:ns2="...">`), but child elements are unqualified. `strip_xml_namespaces()` removes the prefix before serde can parse it.

## Constraints ordering

The bounds arrays extracted from `Constraints` preserve insertion order (category order, then constraint order within each category). This is ABI — the optimizer's parameter vector indices depend on this layout.

## Tests

17 tests covering instrument/tuning/constraints parsing from all oracle NAF XMLs (6 instruments, 6 tunings, 16 constraint files across 8 objective function types), namespace stripping, and unit conversion.
