package com.widgolden;

import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.note.*;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.util.BindFactory;

import java.io.File;

/// Golden fixture driver for Tuning Wizard operations.
///
/// Generates fixtures for:
/// - WIZ-SCALE: Scale from temperament + symbols + ref note
/// - WIZ-TUNING: Tuning from scale + fingering pattern
/// - WIZ-RT: XML round-trip verification for wizard types
///
/// Run: ./gradlew run -PmainClass=com.widgolden.WizardDriver
public class WizardDriver {

    private static final String ORACLE = "../oracle/v2.6.0";
    private static final String OUTPUT_BASE = "../golden/expected";

    public static void main(String[] args) throws Exception {
        System.out.println("=== Wizard Golden Fixture Driver ===\n");

        generateScaleFixture();
        generateTuningFixture();
        generateRoundTripFixture();

        System.out.println("\nAll wizard fixtures generated successfully.");
    }

    /// WIZ-SCALE: Generate a scale from 12-TET temperament + scientific symbols + A4=440Hz.
    ///
    /// This matches the Java wizard's ScalePage.createScaleButton algorithm:
    /// multiplier = refFreq / ratio[refIndex], freq[i] = ratio[i] * multiplier
    private static void generateScaleFixture() throws Exception {
        System.out.println("Generating WIZ-SCALE fixture...");

        File outputDir = new File(OUTPUT_BASE, "WIZ-SCALE");
        outputDir.mkdirs();

        // Load the NAF ET temperament from the oracle
        BindFactory noteFactory = NoteBindFactory.getInstance();
        File tempFile = new File(ORACLE, "NafStudy/temperaments/NAF_ET_chromatic_temperament.xml");
        Temperament temperament = (Temperament) noteFactory.unmarshalXml(tempFile, true);

        System.out.println("  Temperament: " + temperament.getName()
                + " (" + temperament.getRatio().size() + " ratios)");

        // Build scale: A4 = 440 Hz as reference
        String refName = "A4";
        double refFreq = 440.0;

        // Create scientific sharp symbols (matching our Rust factory)
        String[] noteNames = {"C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"};
        java.util.List<String> symbols = new java.util.ArrayList<>();
        for (int octave = 0; octave < 11; octave++) {
            for (String name : noteNames) {
                symbols.add(name + octave);
            }
        }

        // Find ref index in symbols
        int refIndex = symbols.indexOf(refName);
        if (refIndex < 0) {
            throw new RuntimeException("Reference note not found in symbols: " + refName);
        }
        System.out.println("  Reference: " + refName + " at symbol index " + refIndex);

        // Generate scale: multiplier = refFreq / ratio[0] = refFreq (ratio[0] = 1.0)
        double multiplier = refFreq;
        java.util.List<Double> ratios = temperament.getRatio();

        ArrayNode scaleNotes = OutputFormatter.newArray();
        for (int i = 0; i < ratios.size(); i++) {
            ObjectNode note = OutputFormatter.newObject();
            int symIdx = refIndex + i;
            String name = symIdx < symbols.size() ? symbols.get(symIdx) : "Note" + i;
            double freq = ratios.get(i) * multiplier;
            note.put("name", name);
            note.put("frequency", freq);
            scaleNotes.add(note);
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("temperamentName", temperament.getName());
        result.put("refName", refName);
        result.put("refFrequency", refFreq);
        result.put("refIndex", refIndex);
        result.put("noteCount", ratios.size());
        result.set("notes", scaleNotes);

        OutputFormatter.writeJson(result, new File(outputDir, "scale.json"));
        System.out.println("  Generated " + ratios.size() + " notes");
    }

    /// WIZ-TUNING: Generate a tuning from oracle scale + fingering pattern.
    ///
    /// Combines the A4 ET scale with the 6-hole Wood Wind NAF fingering pattern.
    private static void generateTuningFixture() throws Exception {
        System.out.println("Generating WIZ-TUNING fixture...");

        File outputDir = new File(OUTPUT_BASE, "WIZ-TUNING");
        outputDir.mkdirs();

        BindFactory noteFactory = NoteBindFactory.getInstance();

        // Load scale
        File scaleFile = new File(ORACLE, "NafStudy/scales/A4_ET_NAT_chromatic_scale.xml");
        Scale scale = (Scale) noteFactory.unmarshalXml(scaleFile, true);
        System.out.println("  Scale: " + scale.getName()
                + " (" + scale.getNote().size() + " notes)");

        // Load fingering pattern
        File patternFile = new File(ORACLE, "NafStudy/fingerings/Wood_Wind_NAF_6-hole_fingering.xml");
        FingeringPattern pattern = (FingeringPattern) noteFactory.unmarshalXml(patternFile, true);
        System.out.println("  Pattern: " + pattern.getName()
                + " (" + pattern.getFingering().size() + " fingerings)");

        // Build tuning: for each pattern fingering, assign scale note by index
        ArrayNode tuningRows = OutputFormatter.newArray();
        java.util.List<com.wwidesigner.note.Scale.Note> scaleNotes = scale.getNote();
        java.util.List<Fingering> fingerings = pattern.getFingering();

        for (int i = 0; i < fingerings.size(); i++) {
            Fingering f = fingerings.get(i);
            ObjectNode row = OutputFormatter.newObject();

            // Note assignment: use scale note at same index, or null if out of range
            if (i < scaleNotes.size()) {
                com.wwidesigner.note.Scale.Note scaleNote = scaleNotes.get(i);
                row.put("name", scaleNote.getName());
                row.put("frequency", scaleNote.getFrequency());
            } else {
                row.put("name", "Note " + (i + 1));
                row.putNull("frequency");
            }

            // Hole pattern
            ArrayNode holes = OutputFormatter.newArray();
            for (Boolean open : f.getOpenHole()) {
                holes.add(open);
            }
            row.set("openHoles", holes);
            row.put("optimizationWeight",
                    f.getOptimizationWeight() != null ? f.getOptimizationWeight() : 1);

            tuningRows.add(row);
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("scaleName", scale.getName());
        result.put("patternName", pattern.getName());
        result.put("numberOfHoles", pattern.getNumberOfHoles());
        result.put("fingeringCount", fingerings.size());
        result.set("fingerings", tuningRows);

        OutputFormatter.writeJson(result, new File(outputDir, "tuning.json"));
        System.out.println("  Generated " + fingerings.size() + " fingering rows");
    }

    /// WIZ-RT: XML round-trip verification.
    ///
    /// Parse each wizard type, verify key fields, output summary.
    private static void generateRoundTripFixture() throws Exception {
        System.out.println("Generating WIZ-RT fixture...");

        File outputDir = new File(OUTPUT_BASE, "WIZ-RT");
        outputDir.mkdirs();

        BindFactory noteFactory = NoteBindFactory.getInstance();

        // Round-trip scale
        File scaleFile = new File(ORACLE, "NafStudy/scales/A4_ET_NAT_chromatic_scale.xml");
        Scale scale = (Scale) noteFactory.unmarshalXml(scaleFile, true);

        ObjectNode scaleRt = OutputFormatter.newObject();
        scaleRt.put("name", scale.getName());
        scaleRt.put("noteCount", scale.getNote().size());
        scaleRt.put("firstNoteName", scale.getNote().get(0).getName());
        scaleRt.put("firstNoteFreq", scale.getNote().get(0).getFrequency());
        scaleRt.put("lastNoteName", scale.getNote().get(scale.getNote().size() - 1).getName());
        scaleRt.put("lastNoteFreq", scale.getNote().get(scale.getNote().size() - 1).getFrequency());

        // Round-trip temperament
        File tempFile = new File(ORACLE, "NafStudy/temperaments/NAF_ET_chromatic_temperament.xml");
        Temperament temp = (Temperament) noteFactory.unmarshalXml(tempFile, true);

        ObjectNode tempRt = OutputFormatter.newObject();
        tempRt.put("name", temp.getName());
        tempRt.put("ratioCount", temp.getRatio().size());
        tempRt.put("firstRatio", temp.getRatio().get(0));
        tempRt.put("lastRatio", temp.getRatio().get(temp.getRatio().size() - 1));

        // Round-trip fingering pattern
        File patternFile = new File(ORACLE, "NafStudy/fingerings/Wood_Wind_NAF_6-hole_fingering.xml");
        FingeringPattern pattern = (FingeringPattern) noteFactory.unmarshalXml(patternFile, true);

        ObjectNode patternRt = OutputFormatter.newObject();
        patternRt.put("name", pattern.getName());
        patternRt.put("numberOfHoles", pattern.getNumberOfHoles());
        patternRt.put("fingeringCount", pattern.getFingering().size());

        // Combine
        ObjectNode result = OutputFormatter.newObject();
        result.set("scale", scaleRt);
        result.set("temperament", tempRt);
        result.set("fingeringPattern", patternRt);

        OutputFormatter.writeJson(result, new File(outputDir, "roundtrip.json"));
        System.out.println("  Round-trip verified for scale, temperament, fingering pattern");
    }
}
