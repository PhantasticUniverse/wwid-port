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
| `parse_instrument_xml()` | Deserialize instrument from WIDesigner XML |
| `parse_tuning_xml()` | Deserialize tuning from WIDesigner XML |
| `strip_xml_namespaces()` | Remove `ns2:` prefix before deserialization |

## Dependencies

- `serde` — derive deserialization
- `quick-xml` — XML parsing with serde integration

## Namespace handling

WIDesigner XML uses a namespace prefix on the root element (`<ns2:instrument xmlns:ns2="...">`), but child elements are unqualified. `strip_xml_namespaces()` removes the prefix before serde can parse it.

## Tests

9 tests covering instrument/tuning parsing from oracle XMLs, namespace stripping, and unit conversion.
