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
 * Generate optimization golden fixtures for Flute study model.
 *
 * Exercises three hole optimization objective functions:
 * <ol>
 *   <li>{@code HoleSizeObjectiveFunction} — N-dim BOBYQA, optimizes hole diameters</li>
 *   <li>{@code HolePositionObjectiveFunction} — (N+1)-dim BOBYQA, optimizes positions</li>
 *   <li>{@code HoleObjectiveFunction} — (2N+1)-dim BOBYQA, optimizes both</li>
 * </ol>
 *
 * Uses {@code FluteCalculator} with {@code LinearVInstrumentTuner(BLOWING_LEVEL=5)}
 * and {@code CentDeviationEvaluator} at 72°F.
 *
 * Instrument: SamplePVC-Flute.xml (6 holes), Tuning: D4-Equal.xml.
 * Constraints from oracle FluteStudyModel constraint files.
 *
 * Output: golden/expected/FL-OPT/
 *   - opt_hole_size.json
 *   - opt_hole_position.json
 *   - opt_hole.json
 */
public class FluteOptDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/FL-OPT";
    private static final int BLOWING_LEVEL = 5;

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/FluteStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/FluteStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        String instrFile = "SamplePVC-Flute.xml";
        String tuningFile = "D4-Equal.xml";

        System.out.println("=== Flute Optimization Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // Load the merged constraints (13 total: 7 position + 6 size)
        Constraints fullConstraints = loadConstraints(
            "FluteStudyModel/HoleObjectiveFunction/LargeHoleSize_Spacing_6holes.xml");
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

        // Load size-only constraints from dedicated file
        Constraints sizeConstraints = loadConstraints(
            "FluteStudyModel/HoleSizeObjectiveFunction/LargeHoleSize_6holes.xml");

        // --- HoleSize optimization ---
        {
            System.out.println("--- HoleSize Optimization ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HoleSizeObjectiveFunction objective =
                new HoleSizeObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            objective.setConstraintsBounds(sizeConstraints);

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

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HolePositionObjectiveFunction objective =
                new HolePositionObjectiveFunction(calculator, tuning, evaluator,
                    BoreLengthAdjustmentType.PRESERVE_TAPER);

            double[] initialGeometry = objective.getGeometryPoint();

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

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            HoleObjectiveFunction objective =
                new HoleObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

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
