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
 * Generate calibration golden fixtures for Flute study model.
 *
 * Exercises three calibration objective functions from the Flute study model:
 * <ol>
 *   <li>{@code AirstreamLengthObjectiveFunction} — 1D Brent minimizer,
 *       adjusts {@code embouchureHole.airstreamLength} using FmaxEvaluator</li>
 *   <li>{@code BetaObjectiveFunction} — 1D Brent minimizer,
 *       adjusts mouthpiece beta using FminEvaluator</li>
 *   <li>{@code FluteCalibrationObjectiveFunction} — 2D BOBYQA,
 *       jointly optimizes airstream length + beta using FminmaxEvaluator</li>
 * </ol>
 *
 * All three use {@code FluteCalculator} (embouchure hole mouthpiece) with
 * {@code LinearVInstrumentTuner(BLOWING_LEVEL=5)} at 72°F.
 *
 * Output: golden/expected/FL-CAL/
 *   - calib_airstream_length.json
 *   - calib_beta.json
 *   - calib_joint.json
 */
public class FluteCalibDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/FL-CAL";
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

        System.out.println("=== Flute Calibration Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // --- AirstreamLength calibration (1D Brent) ---
        {
            System.out.println("--- AirstreamLength Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FmaxEvaluator evaluator = new FmaxEvaluator(calculator, tuner);

            AirstreamLengthObjectiveFunction objective =
                new AirstreamLengthObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            Constraints constraints = loadConstraints(
                "FluteStudyModel/AirstreamLengthObjectiveFunction/AirstreamLengthDefaultConstraints.xml");
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
            out.put("optimizer", "AirstreamLengthObjectiveFunction");
            out.put("initial_airstream_length", initialGeometry[0]);
            out.put("final_airstream_length", finalGeometry[0]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_airstream_length.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  AirstreamLength: %.6f -> %.6f (norm %.4f -> %.4f)%n",
                initialGeometry[0], finalGeometry[0], initialNorm, finalNorm);
        }

        // --- Beta calibration (1D Brent) ---
        {
            System.out.println("--- Beta Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FminEvaluator evaluator = new FminEvaluator(calculator, tuner);

            BetaObjectiveFunction objective =
                new BetaObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            Constraints constraints = loadConstraints(
                "FluteStudyModel/BetaObjectiveFunction/BetaDefaultConstraints.xml");
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

        // --- Joint FluteCalibration (2D BOBYQA) ---
        {
            System.out.println("--- Joint Flute Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new FluteCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
            tuner.setParams(params);
            FminmaxEvaluator evaluator = new FminmaxEvaluator(calculator, tuner);

            FluteCalibrationObjectiveFunction objective =
                new FluteCalibrationObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // Build merged constraints: [airstream_length, beta]
            Constraints alConstraints = loadConstraints(
                "FluteStudyModel/AirstreamLengthObjectiveFunction/AirstreamLengthDefaultConstraints.xml");
            Constraints betaConstraints = loadConstraints(
                "FluteStudyModel/BetaObjectiveFunction/BetaDefaultConstraints.xml");

            Constraints mergedConstraints = new Constraints();
            mergedConstraints.setConstraintsName("Default");
            mergedConstraints.setObjectiveFunctionName("FluteCalibrationObjectiveFunction");
            mergedConstraints.setObjectiveDisplayName("Flute calibration");
            mergedConstraints.setNumberOfHoles(6);
            for (Constraint c : alConstraints.getConstraint()) {
                mergedConstraints.addConstraint(c);
            }
            for (Constraint c : betaConstraints.getConstraint()) {
                mergedConstraints.addConstraint(c);
            }
            objective.setConstraintsBounds(mergedConstraints);

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

            ObjectNode out = mapper.createObjectNode();
            out.put("optimizer", "FluteCalibrationObjectiveFunction");
            out.put("initial_airstream_length", initialGeometry[0]);
            out.put("final_airstream_length", finalGeometry[0]);
            out.put("initial_beta", initialGeometry[1]);
            out.put("final_beta", finalGeometry[1]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_joint.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  Joint: AL %.6f->%.6f, Beta %.6f->%.6f (norm %.4f->%.4f)%n",
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
