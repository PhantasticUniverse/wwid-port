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

import org.apache.commons.math3.optim.*;
import org.apache.commons.math3.optim.nonlinear.scalar.GoalType;
import org.apache.commons.math3.optim.nonlinear.scalar.ObjectiveFunction;
import org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer;

import java.io.File;
import java.util.List;

/**
 * Generate optimization golden fixtures for Whistle study model.
 *
 * Runs HoleSizeObjectiveFunction (N-dim BOBYQA),
 * HolePositionObjectiveFunction ((N+1)-dim BOBYQA), and
 * HoleObjectiveFunction (merged (2N+1)-dim BOBYQA)
 * against SamplePVC-Whistle instrument + tuning.
 *
 * Output: golden/expected/WH-OPT/
 *   - opt_hole_size.json
 *   - opt_hole_position.json
 *   - opt_hole.json
 */
public class WhistleOptDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/WH-OPT";
    private static final int BLOWING_LEVEL = 5;

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/WhistleStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/WhistleStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        // Use SamplePVC-Whistle for hole optimization (standard 6-hole whistle)
        String instrFile = "SamplePVC-Whistle.xml";
        String tuningFile = "SamplePVC-tuning.xml";

        System.out.println("=== Whistle Optimization Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // Load the merged constraints (13 total: 7 position + 6 size)
        Constraints fullConstraints = loadConstraints(
            "WhistleStudyModel/HoleObjectiveFunction/DefaultHoleConstraints.xml");
        List<Constraint> allC = fullConstraints.getConstraint();
        int nHoles = 6;
        int nPosition = nHoles + 1;  // 7

        // Extract position bounds (first 7 constraints)
        double[] posLower = new double[nPosition];
        double[] posUpper = new double[nPosition];
        for (int i = 0; i < nPosition; i++) {
            posLower[i] = allC.get(i).getLowerBound();
            posUpper[i] = allC.get(i).getUpperBound();
        }

        // Extract size bounds (last 6 constraints)
        double[] sizeLower = new double[nHoles];
        double[] sizeUpper = new double[nHoles];
        for (int i = 0; i < nHoles; i++) {
            sizeLower[i] = allC.get(nPosition + i).getLowerBound();
            sizeUpper[i] = allC.get(nPosition + i).getUpperBound();
        }

        // --- HoleSize optimization ---
        {
            System.out.println("--- HoleSize Optimization ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HoleSizeObjectiveFunction objective =
                new HoleSizeObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // Set diameter-only bounds (last 6 constraints from merged file)
            objective.setLowerBounds(sizeLower);
            objective.setUpperBounds(sizeUpper);

            double initialNorm = objective.value(initialGeometry);

            // Run BOBYQA
            int nrDims = objective.getNrDimensions();
            int maxEval = objective.getMaxEvaluations();
            double[] initialPoint = objective.getInitialPoint();
            double initTrust = objective.getInitialTrustRegionRadius(initialPoint);
            double stopTrust = objective.getStoppingTrustRegionRadius();

            BOBYQAOptimizer optimizer = new BOBYQAOptimizer(
                2 * nrDims + 1, initTrust, stopTrust);

            PointValuePair result = optimizer.optimize(
                new MaxEval(maxEval),
                new ObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new InitialGuess(initialPoint),
                new SimpleBounds(objective.getLowerBounds(), objective.getUpperBounds()));

            objective.setGeometryPoint(result.getPoint());
            double finalNorm = result.getValue();
            double[] finalGeometry = objective.getGeometryPoint();
            int evaluations = optimizer.getEvaluations();

            ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations,
                initialGeometry, finalGeometry);
            out.put("optimizer", "HoleSizeObjectiveFunction");

            File outFile = new File(outDir, "opt_hole_size.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  HoleSize: norm %.6f -> %.6f (%d dims, %d evals)%n",
                initialNorm, finalNorm, nrDims, evaluations);
        }

        // --- HolePosition optimization ---
        {
            System.out.println("--- HolePosition Optimization ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HolePositionObjectiveFunction objective =
                new HolePositionObjectiveFunction(calculator, tuning, evaluator,
                    BoreLengthAdjustmentType.PRESERVE_TAPER);

            double[] initialGeometry = objective.getGeometryPoint();

            // Set position-only bounds (first 7 constraints from merged file)
            objective.setLowerBounds(posLower);
            objective.setUpperBounds(posUpper);

            double initialNorm = objective.value(initialGeometry);

            int nrDims = objective.getNrDimensions();
            int maxEval = objective.getMaxEvaluations();
            double[] initialPoint = objective.getInitialPoint();
            double initTrust = objective.getInitialTrustRegionRadius(initialPoint);
            double stopTrust = objective.getStoppingTrustRegionRadius();

            BOBYQAOptimizer optimizer = new BOBYQAOptimizer(
                2 * nrDims + 1, initTrust, stopTrust);

            PointValuePair result = optimizer.optimize(
                new MaxEval(maxEval),
                new ObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new InitialGuess(initialPoint),
                new SimpleBounds(objective.getLowerBounds(), objective.getUpperBounds()));

            objective.setGeometryPoint(result.getPoint());
            double finalNorm = result.getValue();
            double[] finalGeometry = objective.getGeometryPoint();
            int evaluations = optimizer.getEvaluations();

            ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations,
                initialGeometry, finalGeometry);
            out.put("optimizer", "HolePositionObjectiveFunction");

            File outFile = new File(outDir, "opt_hole_position.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  HolePosition: norm %.6f -> %.6f (%d dims, %d evals)%n",
                initialNorm, finalNorm, nrDims, evaluations);
        }

        // --- Hole (merged position + size) optimization ---
        {
            System.out.println("--- Hole (merged) Optimization ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HoleObjectiveFunction objective =
                new HoleObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // Full merged constraints (13: 7 position + 6 size)
            objective.setConstraintsBounds(fullConstraints);

            double initialNorm = objective.value(initialGeometry);

            int nrDims = objective.getNrDimensions();
            int maxEval = objective.getMaxEvaluations();
            double[] initialPoint = objective.getInitialPoint();
            double initTrust = objective.getInitialTrustRegionRadius(initialPoint);
            double stopTrust = objective.getStoppingTrustRegionRadius();

            BOBYQAOptimizer optimizer = new BOBYQAOptimizer(
                2 * nrDims + 1, initTrust, stopTrust);

            PointValuePair result = optimizer.optimize(
                new MaxEval(maxEval),
                new ObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new InitialGuess(initialPoint),
                new SimpleBounds(objective.getLowerBounds(), objective.getUpperBounds()));

            objective.setGeometryPoint(result.getPoint());
            double finalNorm = result.getValue();
            double[] finalGeometry = objective.getGeometryPoint();
            int evaluations = optimizer.getEvaluations();

            ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations,
                initialGeometry, finalGeometry);
            out.put("optimizer", "HoleObjectiveFunction");

            File outFile = new File(outDir, "opt_hole.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  Hole: norm %.6f -> %.6f (%d dims, %d evals)%n",
                initialNorm, finalNorm, nrDims, evaluations);
        }

        System.out.println("\nAll optimization fixtures written to " + outDir.getAbsolutePath());
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
