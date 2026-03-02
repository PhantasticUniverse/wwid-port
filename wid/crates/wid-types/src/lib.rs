//! XML domain types for WIDesigner instruments and tunings.
//!
//! These types map directly to the WIDesigner XML schema. All dimensional
//! values are stored in the units specified by `length_type` (typically inches).
//! Conversion to metres happens during compilation in [`wid_compile`].
//!
//! # Namespace handling
//!
//! WIDesigner XML uses a namespace prefix on the root element (e.g.,
//! `<ns2:instrument xmlns:ns2="...">`), but child elements are unqualified.
//! Use [`strip_xml_namespaces`] to remove the prefix before deserializing.

pub mod constraints;
pub mod instrument;
pub mod tuning;

pub use constraints::*;
pub use instrument::*;
pub use tuning::*;

/// Strip WIDesigner namespace prefixes from XML for serde deserialization.
///
/// Removes `ns2:` (or similar) prefixes from element tags and the
/// `xmlns:ns2="..."` declaration from the root element.
pub fn strip_xml_namespaces(xml: &str) -> String {
    let mut s = xml.replace("<ns2:", "<").replace("</ns2:", "</");
    // Remove xmlns:ns2="..." attribute
    let needle = " xmlns:ns2=\"";
    if let Some(start) = s.find(needle) {
        let rest = &s[start + needle.len()..];
        if let Some(end_rel) = rest.find('"') {
            let end = start + needle.len() + end_rel + 1;
            s = format!("{}{}", &s[..start], &s[end..]);
        }
    }
    s
}

/// Deserialize an instrument from WIDesigner XML.
pub fn parse_instrument_xml(xml: &str) -> Result<InstrumentRaw, quick_xml::DeError> {
    let clean = strip_xml_namespaces(xml);
    quick_xml::de::from_str(&clean)
}

/// Deserialize a tuning from WIDesigner XML.
pub fn parse_tuning_xml(xml: &str) -> Result<Tuning, quick_xml::DeError> {
    let clean = strip_xml_namespaces(xml);
    quick_xml::de::from_str(&clean)
}

/// Deserialize constraints from WIDesigner XML.
pub fn parse_constraints_xml(xml: &str) -> Result<Constraints, quick_xml::DeError> {
    let clean = strip_xml_namespaces(xml);
    quick_xml::de::from_str(&clean)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const NAF_0HOLE_XML: &str =
        include_str!("../../../../golden/scenarios/support/NAF-FF-02_instrument_0hole.xml");
    const TUNING_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );

    // ── Instrument parsing ──────────────────────────────────────

    #[test]
    fn parse_6hole_naf_instrument() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).expect("parse failed");
        assert_eq!(inst.name, "3/4\" bore, 6-hole NAF start");
        assert_eq!(inst.length_type, LengthType::Inches);
        assert_eq!(inst.bore_points.len(), 2);
        assert_eq!(inst.holes.len(), 6);

        // Mouthpiece
        assert_abs_diff_eq!(inst.mouthpiece.position, 0.18000068040110218, epsilon = 1e-15);
        let fipple = inst.mouthpiece.fipple.as_ref().expect("fipple missing");
        assert_abs_diff_eq!(fipple.fipple_factor.unwrap(), 0.75, epsilon = 1e-15);
        assert_abs_diff_eq!(
            fipple.windway_height.unwrap(),
            0.03200012096019596,
            epsilon = 1e-15
        );

        // Bore points
        assert_abs_diff_eq!(inst.bore_points[0].bore_position, 0.0, epsilon = 1e-15);
        assert_abs_diff_eq!(
            inst.bore_points[0].bore_diameter,
            0.750001215000656,
            epsilon = 1e-12
        );
        assert_abs_diff_eq!(
            inst.bore_points[1].bore_position,
            12.790953423936331,
            epsilon = 1e-12
        );

        // First hole (Hole 6 - closest to mouthpiece)
        assert_eq!(inst.holes[0].name.as_deref(), Some("Hole 6"));
        assert_abs_diff_eq!(
            inst.holes[0].bore_position,
            3.508458242868765,
            epsilon = 1e-12
        );

        // Termination
        assert_abs_diff_eq!(
            inst.termination.flange_diameter,
            1.1250018225009841,
            epsilon = 1e-12
        );
    }

    #[test]
    fn parse_0hole_naf_instrument() {
        let inst = parse_instrument_xml(NAF_0HOLE_XML).expect("parse failed");
        assert_eq!(inst.holes.len(), 0);
        assert_eq!(inst.bore_points.len(), 2);
    }

    #[test]
    fn fipple_optional_fields_are_none_when_absent() {
        let inst = parse_instrument_xml(NAF_6HOLE_XML).expect("parse failed");
        let fipple = inst.mouthpiece.fipple.as_ref().unwrap();
        assert!(fipple.window_height.is_none());
        assert!(fipple.windway_length.is_none());
    }

    // ── Tuning parsing ──────────────────────────────────────────

    #[test]
    fn parse_6hole_naf_tuning() {
        let tuning = parse_tuning_xml(TUNING_XML).expect("parse failed");
        assert_eq!(tuning.name, "F#4 ET 6-hole NAF chromatic tuning");
        assert_eq!(tuning.number_of_holes, 6);
        assert_eq!(tuning.fingerings.len(), 15);
    }

    #[test]
    fn tuning_first_fingering_all_closed() {
        let tuning = parse_tuning_xml(TUNING_XML).unwrap();
        let f0 = &tuning.fingerings[0];
        assert_eq!(f0.note.name, "F#4");
        assert_abs_diff_eq!(f0.note.frequency.unwrap(), 369.9944227116344, epsilon = 1e-10);
        assert_eq!(f0.open_holes, vec![false, false, false, false, false, false]);
        assert_eq!(f0.optimization_weight, Some(1));
    }

    #[test]
    fn tuning_g5_open_all_holes_open() {
        let tuning = parse_tuning_xml(TUNING_XML).unwrap();
        // G5 (open) is fingering index 11
        let g5_open = &tuning.fingerings[11];
        assert_eq!(g5_open.note.name, "G5 (open)");
        assert_eq!(g5_open.open_holes, vec![true, true, true, true, true, true]);
    }

    #[test]
    fn tuning_a4_one_hole_open() {
        let tuning = parse_tuning_xml(TUNING_XML).unwrap();
        let a4 = &tuning.fingerings[1];
        assert_eq!(a4.note.name, "A4");
        assert_eq!(
            a4.open_holes,
            vec![false, false, false, false, false, true]
        );
    }

    // ── Namespace stripping ─────────────────────────────────────

    #[test]
    fn strip_ns_removes_prefix_and_declaration() {
        let input = r#"<ns2:instrument xmlns:ns2="http://www.wwidesigner.com/Instrument"><name>test</name></ns2:instrument>"#;
        let output = strip_xml_namespaces(input);
        assert_eq!(
            output,
            r#"<instrument><name>test</name></instrument>"#
        );
    }

    // ── LengthType conversion ───────────────────────────────────

    #[test]
    fn inches_to_metres() {
        assert_abs_diff_eq!(LengthType::Inches.to_metres(), 0.0254, epsilon = 1e-10);
    }

    // ── Constraints parsing ───────────────────────────────────────

    const CONSTRAINTS_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.25_max_hole_spacing.xml"
    );
    const CONSTRAINTS_FIPPLE_0HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/FippleFactorObjectiveFunction/0/0_holes.xml"
    );
    const CONSTRAINTS_FIPPLE_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/FippleFactorObjectiveFunction/6/6_holes.xml"
    );

    #[test]
    fn parse_hole_from_top_constraints() {
        let c = parse_constraints_xml(CONSTRAINTS_6HOLE_XML).expect("parse failed");
        assert_eq!(c.name, "1-1/4\" max spacing");
        assert_eq!(c.objective_function_name, "HoleFromTopObjectiveFunction");
        assert_eq!(c.number_of_holes, 6);
        assert_eq!(c.constraint_list.len(), 13);
    }

    #[test]
    fn hole_from_top_lower_bounds() {
        let c = parse_constraints_xml(CONSTRAINTS_6HOLE_XML).unwrap();
        let lb = c.lower_bounds();
        assert_eq!(lb.len(), 13);

        // Position constraints (7): bore length, fraction, 5 spacings
        assert_abs_diff_eq!(lb[0], 0.1905, epsilon = 1e-10);     // bore length
        assert_abs_diff_eq!(lb[1], 0.25, epsilon = 1e-10);       // fraction (dimensionless)
        assert_abs_diff_eq!(lb[2], 0.02032, epsilon = 1e-10);    // hole 6→5
        assert_abs_diff_eq!(lb[3], 0.02032, epsilon = 1e-10);    // hole 5→4
        assert_abs_diff_eq!(lb[4], 0.02032, epsilon = 1e-10);    // hole 4→3
        assert_abs_diff_eq!(lb[5], 0.02032, epsilon = 1e-10);    // hole 3→2
        assert_abs_diff_eq!(lb[6], 0.02032, epsilon = 1e-10);    // hole 2→1

        // Size constraints (6): hole diameters
        assert_abs_diff_eq!(lb[7], 0.0015875, epsilon = 1e-10);  // hole 6 (top)
        assert_abs_diff_eq!(lb[8], 0.003175, epsilon = 1e-10);   // hole 5
        assert_abs_diff_eq!(lb[9], 0.003175, epsilon = 1e-10);   // hole 4
        assert_abs_diff_eq!(lb[10], 0.003175, epsilon = 1e-10);  // hole 3
        assert_abs_diff_eq!(lb[11], 0.003175, epsilon = 1e-10);  // hole 2
        assert_abs_diff_eq!(lb[12], 0.003175, epsilon = 1e-10);  // hole 1 (bottom)
    }

    #[test]
    fn hole_from_top_upper_bounds() {
        let c = parse_constraints_xml(CONSTRAINTS_6HOLE_XML).unwrap();
        let ub = c.upper_bounds();
        assert_eq!(ub.len(), 13);

        // Position constraints
        assert_abs_diff_eq!(ub[0], 0.6985, epsilon = 1e-10);     // bore length
        assert_abs_diff_eq!(ub[1], 0.5, epsilon = 1e-10);        // fraction
        assert_abs_diff_eq!(ub[2], 0.03175, epsilon = 1e-10);    // hole 6→5
        assert_abs_diff_eq!(ub[3], 0.03175, epsilon = 1e-10);    // hole 5→4
        assert_abs_diff_eq!(ub[4], 0.06985, epsilon = 1e-10);    // hole 4→3 (wider gap)
        assert_abs_diff_eq!(ub[5], 0.03175, epsilon = 1e-10);    // hole 3→2
        assert_abs_diff_eq!(ub[6], 0.03175, epsilon = 1e-10);    // hole 2→1

        // Size constraints
        assert_abs_diff_eq!(ub[7], 0.0127, epsilon = 1e-10);     // hole 6
        assert_abs_diff_eq!(ub[8], 0.0127, epsilon = 1e-10);     // hole 5
        assert_abs_diff_eq!(ub[9], 0.0127, epsilon = 1e-10);     // hole 4
        assert_abs_diff_eq!(ub[10], 0.0127, epsilon = 1e-10);    // hole 3
        assert_abs_diff_eq!(ub[11], 0.0127, epsilon = 1e-10);    // hole 2
        assert_abs_diff_eq!(ub[12], 0.0127, epsilon = 1e-10);    // hole 1
    }

    #[test]
    fn parse_fipple_factor_constraints() {
        let c0 = parse_constraints_xml(CONSTRAINTS_FIPPLE_0HOLE_XML).unwrap();
        assert_eq!(c0.objective_function_name, "FippleFactorObjectiveFunction");
        assert_eq!(c0.number_of_holes, 0);
        assert_eq!(c0.constraint_list.len(), 1);
        let lb = c0.lower_bounds();
        let ub = c0.upper_bounds();
        assert_eq!(lb.len(), 1);
        assert_abs_diff_eq!(lb[0], 0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(ub[0], 1.5, epsilon = 1e-10);

        // 6-hole variant has same bounds
        let c6 = parse_constraints_xml(CONSTRAINTS_FIPPLE_6HOLE_XML).unwrap();
        assert_eq!(c6.number_of_holes, 6);
        assert_abs_diff_eq!(c6.lower_bounds()[0], 0.2, epsilon = 1e-10);
        assert_abs_diff_eq!(c6.upper_bounds()[0], 1.5, epsilon = 1e-10);
    }

    #[test]
    fn constraint_category_ordering_preserved() {
        let c = parse_constraints_xml(CONSTRAINTS_6HOLE_XML).unwrap();
        // First 7 should be "Hole position", last 6 should be "Hole size"
        for i in 0..7 {
            assert_eq!(c.constraint_list[i].category, "Hole position");
        }
        for i in 7..13 {
            assert_eq!(c.constraint_list[i].category, "Hole size");
        }
    }

    // ── All NAF instrument parsing ───────────────────────────────

    #[test]
    fn parse_all_naf_instruments() {
        let instruments: &[(&str, &str)] = &[
            ("0.5-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/0.5-bore_6-hole_NAF_starter.xml")),
            ("0.625-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/0.625-bore_6-hole_NAF_starter.xml")),
            ("0.75-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml")),
            ("0.875-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/0.875-bore_6-hole_NAF_starter.xml")),
            ("1.00-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/1.00-bore_6-hole_NAF_starter.xml")),
            ("1.25-bore", include_str!("../../../../oracle/v2.6.0/NafStudy/instruments/1.25-bore_6-hole_NAF_starter.xml")),
        ];

        for (label, xml) in instruments {
            let inst = parse_instrument_xml(xml)
                .unwrap_or_else(|e| panic!("Parse {label} failed: {e}"));
            assert_eq!(inst.holes.len(), 6, "{label}: expected 6 holes");
            assert_eq!(inst.bore_points.len(), 2, "{label}: expected 2 bore points");
            assert!(inst.mouthpiece.fipple.is_some(), "{label}: expected fipple mouthpiece");
            assert_eq!(inst.length_type, LengthType::Inches, "{label}: expected inches");
        }
    }

    // ── All NAF tuning parsing ───────────────────────────────────

    #[test]
    fn parse_all_naf_tunings() {
        let tunings: &[(&str, &str)] = &[
            ("A4", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/A4_ET_6-hole_NAF_chromatic_tuning.xml")),
            ("B3", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/B3_ET_6-hole_NAF_chromatic_tuning.xml")),
            ("C5", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/C5_ET_6-hole_NAF_chromatic_tuning.xml")),
            ("D#4", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/D#4_ET_6-hole_NAF_chromatic_tuning.xml")),
            ("D#5", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/D#5_ET_6-hole_NAF_chromatic_tuning.xml")),
            ("F#4", include_str!("../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml")),
        ];

        for (label, xml) in tunings {
            let tuning = parse_tuning_xml(xml)
                .unwrap_or_else(|e| panic!("Parse {label} tuning failed: {e}"));
            assert_eq!(tuning.fingerings.len(), 15, "{label}: expected 15 fingerings");
            assert_eq!(tuning.number_of_holes, 6, "{label}: expected 6 holes");

            for (i, f) in tuning.fingerings.iter().enumerate() {
                assert!(
                    f.note.frequency.is_some(),
                    "{label} fingering {i} ({}) missing frequency",
                    f.note.name,
                );
                assert_eq!(f.open_holes.len(), 6, "{label} fingering {i}: expected 6 hole states");
            }
        }
    }

    // ── All NAF constraint XMLs parse ────────────────────────────

    #[test]
    fn parse_all_naf_constraints() {
        // All 16 NAF constraint XMLs across all objective functions
        let constraints: &[(&str, &str, usize)] = &[
            // FippleFactorObjectiveFunction
            ("fipple_0hole", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/FippleFactorObjectiveFunction/0/0_holes.xml"), 1),
            ("fipple_6hole", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/FippleFactorObjectiveFunction/6/6_holes.xml"), 1),
            // HoleFromTopObjectiveFunction (4 spacing variants)
            ("hft_1.125", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.125_max_hole_spacing.xml"), 13),
            ("hft_1.25", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.25_max_hole_spacing.xml"), 13),
            ("hft_1.4", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.4_max_hole_spacing.xml"), 13),
            ("hft_1.5", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.5_max_hole_spacing.xml"), 13),
            // HoleGroupFromTopObjectiveFunction
            ("hgft_2grp", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleGroupFromTopObjectiveFunction/6/2-group_1.25_max_spacing.xml"), 11),
            // NafHoleSizeObjectiveFunction
            ("nhs_0.5", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/NafHoleSizeObjectiveFunction/6/0.5-max-hole-size.xml"), 6),
            // SingleTaperHoleGroupFromTopHemiHeadObjectiveFunction
            ("sthgfthh", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperHoleGroupFromTopHemiHeadObjectiveFunction/6/2-group_1.25-max-spacing.xml"), 14),
            // SingleTaperHoleGroupFromTopObjectiveFunction
            ("sthgft", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperHoleGroupFromTopObjectiveFunction/6/2-group_1.25-max-spacing.xml"), 14),
            // SingleTaperNoHoleGroupingFromTopHemiHeadObjectiveFunction
            ("stnhgfthh", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopHemiHeadObjectiveFunction/6/1.25_max_hole_spacing.xml"), 16),
            // SingleTaperNoHoleGroupingFromTopObjectiveFunction (5 spacing variants)
            ("stnhgft_1.0", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopObjectiveFunction/6/1.0_max_hole_spacing.xml"), 16),
            ("stnhgft_1.125", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopObjectiveFunction/6/1.125_max_hole_spacing.xml"), 16),
            ("stnhgft_1.25", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopObjectiveFunction/6/1.25_max_hole_spacing.xml"), 16),
            ("stnhgft_1.4", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopObjectiveFunction/6/1.4_max_hole_spacing.xml"), 16),
            ("stnhgft_1.5", include_str!("../../../../oracle/v2.6.0/constraints/NafStudyModel/SingleTaperNoHoleGroupingFromTopObjectiveFunction/6/1.5_max_hole_spacing.xml"), 16),
        ];

        for (label, xml, expected_count) in constraints {
            let c = parse_constraints_xml(xml)
                .unwrap_or_else(|e| panic!("Parse constraints {label} failed: {e}"));
            assert_eq!(
                c.constraint_list.len(),
                *expected_count,
                "{label}: expected {expected_count} constraints, got {}",
                c.constraint_list.len()
            );
            assert_eq!(c.number_of_holes, if label.contains("0hole") { 0 } else { 6 },
                "{label}: unexpected number_of_holes");

            // Verify bounds arrays match constraint count
            let lb = c.lower_bounds();
            let ub = c.upper_bounds();
            assert_eq!(lb.len(), *expected_count, "{label}: lower_bounds length mismatch");
            assert_eq!(ub.len(), *expected_count, "{label}: upper_bounds length mismatch");

            // Verify all lower bounds < upper bounds
            for i in 0..*expected_count {
                assert!(
                    lb[i] <= ub[i],
                    "{label}: lower_bound[{i}] ({}) > upper_bound[{i}] ({})",
                    lb[i], ub[i]
                );
            }
        }
    }
}
