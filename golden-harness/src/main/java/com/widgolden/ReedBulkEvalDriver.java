package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
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
 * Bulk evaluation of all Reed instrument x tuning combinations.
 *
 * Tests 7 compatible combos (hole counts must match):
 * <ul>
 *   <li>SampleChanter (8 holes) x {A3-ClosedFingering, D4-uilleann-ET,
 *       D4-uilleann-JI, D4-union-ET} — 4 combos</li>
 *   <li>ReiswigChanter (10 holes) x {A3-Reiswig} — 1 combo</li>
 *   <li>Didgeridoo-2stage (0 holes) x {Didgeridoo-D2-D3} — 1 combo</li>
 *   <li>Didgeridoo-3stage (0 holes) x {Didgeridoo-D2-D3} — 1 combo</li>
 * </ul>
 *
 * Uses {@code SimpleReedCalculator} + {@code SimpleInstrumentTuner},
 * matching the wiring in {@code ReedStudyModel}.
 *
 * Output: golden/expected/RD-BULK-EVAL/all_evals.json
 */
public class ReedBulkEvalDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/RD-BULK-EVAL";

    private static final String[] INSTRUMENTS = {
        "SampleChanter.xml",
        "ReiswigChanter.xml",
        "Didgeridoo-2stage-D2-D3.xml",
        "Didgeridoo-3stage-D2-D3.xml",
    };

    private static final String[] TUNINGS = {
        "A3-ClosedFingering.xml",
        "A3-Reiswig.xml",
        "D4-uilleann-ET-tuning.xml",
        "D4-uilleann-JI-tuning.xml",
        "D4-union-ET-tuning.xml",
        "Didgeridoo-D2-D3-tuning.xml",
    };

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        ArrayNode allResults = mapper.createArrayNode();

        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/ReedStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/ReedStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        int total = 0;
        int skipped = 0;
        int failures = 0;

        for (String instrFile : INSTRUMENTS) {
            for (String tuningFile : TUNINGS) {
                String comboName = instrFile.replace(".xml", "") + "__"
                    + tuningFile.replace(".xml", "");

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

                    // Check hole count compatibility
                    int instrHoles = instrument.getHole() != null ? instrument.getHole().size() : 0;
                    int tuningHoles = tuning.getNumberOfHoles();
                    if (instrHoles != tuningHoles) {
                        skipped++;
                        System.out.printf("Skipping: %s (%d holes) + %s (%d holes) - mismatch%n",
                            instrFile, instrHoles, tuningFile, tuningHoles);
                        continue;
                    }

                    total++;
                    System.out.printf("Evaluating: %s + %s%n", instrFile, tuningFile);

                    // Create calculator (SimpleReedCalculator — no LinearV tuner)
                    InstrumentCalculator calculator = new SimpleReedCalculator();
                    calculator.setInstrument(instrument);
                    calculator.setPhysicalParameters(params);

                    // Simple reed uses SimpleInstrumentTuner (standard reactance-zero search)
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
                    total++;
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

        File outFile = new File(outDir, "all_evals.json");
        mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, allResults);

        System.out.printf("%nBulk eval complete: %d evaluated, %d skipped (hole mismatch), %d failed%n",
            total - failures, skipped, failures);
        System.out.println("Output: " + outFile.getAbsolutePath());
    }
}
