package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Fingering;
import com.wwidesigner.note.Note;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;

import java.io.File;
import java.util.*;

/**
 * Bulk evaluation of all Flute instrument x tuning combinations.
 *
 * Generates golden reference data for parity testing: for each combination
 * where hole counts match, evaluates all fingerings and records note name,
 * target freq, predicted freq, and cents deviation.
 *
 * Uses FluteCalculator + LinearVInstrumentTuner(5), matching the wiring
 * in FluteStudyModel (which extends WhistleStudyModel).
 *
 * Output: golden/expected/FLUTE-BULK-EVAL/all_evals.json
 */
public class FluteBulkEvalDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/FLUTE-BULK-EVAL";

    // Both Flute instruments (6 holes each)
    private static final String[] INSTRUMENTS = {
        "SamplePVC-Flute.xml",
        "fife.xml",
    };

    // All 6 Flute tunings (hole-count filtering applied at runtime)
    private static final String[] TUNINGS = {
        "D4-Equal.xml",
        "D4-Equal-3octave.xml",
        "C4-Equal-8key-3octave.xml",
        "fife-tuning.xml",
        "BellNoteTuning-0hole.xml",
        "BellNoteTuning-6hole.xml",
    };

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        ArrayNode allResults = mapper.createArrayNode();

        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/FluteStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/FluteStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        int total = 0;
        int skipped = 0;
        int failures = 0;

        for (String instrFile : INSTRUMENTS) {
            // Load instrument to get hole count
            BindFactory geoFactory = GeometryBindFactory.getInstance();
            Instrument instrument = (Instrument) geoFactory.unmarshalXml(
                new File(instrDir, instrFile), true);
            instrument.updateComponents();
            int instrHoles = instrument.getHole().size();

            for (String tuningFile : TUNINGS) {
                // Load tuning to check hole count
                BindFactory noteFactory = NoteBindFactory.getInstance();
                Tuning tuning = (Tuning) noteFactory.unmarshalXml(
                    new File(tuningDir, tuningFile), true);

                if (tuning.getNumberOfHoles() != instrHoles) {
                    System.out.printf("Skipping: %s (%d holes) + %s (%d holes) — hole count mismatch%n",
                        instrFile, instrHoles, tuningFile, tuning.getNumberOfHoles());
                    skipped++;
                    continue;
                }

                total++;
                String comboName = instrFile.replace(".xml", "") + "__"
                    + tuningFile.replace(".xml", "");
                System.out.printf("Evaluating: %s + %s%n", instrFile, tuningFile);

                try {
                    // Create calculator (same wiring as FluteStudyModel)
                    InstrumentCalculator calculator = new FluteCalculator();
                    calculator.setInstrument(instrument);
                    calculator.setPhysicalParameters(params);

                    // Set up tuner (needs tuning set before predictions work)
                    LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(5);
                    tuner.setCalculator(calculator);
                    tuner.setInstrument(instrument);
                    tuner.setParams(params);

                    List<Fingering> fingeringTargets = tuning.getFingering();

                    // Set tuning on tuner to establish velocity interpolation
                    Tuning tuningWrapper = new Tuning();
                    tuningWrapper.setFingering(fingeringTargets);
                    tuner.setTuning(tuningWrapper);

                    // Compute cents per fingering individually to avoid
                    // CentDeviationEvaluator's index-shift bug when fingerings
                    // lack target frequencies (e.g., fife-tuning octave markers).
                    ObjectNode combo = mapper.createObjectNode();
                    combo.put("instrument", instrFile);
                    combo.put("tuning", tuningFile);
                    combo.put("name", comboName);
                    combo.put("numFingerings", fingeringTargets.size());

                    ArrayNode fingerings = mapper.createArrayNode();
                    for (int i = 0; i < fingeringTargets.size(); i++) {
                        Fingering f = fingeringTargets.get(i);
                        ObjectNode fNode = mapper.createObjectNode();

                        double centDeviation;
                        if (f.getNote() != null) {
                            fNode.put("note", f.getNote().getName() != null
                                ? f.getNote().getName() : "");
                            if (f.getNote().getFrequency() != null) {
                                fNode.put("targetFreq", f.getNote().getFrequency());
                            }
                            Double predicted = tuner.predictedFrequency(f);
                            fNode.put("predictedFreq", predicted);

                            if (f.getNote().getFrequency() != null && predicted != null) {
                                centDeviation = com.wwidesigner.note.Note.cents(
                                    f.getNote().getFrequency(), predicted);
                            } else {
                                // No target frequency → 0 cents (not included in optimization)
                                centDeviation = 0.0;
                            }
                        } else {
                            centDeviation = 0.0;
                        }
                        fNode.put("cents", centDeviation);
                        fingerings.add(fNode);
                    }
                    combo.set("fingerings", fingerings);
                    allResults.add(combo);

                    System.out.printf("  OK: %d fingerings evaluated%n",
                        fingeringTargets.size());
                } catch (Exception e) {
                    failures++;
                    ObjectNode combo = mapper.createObjectNode();
                    combo.put("instrument", instrFile);
                    combo.put("tuning", tuningFile);
                    combo.put("name", comboName);
                    combo.put("error", e.getMessage());
                    allResults.add(combo);
                    System.err.printf("  FAILED: %s%n", e.getMessage());
                }
            }
        }

        // Write output
        File outFile = new File(outDir, "all_evals.json");
        mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, allResults);

        System.out.printf("%nBulk eval complete: %d evaluated, %d skipped (hole mismatch), %d failed%n",
            total - failures, skipped, failures);
        System.out.println("Output: " + outFile.getAbsolutePath());
    }
}
