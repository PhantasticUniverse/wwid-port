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
import org.apache.commons.math3.optim.univariate.*;

import java.io.File;
import java.util.Arrays;

/**
 * Generate optimization golden fixtures for bore optimizers.
 *
 * Scenarios:
 *   WH-BORE-01: Whistle + BoreDiameterFromBottom + SamplePVC-Whistle (1D Brent)
 *   WH-BORE-02: Whistle + BoreDiameterFromTop  + SamplePVC-Whistle (2D BOBYQA)
 *   RD-BORE-01: Reed   + BorePosition           + Didgeridoo-2stage (3D BOBYQA)
 *
 * Output: golden/expected/WH-BORE-01/, WH-BORE-02/, RD-BORE-01/
 *
 * Key wiring details (matching Java study models exactly):
 *   - Whistle: WhistleCalculator + LinearVInstrumentTuner(blowingLevel=5)
 *              + CentDeviationEvaluator
 *   - Reed:    SimpleReedCalculator + SimpleInstrumentTuner
 *              + CentDeviationEvaluator
 *   - BoreDiameterFromBottom auto-detects n_unchanged via getTopOfBody()+1
 *   - BoreDiameterFromTop auto-detects n_changed via getLowestPoint("Head")
 *   - BorePosition auto-detects via getTopOfBody()+1, bottomPointUnchanged=false
 */
public class BoreOptDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_BASE = "../golden/expected";
    private static final int BLOWING_LEVEL = 5;

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        System.out.println("=== Bore Optimization Driver ===\n");

        runWhistleBoreDiaFromBottom(mapper, params);
        runWhistleBoreDiaFromTop(mapper, params);
        runReedBorePosition(mapper, params);

        System.out.println("\nAll bore optimization fixtures generated.");
    }

    // ── WH-BORE-01: BoreDiameterFromBottom (Whistle, 1D Brent) ──────────

    private static void runWhistleBoreDiaFromBottom(ObjectMapper mapper,
            PhysicalParameters params) throws Exception {
        System.out.println("--- WH-BORE-01: BoreDiameterFromBottom (Whistle) ---");

        File outDir = new File(OUTPUT_BASE + "/WH-BORE-01");
        outDir.mkdirs();

        Instrument instrument = loadInstrument(
                ORACLE_BASE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml");
        Tuning tuning = loadTuning(
                ORACLE_BASE + "/WhistleStudy/tunings/SamplePVC-tuning.xml");

        WhistleCalculator calculator = new WhistleCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params);

        LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
        tuner.setParams(params);
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        // Auto-detect boundary: getTopOfBody()+1 → n_unchanged=2, n_dims=1
        BoreDiameterFromBottomObjectiveFunction objective =
                new BoreDiameterFromBottomObjectiveFunction(calculator, tuning, evaluator);

        int nrDimensions = objective.getNrDimensions();
        System.out.printf("  nrDimensions: %d (bore points: %d)%n",
                nrDimensions, instrument.getBorePoint().size());

        // Default bounds from WhistleStudyModel: taper inward toward bottom
        double[] lowerBound = new double[nrDimensions];
        double[] upperBound = new double[nrDimensions];
        Arrays.fill(lowerBound, 0.5);
        Arrays.fill(upperBound, 1.0);
        objective.setLowerBounds(lowerBound);
        objective.setUpperBounds(upperBound);

        double[] initialGeometry = objective.getGeometryPoint();
        double[] initialPoint = objective.getInitialPoint();
        double initialNorm = objective.value(initialPoint);

        runOptimize(objective, initialGeometry, initialNorm, initialPoint,
                "BoreDiameterFromBottomObjectiveFunction", mapper, outDir);
    }

    // ── WH-BORE-02: BoreDiameterFromTop (Whistle, 2D BOBYQA) ────────────

    private static void runWhistleBoreDiaFromTop(ObjectMapper mapper,
            PhysicalParameters params) throws Exception {
        System.out.println("--- WH-BORE-02: BoreDiameterFromTop (Whistle) ---");

        File outDir = new File(OUTPUT_BASE + "/WH-BORE-02");
        outDir.mkdirs();

        Instrument instrument = loadInstrument(
                ORACLE_BASE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml");
        Tuning tuning = loadTuning(
                ORACLE_BASE + "/WhistleStudy/tunings/SamplePVC-tuning.xml");

        WhistleCalculator calculator = new WhistleCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params);

        LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
        tuner.setParams(params);
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        // Auto-detect: getLowestPoint("Head") → n_changed via heuristic
        BoreDiameterFromTopObjectiveFunction objective =
                new BoreDiameterFromTopObjectiveFunction(calculator, tuning, evaluator);

        int nrDimensions = objective.getNrDimensions();
        System.out.printf("  nrDimensions: %d (bore points: %d)%n",
                nrDimensions, instrument.getBorePoint().size());

        // Default bounds from WhistleStudyModel: taper inward toward top
        double[] lowerBound = new double[nrDimensions];
        double[] upperBound = new double[nrDimensions];
        Arrays.fill(lowerBound, 0.5);
        Arrays.fill(upperBound, 1.0);
        lowerBound[0] = 0.999;  // First ratio constrained near 1.0
        objective.setLowerBounds(lowerBound);
        objective.setUpperBounds(upperBound);

        double[] initialGeometry = objective.getGeometryPoint();
        double[] initialPoint = objective.getInitialPoint();
        double initialNorm = objective.value(initialPoint);

        runOptimize(objective, initialGeometry, initialNorm, initialPoint,
                "BoreDiameterFromTopObjectiveFunction", mapper, outDir);
    }

    // ── RD-BORE-01: BorePosition (Reed/Didgeridoo, 3D BOBYQA) ───────────

    private static void runReedBorePosition(ObjectMapper mapper,
            PhysicalParameters params) throws Exception {
        System.out.println("--- RD-BORE-01: BorePosition (Reed/Didgeridoo) ---");

        File outDir = new File(OUTPUT_BASE + "/RD-BORE-01");
        outDir.mkdirs();

        Instrument instrument = loadInstrument(
                ORACLE_BASE + "/ReedStudy/instruments/Didgeridoo-2stage-D2-D3.xml");
        Tuning tuning = loadTuning(
                ORACLE_BASE + "/ReedStudy/tunings/Didgeridoo-D2-D3-tuning.xml");

        // Reed study model uses SimpleReedCalculator + SimpleInstrumentTuner
        SimpleReedCalculator calculator = new SimpleReedCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params);

        SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
        tuner.setParams(params);
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        // Auto-detect: getTopOfBody()+1, bottomPointUnchanged=false
        BorePositionObjectiveFunction objective =
                new BorePositionObjectiveFunction(calculator, tuning, evaluator);

        int nrDimensions = objective.getNrDimensions();
        System.out.printf("  nrDimensions: %d (bore points: %d)%n",
                nrDimensions, instrument.getBorePoint().size());

        // Load oracle constraints from DidgeridooConstraints-2stage.xml
        Constraints constraints = loadConstraints(
                "ReedStudyModel/BorePositionObjectiveFunction/DidgeridooConstraints-2stage.xml");
        objective.setConstraintsBounds(constraints);

        double[] initialGeometry = objective.getGeometryPoint();
        double[] initialPoint = objective.getInitialPoint();
        double initialNorm = objective.value(initialPoint);

        runOptimize(objective, initialGeometry, initialNorm, initialPoint,
                "BorePositionObjectiveFunction", mapper, outDir);
    }

    // ── Shared optimizer runner (auto-selects Brent or BOBYQA) ───────────

    private static void runOptimize(BaseObjectiveFunction objective,
            double[] initialGeometry, double initialNorm, double[] initialPoint,
            String optimizerName, ObjectMapper mapper, File outDir) throws Exception {
        int nrDimensions = objective.getNrDimensions();
        int maxEval = objective.getMaxEvaluations();
        double[] lowerBounds = objective.getLowerBounds();
        double[] upperBounds = objective.getUpperBounds();

        double finalNorm;
        double[] finalGeometry;
        int evaluations;
        String solverType;

        if (nrDimensions == 1) {
            // 1D → Brent optimizer
            BrentOptimizer optimizer = new BrentOptimizer(1e-4, 1e-4);
            UnivariatePointValuePair result = optimizer.optimize(
                    new MaxEval(maxEval),
                    new UnivariateObjectiveFunction(objective),
                    GoalType.MINIMIZE,
                    new SearchInterval(lowerBounds[0], upperBounds[0], initialPoint[0]));

            objective.setGeometryPoint(new double[]{result.getPoint()});
            finalNorm = result.getValue();
            finalGeometry = objective.getGeometryPoint();
            evaluations = optimizer.getEvaluations();
            solverType = "Brent";
        } else {
            // Multi-dim → BOBYQA
            double initTrust = objective.getInitialTrustRegionRadius(initialPoint);
            double stopTrust = objective.getStoppingTrustRegionRadius();

            BOBYQAOptimizer optimizer = new BOBYQAOptimizer(
                    2 * nrDimensions + 1, initTrust, stopTrust);

            PointValuePair result = optimizer.optimize(
                    new MaxEval(maxEval),
                    new ObjectiveFunction(objective),
                    GoalType.MINIMIZE,
                    new InitialGuess(initialPoint),
                    new SimpleBounds(lowerBounds, upperBounds));

            objective.setGeometryPoint(result.getPoint());
            finalNorm = result.getValue();
            finalGeometry = objective.getGeometryPoint();
            evaluations = optimizer.getEvaluations();
            solverType = "BOBYQA";
        }

        ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations, initialGeometry, finalGeometry);
        out.put("optimizer", optimizerName);
        out.put("solverType", solverType);
        out.put("nrDimensions", nrDimensions);

        mapper.writerWithDefaultPrettyPrinter().writeValue(
                new File(outDir, "optimize_0.json"), out);
        System.out.printf("  norm: %.6f -> %.6f (%d dims, %d evals, %s)%n",
                initialNorm, finalNorm, nrDimensions, evaluations, solverType);
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    private static Instrument loadInstrument(String path) throws Exception {
        BindFactory factory = GeometryBindFactory.getInstance();
        Instrument instrument = (Instrument) factory.unmarshalXml(
                new File(path).getCanonicalFile(), true);
        instrument.updateComponents();
        return instrument;
    }

    private static Tuning loadTuning(String path) throws Exception {
        BindFactory factory = NoteBindFactory.getInstance();
        return (Tuning) factory.unmarshalXml(
                new File(path).getCanonicalFile(), true);
    }

    private static Constraints loadConstraints(String relativePath) throws Exception {
        String fullPath = ORACLE_BASE + "/constraints/" + relativePath;
        BindFactory factory =
                com.wwidesigner.optimization.bind.OptimizationBindFactory.getInstance();
        return (Constraints) factory.unmarshalXml(
                new File(fullPath).getCanonicalFile(), true);
    }
}
