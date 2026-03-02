package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.geometry.calculation.DefaultHoleCalculator;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Fingering;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;

import java.io.File;
import java.util.*;

/**
 * Bulk evaluation of all NAF instrument × tuning combinations.
 *
 * Generates golden reference data for parity testing: for each combination,
 * evaluates all fingerings and records note name, target freq, predicted freq,
 * and cents deviation.
 *
 * Output: golden/expected/NAF-BULK-EVAL/all_evals.json
 */
public class NafBulkEvalDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/NAF-BULK-EVAL";

    // All 6 NAF instruments (bore sizes in inches)
    private static final String[] INSTRUMENTS = {
        "0.5-bore_6-hole_NAF_starter.xml",
        "0.625-bore_6-hole_NAF_starter.xml",
        "0.75-bore_6-hole_NAF_starter.xml",
        "0.875-bore_6-hole_NAF_starter.xml",
        "1.00-bore_6-hole_NAF_starter.xml",
        "1.25-bore_6-hole_NAF_starter.xml",
    };

    // All 6 NAF tunings
    private static final String[] TUNINGS = {
        "A4_ET_6-hole_NAF_chromatic_tuning.xml",
        "B3_ET_6-hole_NAF_chromatic_tuning.xml",
        "C5_ET_6-hole_NAF_chromatic_tuning.xml",
        "D#4_ET_6-hole_NAF_chromatic_tuning.xml",
        "D#5_ET_6-hole_NAF_chromatic_tuning.xml",
        "F#4_ET_6-hole_NAF_chromatic_tuning.xml",
    };

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        ArrayNode allResults = mapper.createArrayNode();

        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/NafStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/NafStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        int total = 0;
        int failures = 0;

        for (String instrFile : INSTRUMENTS) {
            for (String tuningFile : TUNINGS) {
                total++;
                String comboName = instrFile.replace(".xml", "") + "__"
                    + tuningFile.replace(".xml", "");
                System.out.printf("Evaluating: %s + %s%n", instrFile, tuningFile);

                try {
                    // Load instrument
                    BindFactory geoFactory = GeometryBindFactory.getInstance();
                    Instrument instrument = (Instrument) geoFactory.unmarshalXml(
                        new File(instrDir, instrFile), true);
                    instrument.updateComponents();

                    // Load tuning
                    BindFactory noteFactory = NoteBindFactory.getInstance();
                    Tuning tuning = (Tuning) noteFactory.unmarshalXml(
                        new File(tuningDir, tuningFile), true);

                    // Create calculator (same wiring as NafStudyModel)
                    InstrumentCalculator calculator = new NAFCalculator();
                    calculator.setInstrument(instrument);
                    calculator.setPhysicalParameters(params);

                    // Create evaluator
                    SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
                    CentDeviationEvaluator evaluator =
                        new CentDeviationEvaluator(calculator, tuner);

                    // Evaluate
                    List<Fingering> fingeringTargets = tuning.getFingering();
                    double[] errorVector = evaluator.calculateErrorVector(fingeringTargets);

                    // Build result
                    ObjectNode combo = mapper.createObjectNode();
                    combo.put("instrument", instrFile);
                    combo.put("tuning", tuningFile);
                    combo.put("name", comboName);
                    combo.put("numFingerings", fingeringTargets.size());

                    ArrayNode fingerings = mapper.createArrayNode();
                    for (int i = 0; i < fingeringTargets.size(); i++) {
                        Fingering f = fingeringTargets.get(i);
                        ObjectNode fNode = mapper.createObjectNode();

                        if (f.getNote() != null) {
                            fNode.put("note", f.getNote().getName() != null
                                ? f.getNote().getName() : "");
                            if (f.getNote().getFrequency() != null) {
                                fNode.put("targetFreq", f.getNote().getFrequency());
                            }
                            fNode.put("predictedFreq", tuner.predictedFrequency(f));
                        }
                        fNode.put("cents", errorVector[i]);
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

        System.out.printf("%nBulk eval complete: %d/%d succeeded, %d failed%n",
            total - failures, total, failures);
        System.out.println("Output: " + outFile.getAbsolutePath());
    }
}
