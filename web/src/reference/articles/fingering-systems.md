# Fingering Systems And Tuning Targets

## Scope

This page covers six-hole NAF fingering families, Flutopedia chart baselines,
the user-provided Wood Wind / Edward Kort-style WIDesigner XML, cross-fingering
rows, scale/temperament targets, pitch standards, and optimizer weight caveats.

## Agent Summary

Do not treat all six-hole fingering charts as the same system. Flutopedia's
pentatonic-minor and diatonic pages are comparison baselines. The local
WIDesigner XML is a separate provisional target set: it implies F#4, A4=440,
equal temperament, and a 15-row scheme that starts all-closed on F#4, jumps to
A4, runs chromatically through A5, and includes two G5 rows. The `G5 (closed)`
row has a user-confirmed zero optimization weight and must not pull optimizer
residuals.

The physical hole order for the XML is not established yet. Preserve serialized
WIDesigner XML order until CAD, WIDesigner, or timestamped video evidence
confirms physical mouth-to-foot mapping.

## Evidence Map

| Evidence path | Role |
| --- | --- |
| `www.flutopedia.com/fingerings.htm` | Flutopedia fingering gateway and chart context. |
| `www.flutopedia.com/fingeringChart_SixPentatonicMinor.htm` | Six-hole pentatonic-minor comparison baseline. |
| `www.flutopedia.com/fingeringChart_SixDiatonic.htm` | Diatonic comparison baseline. |
| `www.flutopedia.com/naf_tunings.htm` | Tuning and key context in the mirror. |
| `www.flutopedia.com/tuning_Grealish.htm` | Chromatic/cross-fingering tuning context. |
| `sources/user_wood_wind_chromatic_tuning_xml.json` | Manifest for the local XML. |
| `sources/user_wood_wind_chromatic_tuning_xml/fsharp4-et-6-hole-naf-chromatic-wid.xml` | Raw WIDesigner tuning XML. |
| `indexes/fingering-schemes/user_wood_wind_fsharp4_chromatic_tuning.v0.json` | Normalized JSON rows, weights, duplicates, and caveats. |
| `docs/independent-agent-research/extend/2026-06-04-alternate-fingering-scheme-cad-intake-contract.md` | Full intake contract for XML/CAD/video comparison. |
| `sources/nakai_art_of_native_american_flute_1996.json` | Book source guide for Nakai tablature/finger-pattern coverage and reuse limits. |
| `widesigner-2.6-docs/docs/data-and-ui-reference.html` | WIDesigner data and UI meanings. |
| `widesigner-2.6-docs/docs/optimization-reference.html` | Weight and residual interpretation lead. |

## Maker Workflow

For a design or tuning question:

1. Identify the fingering family before comparing charts.
2. Record pitch standard, temperament, target frequencies, and source row names.
3. Preserve alternate rows rather than deduplicating by frequency.
4. Keep zero-weight and uncertain-weight rows separate from measured or
   optimized targets.
5. Map serialized hole order to physical hole order only after evidence.
6. Measure pitch under defined breath pressure and environment before claiming
   the design is tuned.
7. Treat Nakai tablature as its own notation/fingering family, with actual
   sounded pitch mapped separately when ensemble or concert-pitch use matters.

## Data Fields

Minimum fingering fields: source id, note name, target frequency, cents target,
pitch standard, temperament, row sequence, hole count, open/closed pattern,
hole-order convention, alternate-row flag, optimization weight, weight
confidence, physical mapping confidence, and measured validation state.

## Do Not Overclaim

Do not describe the user XML as "the Flutopedia fingering." It overlaps parts
of common six-hole pentatonic-minor practice but diverges in upper and
cross-fingered rows.

Do not say the CAD model is definitively in F#4 from the XML alone. The related
XML implies F#4 intent; `docs/source-intake/user_flute_taper_fusion_design.md`
says the Fusion archive itself did not expose an authoritative key.

Do not treat the user XML, normalized JSON, or related CAD intake notes as
public release evidence. They are private, source-governed examples for local
analysis until their release cards, physical hole-order crosswalks, and
measured validation records permit stronger use.

Do not interpret `G5 (closed)` as all holes closed. In the normalized artifact
the pattern is `XOXXXX`, and its weight is zero.

Do not merge Nakai tablature, Flutopedia chart rows, and WIDesigner XML rows
without a crosswalk that records five-hole/six-hole assumptions, primary versus
extended scale, alternate fingerings, unplayable tones, and actual pitch.

## Open Gaps

Remaining work includes timestamped notes for the WIDesigner Introduction video,
physical hole-order verification, measured pitch/pressure rows for the user
design, WIDesigner runtime fixtures for weight behavior, and a broader
fingering-system taxonomy that separates pentatonic-minor, diatonic,
chromatic-extension, and maker-specific schemes.
