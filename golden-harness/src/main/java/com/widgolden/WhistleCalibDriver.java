package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.optimization.*;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;

import org.apache.commons.math3.optim.univariate.*;
import org.apache.commons.math3.optim.*;
import org.apache.commons.math3.optim.nonlinear.scalar.GoalType;
import org.apache.commons.math3.optim.nonlinear.scalar.ObjectiveFunction;
import org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer;

import java.io.File;

/**
 * Generate calibration golden fixtures for Whistle study model.
 *
 * Runs WindowHeightObjectiveFunction (1D Brent),
 * BetaObjectiveFunction (1D Brent), and
 * WhistleCalibrationObjectiveFunction (2D BOBYQA)
 * against FeadogMk1 instrument + tuning.
 *
 * Output: golden/expected/WH-CAL/
 *   - calib_window_height.json
 *   - calib_beta.json
 *   - calib_joint.json
 */
public class WhistleCalibDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/WH-CAL";
    private static final int BLOWING_LEVEL = 5;

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/WhistleStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/WhistleStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        // Use FeadogMk1 which has explicit windowHeight and beta
        String instrFile = "FeadogMk1.xml";
        String tuningFile = "FeadogMk1-tuning.xml";

        System.out.println("=== Whistle Calibration Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // --- WindowHeight calibration (1D Brent) ---
        {
            System.out.println("--- WindowHeight Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FmaxEvaluator evaluator = new FmaxEvaluator(calculator, tuner);

            WindowHeightObjectiveFunction objective =
                new WindowHeightObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // Load default constraints (bounds)
            Constraints constraints = loadConstraints(
                "WhistleStudyModel/WindowHeightObjectiveFunction/WindowHeightDefaultConstraints.xml");
            objective.setConstraintsBounds(constraints);

            double initialNorm = objective.value(initialGeometry);

            // Run Brent optimizer (1D)
            double[] lowerBounds = objective.getLowerBounds();
            double[] upperBounds = objective.getUpperBounds();

            BrentOptimizer optimizer = new BrentOptimizer(1e-4, 1e-4);
            UnivariatePointValuePair result = optimizer.optimize(
                new MaxEval(100),
                new UnivariateObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new SearchInterval(lowerBounds[0], upperBounds[0], initialGeometry[0]));

            // Apply the result
            objective.setGeometryPoint(new double[]{result.getPoint()});
            double finalNorm = result.getValue();

            double[] finalGeometry = objective.getGeometryPoint();

            ObjectNode out = mapper.createObjectNode();
            out.put("optimizer", "WindowHeightObjectiveFunction");
            out.put("initial_window_height", initialGeometry[0]);
            out.put("final_window_height", finalGeometry[0]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_window_height.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  WindowHeight: %.6f -> %.6f (norm %.4f -> %.4f)%n",
                initialGeometry[0], finalGeometry[0], initialNorm, finalNorm);
        }

        // --- Beta calibration (1D Brent) ---
        {
            System.out.println("--- Beta Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FminEvaluator evaluator = new FminEvaluator(calculator, tuner);

            BetaObjectiveFunction objective =
                new BetaObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            Constraints constraints = loadConstraints(
                "WhistleStudyModel/BetaObjectiveFunction/BetaDefaultConstraints.xml");
            objective.setConstraintsBounds(constraints);

            double initialNorm = objective.value(initialGeometry);

            double[] lowerBounds = objective.getLowerBounds();
            double[] upperBounds = objective.getUpperBounds();

            BrentOptimizer optimizer = new BrentOptimizer(1e-4, 1e-4);
            UnivariatePointValuePair result = optimizer.optimize(
                new MaxEval(100),
                new UnivariateObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new SearchInterval(lowerBounds[0], upperBounds[0], initialGeometry[0]));

            objective.setGeometryPoint(new double[]{result.getPoint()});
            double finalNorm = result.getValue();
            double[] finalGeometry = objective.getGeometryPoint();

            ObjectNode out = mapper.createObjectNode();
            out.put("optimizer", "BetaObjectiveFunction");
            out.put("initial_beta", initialGeometry[0]);
            out.put("final_beta", finalGeometry[0]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_beta.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  Beta: %.6f -> %.6f (norm %.4f -> %.4f)%n",
                initialGeometry[0], finalGeometry[0], initialNorm, finalNorm);
        }

        // --- Joint WhistleCalibration (2D BOBYQA) ---
        {
            System.out.println("--- Joint Whistle Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new WhistleCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FminmaxEvaluator evaluator = new FminmaxEvaluator(calculator, tuner);

            WhistleCalibrationObjectiveFunction objective =
                new WhistleCalibrationObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // For joint calibration, build constraints with both WH and beta bounds
            Constraints whConstraints = loadConstraints(
                "WhistleStudyModel/WindowHeightObjectiveFunction/WindowHeightDefaultConstraints.xml");
            Constraints betaConstraints = loadConstraints(
                "WhistleStudyModel/BetaObjectiveFunction/BetaDefaultConstraints.xml");

            // Build merged constraints: [window_height, beta]
            Constraints mergedConstraints = new Constraints();
            mergedConstraints.setConstraintsName("Default");
            mergedConstraints.setObjectiveFunctionName("WhistleCalibrationObjectiveFunction");
            mergedConstraints.setObjectiveDisplayName("Whistle calibration");
            mergedConstraints.setNumberOfHoles(6);
            for (Constraint c : whConstraints.getConstraint()) {
                mergedConstraints.addConstraint(c);
            }
            for (Constraint c : betaConstraints.getConstraint()) {
                mergedConstraints.addConstraint(c);
            }
            objective.setConstraintsBounds(mergedConstraints);

            double initialNorm = objective.value(initialGeometry);

            // Run BOBYQA optimizer (2D)
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

            ObjectNode out = mapper.createObjectNode();
            out.put("optimizer", "WhistleCalibrationObjectiveFunction");
            out.put("initial_window_height", initialGeometry[0]);
            out.put("final_window_height", finalGeometry[0]);
            out.put("initial_beta", initialGeometry[1]);
            out.put("final_beta", finalGeometry[1]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_joint.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  Joint: WH %.6f->%.6f, Beta %.6f->%.6f (norm %.4f->%.4f)%n",
                initialGeometry[0], finalGeometry[0],
                initialGeometry[1], finalGeometry[1],
                initialNorm, finalNorm);
        }

        System.out.println("\nAll calibration fixtures written to " + outDir.getAbsolutePath());
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
