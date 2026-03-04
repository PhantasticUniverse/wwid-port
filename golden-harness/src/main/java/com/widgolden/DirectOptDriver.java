package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.optimization.*;
import com.wwidesigner.optimization.BoreLengthAdjustmentInterface.BoreLengthAdjustmentType;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;

import java.io.File;

/**
 * Generate golden fixtures for DIRECT-C → BOBYQA global optimization.
 *
 * Runs GlobalHoleObjectiveFunction on SamplePVC-Whistle through
 * ObjectiveFunctionOptimizer.optimizeObjectiveFunction(), which internally:
 *   1. DIRECT-C global search (convergence 7e-8, target 0.001, 2× maxEval)
 *   2. BOBYQA local refinement from DIRECT-C's best point
 *   3. Keeps whichever result is better
 *
 * Also runs GlobalHolePositionObjectiveFunction for a position-only variant.
 *
 * Output: golden/expected/DIRECT-01/
 *   - global_hole.json
 *   - global_hole_position.json
 */
public class DirectOptDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/DIRECT-01";
    private static final int BLOWING_LEVEL = 5;

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/WhistleStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/WhistleStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        String instrFile = "SamplePVC-Whistle.xml";
        String tuningFile = "SamplePVC-tuning.xml";

        System.out.println("=== DIRECT-C Global Optimization Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // Load the merged constraints (13 total: 7 position + 6 size)
        Constraints fullConstraints = loadConstraints(
            "WhistleStudyModel/HoleObjectiveFunction/DefaultHoleConstraints.xml");

        // --- GlobalHole optimization (DIRECT-C → BOBYQA, merged position + size) ---
        {
            System.out.println("--- GlobalHole Optimization (DIRECT-C → BOBYQA) ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            GlobalHoleObjectiveFunction objective =
                new GlobalHoleObjectiveFunction(calculator, tuning, evaluator,
                    BoreLengthAdjustmentType.PRESERVE_TAPER);

            // Set merged constraints
            objective.setConstraintsBounds(fullConstraints);

            double[] initialGeometry = objective.getGeometryPoint();
            double initialNorm = objective.value(initialGeometry);

            System.out.printf("  Initial norm: %.6f  (%d dims, maxEval=%d)%n",
                initialNorm, objective.getNrDimensions(), objective.getMaxEvaluations());

            // Run through ObjectiveFunctionOptimizer (DIRECT-C → BOBYQA)
            ObjectiveFunctionOptimizer.optimizeObjectiveFunction(
                objective, objective.getOptimizerType());

            double[] finalGeometry = objective.getGeometryPoint();
            double finalNorm = ObjectiveFunctionOptimizer.getFinalNorm();
            int totalEvals = objective.getNumberOfEvaluations();

            ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, totalEvals,
                initialGeometry, finalGeometry);
            out.put("optimizer", "GlobalHoleObjectiveFunction");
            out.put("strategy", "DIRECT-C → BOBYQA");

            File outFile = new File(outDir, "global_hole.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  GlobalHole: norm %.6f -> %.6f (%d evals)%n",
                initialNorm, finalNorm, totalEvals);
        }

        // --- GlobalHolePosition optimization (DIRECT-C → BOBYQA, position only) ---
        {
            System.out.println("\n--- GlobalHolePosition Optimization (DIRECT-C → BOBYQA) ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            GlobalHolePositionObjectiveFunction objective =
                new GlobalHolePositionObjectiveFunction(calculator, tuning, evaluator,
                    BoreLengthAdjustmentType.PRESERVE_TAPER);

            // Extract position-only bounds (first 7 constraints)
            java.util.List<Constraint> allC = fullConstraints.getConstraint();
            int nHoles = 6;
            int nPosition = nHoles + 1;
            double[] posLower = new double[nPosition];
            double[] posUpper = new double[nPosition];
            for (int i = 0; i < nPosition; i++) {
                posLower[i] = allC.get(i).getLowerBound();
                posUpper[i] = allC.get(i).getUpperBound();
            }
            objective.setLowerBounds(posLower);
            objective.setUpperBounds(posUpper);

            double[] initialGeometry = objective.getGeometryPoint();
            double initialNorm = objective.value(initialGeometry);

            System.out.printf("  Initial norm: %.6f  (%d dims, maxEval=%d)%n",
                initialNorm, objective.getNrDimensions(), objective.getMaxEvaluations());

            // Run through ObjectiveFunctionOptimizer (DIRECT-C → BOBYQA)
            ObjectiveFunctionOptimizer.optimizeObjectiveFunction(
                objective, objective.getOptimizerType());

            double[] finalGeometry = objective.getGeometryPoint();
            double finalNorm = ObjectiveFunctionOptimizer.getFinalNorm();
            int totalEvals = objective.getNumberOfEvaluations();

            ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, totalEvals,
                initialGeometry, finalGeometry);
            out.put("optimizer", "GlobalHolePositionObjectiveFunction");
            out.put("strategy", "DIRECT-C → BOBYQA");

            File outFile = new File(outDir, "global_hole_position.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  GlobalHolePosition: norm %.6f -> %.6f (%d evals)%n",
                initialNorm, finalNorm, totalEvals);
        }

        System.out.println("\nAll DIRECT-01 fixtures written to " + outDir.getAbsolutePath());
    }

    private static Instrument loadInstrument(File dir, String filename) throws Exception {
        BindFactory factory = GeometryBindFactory.getInstance();
        Instrument instrument = (Instrument) factory.unmarshalXml(
            new File(dir, filename), true);
        instrument.updateComponents();
        return instrument;
    }

    private static Tuning loadTuning(File dir, String filename) throws Exception {
        BindFactory factory = NoteBindFactory.getInstance();
        return (Tuning) factory.unmarshalXml(new File(dir, filename), true);
    }

    private static Constraints loadConstraints(String relativePath) throws Exception {
        String fullPath = ORACLE_BASE + "/constraints/" + relativePath;
        BindFactory factory = com.wwidesigner.optimization.bind.OptimizationBindFactory.getInstance();
        return (Constraints) factory.unmarshalXml(new File(fullPath).getCanonicalFile(), true);
    }
}
