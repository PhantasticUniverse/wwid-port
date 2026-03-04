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

import org.apache.commons.math3.optim.*;
import org.apache.commons.math3.optim.nonlinear.scalar.GoalType;
import org.apache.commons.math3.optim.nonlinear.scalar.ObjectiveFunction;
import org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer;

import java.io.File;

/**
 * Generate calibration golden fixture for Reed study model.
 *
 * Exercises the {@code ReedCalibratorObjectiveFunction} — 2D BOBYQA
 * jointly optimizing alpha + beta using CentDeviationEvaluator.
 *
 * Uses {@code SimpleReedCalculator} + {@code SimpleInstrumentTuner} at 72°F.
 *
 * Output: golden/expected/RD-CAL/calib_joint.json
 */
public class ReedCalibDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/RD-CAL";

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/ReedStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/ReedStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        String instrFile = "SampleChanter.xml";
        String tuningFile = "A3-ClosedFingering.xml";

        System.out.println("=== Reed Calibration Driver ===");
        System.out.printf("Instrument: %s%nTuning: %s%n%n", instrFile, tuningFile);

        // --- Joint Reed Calibration (2D BOBYQA) ---
        {
            System.out.println("--- Joint Reed Calibration ---");
            Instrument instrument = loadInstrument(instrDir, instrFile);
            Tuning tuning = loadTuning(tuningDir, tuningFile);

            InstrumentCalculator calculator = new SimpleReedCalculator();
            calculator.setInstrument(instrument);
            calculator.setPhysicalParameters(params);

            SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
            tuner.setParams(params);
            CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

            ReedCalibratorObjectiveFunction objective =
                new ReedCalibratorObjectiveFunction(calculator, tuning, evaluator);

            double[] initialGeometry = objective.getGeometryPoint();

            // Build constraints: [alpha, beta] both [0.0, 10.0]
            double[] lowerBound = new double[] { 0.00, 0.00 };
            double[] upperBound = new double[] { 10.0, 10.0 };
            objective.setLowerBounds(lowerBound);
            objective.setUpperBounds(upperBound);

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
            out.put("optimizer", "ReedCalibratorObjectiveFunction");
            out.put("initial_alpha", initialGeometry[0]);
            out.put("final_alpha", finalGeometry[0]);
            out.put("initial_beta", initialGeometry[1]);
            out.put("final_beta", finalGeometry[1]);
            out.put("initial_norm", initialNorm);
            out.put("final_norm", finalNorm);

            File outFile = new File(outDir, "calib_joint.json");
            mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, out);
            System.out.printf("  Joint: Alpha %.6f->%.6f, Beta %.6f->%.6f (norm %.4f->%.4f)%n",
                initialGeometry[0], finalGeometry[0],
                initialGeometry[1], finalGeometry[1],
                initialNorm, finalNorm);
        }

        System.out.println("\nReed calibration fixture written to " + outDir.getAbsolutePath());
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
}
