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
import org.apache.commons.math3.optim.univariate.*;

import java.io.File;
import java.util.Arrays;

/**
 * Generic optimization golden fixture driver.
 *
 * Handles all study models (NAF, Whistle, Flute, Reed) and all
 * objective function types. Uses scenario JSON files with:
 *   - studyKind, instrument, tuning, constraints (optional)
 *   - optimizer key, constraintsPath (oracle path), boreLengthAdjustment
 *
 * Dispatches to the correct calculator/tuner per study model,
 * creates the objective function, and runs Brent (1D) or BOBYQA (N-D).
 */
public class GenericOptDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_BASE = "../golden/expected";
    private static final int BLOWING_LEVEL = 5;

    // ── Scenario definition (inline, not JSON) ──────────────────
    static class Spec {
        String id;
        String studyKind;
        String instrumentPath;
        String tuningPath;
        String constraintsPath;   // Oracle path relative to constraints/
        String optimizer;
        // Optional bounds override for bore optimizers
        double[] lowerBoundsOverride;
        double[] upperBoundsOverride;

        Spec(String id, String studyKind, String instrumentPath, String tuningPath,
             String constraintsPath, String optimizer) {
            this.id = id;
            this.studyKind = studyKind;
            this.instrumentPath = instrumentPath;
            this.tuningPath = tuningPath;
            this.constraintsPath = constraintsPath;
            this.optimizer = optimizer;
        }

        Spec withBounds(double[] lower, double[] upper) {
            this.lowerBoundsOverride = lower;
            this.upperBoundsOverride = upper;
            return this;
        }
    }

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();
        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        System.out.println("=== Generic Optimization Driver ===\n");

        // Whistle instrument/tuning paths
        String whInstr = ORACLE_BASE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml";
        String whTuning = ORACLE_BASE + "/WhistleStudy/tunings/SamplePVC-tuning.xml";
        String whHoleConstraints = "WhistleStudyModel/HoleObjectiveFunction/DefaultHoleConstraints.xml";

        // Flute instrument/tuning paths
        String flInstr = ORACLE_BASE + "/FluteStudy/instruments/fife.xml";
        String flTuning = ORACLE_BASE + "/FluteStudy/tunings/fife-tuning.xml";
        String flHoleConstraints = "FluteStudyModel/HoleObjectiveFunction/LargeHoleSize_Spacing_6holes.xml";

        // Reed instrument/tuning paths
        String rdInstr = ORACLE_BASE + "/ReedStudy/instruments/SampleChanter.xml";
        String rdTuning = ORACLE_BASE + "/ReedStudy/tunings/D4-uilleann-ET-tuning.xml";
        String rdHoleConstraints = "ReedStudyModel/HoleObjectiveFunction/SampleChanterHoleConstraints.xml";

        Spec[] specs = {
            // ── Whistle standalone optimizers ──
            new Spec("WH-TAPER-01", "Whistle", whInstr, whTuning,
                whHoleConstraints, "BasicTaperObjectiveFunction"),
            new Spec("WH-BORE-SPACING-01", "Whistle", whInstr, whTuning,
                "WhistleStudyModel/BoreSpacingFromTopObjectiveFunction/SteppedCylinderSpacing.xml",
                "BoreSpacingFromTopObjectiveFunction"),

            // ── Whistle merged optimizers ──
            new Spec("WH-MERGED-01", "Whistle", whInstr, whTuning,
                whHoleConstraints, "HoleAndBoreDiameterFromTopObjectiveFunction"),
            new Spec("WH-MERGED-02", "Whistle", whInstr, whTuning,
                whHoleConstraints, "HoleAndBoreDiameterFromBottomObjectiveFunction"),
            new Spec("WH-MERGED-03", "Whistle", whInstr, whTuning,
                whHoleConstraints, "HoleAndBoreSpacingFromTopObjectiveFunction"),
            new Spec("WH-MERGED-04", "Whistle", whInstr, whTuning,
                whHoleConstraints, "HoleAndBasicTaperObjectiveFunction"),
            new Spec("WH-MERGED-05", "Whistle", whInstr, whTuning,
                whHoleConstraints, "HoleAndHeadjointObjectiveFunction"),

            // ── Flute standalone optimizers ──
            new Spec("FL-STOPPER-01", "Flute", flInstr, flTuning,
                null, "StopperPositionObjectiveFunction"),
            new Spec("FL-HEADJOINT-01", "Flute", flInstr, flTuning,
                null, "HeadjointObjectiveFunction"),
            new Spec("FL-TAPER-01", "Flute", flInstr, flTuning,
                flHoleConstraints, "BasicTaperObjectiveFunction"),

            // ── Flute merged optimizers ──
            new Spec("FL-MERGED-01", "Flute", flInstr, flTuning,
                flHoleConstraints, "HoleAndBoreDiameterFromBottomObjectiveFunction"),
            new Spec("FL-MERGED-02", "Flute", flInstr, flTuning,
                flHoleConstraints, "HoleAndBoreSpacingFromTopObjectiveFunction"),
            new Spec("FL-MERGED-03", "Flute", flInstr, flTuning,
                flHoleConstraints, "HoleAndBasicTaperObjectiveFunction"),
            new Spec("FL-MERGED-04", "Flute", flInstr, flTuning,
                flHoleConstraints, "HoleAndHeadjointObjectiveFunction"),

            // ── Reed standalone optimizers ──
            new Spec("RD-OPT-01", "Reed", rdInstr, rdTuning,
                rdHoleConstraints, "HoleObjectiveFunction"),
            new Spec("RD-BORE-02", "Reed", rdInstr, rdTuning,
                null, "BoreDiameterFromBottomObjectiveFunction"),
            new Spec("RD-BORE-03", "Reed", rdInstr, rdTuning,
                null, "BoreFromBottomObjectiveFunction"),

            // ── Reed merged optimizers ──
            new Spec("RD-MERGED-01", "Reed", rdInstr, rdTuning,
                rdHoleConstraints, "HoleAndBoreDiameterFromBottomObjectiveFunction"),
            new Spec("RD-MERGED-02", "Reed", rdInstr, rdTuning,
                rdHoleConstraints, "HoleAndBorePositionObjectiveFunction"),
            new Spec("RD-MERGED-03", "Reed", rdInstr, rdTuning,
                rdHoleConstraints, "HoleAndBoreFromBottomObjectiveFunction"),
        };

        // Allow filtering by scenario ID via command-line args
        for (Spec spec : specs) {
            if (args.length > 0 && !spec.id.equals(args[0]) && !args[0].equals("all")) {
                continue;
            }
            try {
                runScenario(spec, mapper, params);
            } catch (Exception e) {
                System.err.println("  FAILED: " + spec.id + ": " + e.getMessage());
                e.printStackTrace();
            }
        }

        System.out.println("\nDone.");
    }

    private static void runScenario(Spec spec, ObjectMapper mapper,
            PhysicalParameters params) throws Exception {
        System.out.println("--- " + spec.id + ": " + spec.optimizer + " (" + spec.studyKind + ") ---");

        File outDir = new File(OUTPUT_BASE + "/" + spec.id);
        outDir.mkdirs();

        Instrument instrument = loadInstrument(spec.instrumentPath);
        Tuning tuning = loadTuning(spec.tuningPath);

        // Create calculator + tuner per study model
        InstrumentCalculator calculator;
        EvaluatorInterface evaluator;
        switch (spec.studyKind) {
            case "Whistle": {
                calculator = new WhistleCalculator();
                calculator.setInstrument(instrument);
                calculator.setPhysicalParameters(params);
                LinearVInstrumentTuner wTuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
                wTuner.setParams(params);
                evaluator = new CentDeviationEvaluator(calculator, wTuner);
                break;
            }
            case "Flute": {
                calculator = new FluteCalculator();
                calculator.setInstrument(instrument);
                calculator.setPhysicalParameters(params);
                LinearVInstrumentTuner fTuner = new LinearVInstrumentTuner(BLOWING_LEVEL);
                fTuner.setParams(params);
                evaluator = new CentDeviationEvaluator(calculator, fTuner);
                break;
            }
            case "Reed": {
                calculator = new SimpleReedCalculator();
                calculator.setInstrument(instrument);
                calculator.setPhysicalParameters(params);
                SimpleInstrumentTuner rTuner = new SimpleInstrumentTuner();
                rTuner.setParams(params);
                evaluator = new CentDeviationEvaluator(calculator, rTuner);
                break;
            }
            default: {
                calculator = new NAFCalculator();
                calculator.setInstrument(instrument);
                calculator.setPhysicalParameters(params);
                SimpleInstrumentTuner nTuner = new SimpleInstrumentTuner();
                nTuner.setParams(params);
                evaluator = new CentDeviationEvaluator(calculator, nTuner);
                break;
            }
        }

        // Create objective function
        BaseObjectiveFunction objective = createObjectiveFunction(
                spec.optimizer, calculator, tuning, evaluator, instrument);

        // Set constraints bounds
        if (spec.constraintsPath != null) {
            Constraints constraints = loadConstraints(spec.constraintsPath);
            int constraintDims = constraints.getConstraint().size();
            int objectiveDims = objective.getNrDimensions();
            if (constraintDims == objectiveDims) {
                objective.setConstraintsBounds(constraints);
            } else if (constraintDims < objectiveDims) {
                // Merged optimizer: hole bounds from constraints + bore defaults
                double[] holeLower = constraints.getLowerBounds();
                double[] holeUpper = constraints.getUpperBounds();
                double[] lower = new double[objectiveDims];
                double[] upper = new double[objectiveDims];
                System.arraycopy(holeLower, 0, lower, 0, constraintDims);
                System.arraycopy(holeUpper, 0, upper, 0, constraintDims);
                fillBoreDefaults(spec.optimizer, lower, upper, constraintDims, objectiveDims);
                objective.setLowerBounds(lower);
                objective.setUpperBounds(upper);
                System.out.printf("  Merged bounds: %d hole dims + %d bore dims%n",
                        constraintDims, objectiveDims - constraintDims);
            }
        } else if (spec.lowerBoundsOverride != null) {
            objective.setLowerBounds(spec.lowerBoundsOverride);
            objective.setUpperBounds(spec.upperBoundsOverride);
        }

        // Fallback for standalone optimizers still without bounds
        if (objective.getLowerBounds() == null) {
            ensureDefaultBounds(spec, objective, instrument);
        }

        double[] initialGeometry = objective.getGeometryPoint();
        double[] initialPoint = objective.getInitialPoint();
        double initialNorm = objective.value(initialPoint);

        // Run optimizer
        int nrDimensions = objective.getNrDimensions();
        int maxEval = Math.max(objective.getMaxEvaluations(), 50000);
        double[] lowerBounds = objective.getLowerBounds();
        double[] upperBounds = objective.getUpperBounds();

        double finalNorm;
        double[] finalGeometry;
        int evaluations;
        String solverType;

        if (nrDimensions == 1) {
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

        // Also run eval_before and eval_after
        // Reload for eval_before
        Instrument instrBefore = loadInstrument(spec.instrumentPath);
        calculator.setInstrument(instrBefore);

        ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations, initialGeometry, finalGeometry);
        out.put("optimizer", spec.optimizer);
        out.put("solverType", solverType);
        out.put("nrDimensions", nrDimensions);

        mapper.writerWithDefaultPrettyPrinter().writeValue(
                new File(outDir, "optimize_0.json"), out);

        System.out.printf("  norm: %.6f -> %.6f (%d dims, %d evals, %s)%n",
                initialNorm, finalNorm, nrDimensions, evaluations, solverType);
    }

    /** Fill bore default bounds for extra dims in merged optimizers. */
    private static void fillBoreDefaults(String optimizer, double[] lower, double[] upper,
            int from, int to) {
        switch (optimizer) {
            case "HoleAndBasicTaperObjectiveFunction":
                // BasicTaper: head length ratio + foot diameter ratio
                lower[from] = 0.01; upper[from] = 0.99;
                if (from + 1 < to) {
                    lower[from + 1] = 0.5; upper[from + 1] = 2.0;
                }
                break;
            case "HoleAndHeadjointObjectiveFunction":
                // Stopper position + bore diameter ratios
                lower[from] = 0.0; upper[from] = 0.1;
                for (int i = from + 1; i < to; i++) {
                    lower[i] = 0.5; upper[i] = 2.0;
                }
                break;
            default:
                // Generic bore dims: ratios [0.5, 2.0]
                for (int i = from; i < to; i++) {
                    lower[i] = 0.5; upper[i] = 2.0;
                }
                break;
        }
    }

    /** Ensure sensible default bounds for standalone optimizers without constraints. */
    private static void ensureDefaultBounds(Spec spec, BaseObjectiveFunction objective,
            Instrument instrument) {
        String opt = spec.optimizer;
        int dims = objective.getNrDimensions();

        // Basic taper: 2 dims — head length ratio [0.01, 0.99], foot dia ratio [0.5, 2.0]
        if (opt.equals("BasicTaperObjectiveFunction")) {
            objective.setLowerBounds(new double[]{0.01, 0.5});
            objective.setUpperBounds(new double[]{0.99, 2.0});
            return;
        }

        // Bore diameter ratio optimizers: [0.5, 2.0] per dimension
        if (opt.contains("BoreDiameter")) {
            double[] lower = new double[dims];
            double[] upper = new double[dims];
            Arrays.fill(lower, 0.5);
            Arrays.fill(upper, 2.0);
            objective.setLowerBounds(lower);
            objective.setUpperBounds(upper);
            return;
        }

        // Stopper position: [0.0, 0.1]
        if (opt.equals("StopperPositionObjectiveFunction")) {
            objective.setLowerBounds(new double[]{0.0});
            objective.setUpperBounds(new double[]{0.1});
            return;
        }

        // Headjoint: stopper + bore dia from top
        if (opt.equals("HeadjointObjectiveFunction")) {
            double[] lower = new double[dims];
            double[] upper = new double[dims];
            lower[0] = 0.0; upper[0] = 0.1;
            for (int i = 1; i < dims; i++) {
                lower[i] = 0.5; upper[i] = 2.0;
            }
            objective.setLowerBounds(lower);
            objective.setUpperBounds(upper);
            return;
        }

        // BoreFromBottom: positions + diameters as ratios
        if (opt.equals("BoreFromBottomObjectiveFunction")) {
            double[] lower = new double[dims];
            double[] upper = new double[dims];
            Arrays.fill(lower, 0.5);
            Arrays.fill(upper, 2.0);
            objective.setLowerBounds(lower);
            objective.setUpperBounds(upper);
            return;
        }

        // BoreSpacingFromTop: spacing ratios
        if (opt.equals("BoreSpacingFromTopObjectiveFunction")) {
            double[] lower = new double[dims];
            double[] upper = new double[dims];
            Arrays.fill(lower, 0.5);
            Arrays.fill(upper, 2.0);
            objective.setLowerBounds(lower);
            objective.setUpperBounds(upper);
            return;
        }

        System.err.println("  WARNING: No default bounds for " + opt + " (" + dims + " dims)");
    }

    // ── Objective function factory ──────────────────────────────

    private static BaseObjectiveFunction createObjectiveFunction(
            String name, InstrumentCalculator calc, Tuning tun,
            EvaluatorInterface eval, Instrument instrument) throws Exception {
        switch (name) {
            // ── Hole optimizers (shared across models) ──
            case "HoleSizeObjectiveFunction":
                return new HoleSizeObjectiveFunction(calc, tun, eval);
            case "HolePositionObjectiveFunction":
                return new HolePositionObjectiveFunction(calc, tun, eval,
                        BoreLengthAdjustmentType.PRESERVE_TAPER);
            case "HoleObjectiveFunction":
                return new HoleObjectiveFunction(calc, tun, eval);

            // ── Bore optimizers ──
            case "BasicTaperObjectiveFunction":
                return new BasicTaperObjectiveFunction(calc, tun, eval);
            case "BoreDiameterFromTopObjectiveFunction":
                return new BoreDiameterFromTopObjectiveFunction(calc, tun, eval);
            case "BoreDiameterFromBottomObjectiveFunction":
                return new BoreDiameterFromBottomObjectiveFunction(calc, tun, eval);
            case "BoreSpacingFromTopObjectiveFunction":
                return new BoreSpacingFromTopObjectiveFunction(calc, tun, eval);
            case "BorePositionObjectiveFunction":
                return new BorePositionObjectiveFunction(calc, tun, eval);
            case "BoreFromBottomObjectiveFunction":
                return new BoreFromBottomObjectiveFunction(calc, tun, eval);
            case "StopperPositionObjectiveFunction":
                return new StopperPositionObjectiveFunction(calc, tun, eval, true);
            case "HeadjointObjectiveFunction":
                return new HeadjointObjectiveFunction(calc, tun, eval);

            // ── Merged hole+bore optimizers (all use auto-detect no-arg overloads) ──
            case "HoleAndBasicTaperObjectiveFunction":
                return new HoleAndTaperObjectiveFunction(calc, tun, eval);
            case "HoleAndBoreDiameterFromTopObjectiveFunction":
                return new HoleAndBoreDiameterFromTopObjectiveFunction(calc, tun, eval);
            case "HoleAndBoreDiameterFromBottomObjectiveFunction":
                return new HoleAndBoreDiameterFromBottomObjectiveFunction(calc, tun, eval);
            case "HoleAndBoreSpacingFromTopObjectiveFunction":
                return new HoleAndBoreSpacingFromTopObjectiveFunction(calc, tun, eval);
            case "HoleAndBorePositionObjectiveFunction":
                return new HoleAndBorePositionObjectiveFunction(calc, tun, eval);
            case "HoleAndBoreFromBottomObjectiveFunction":
                return new HoleAndBoreFromBottomObjectiveFunction(calc, tun, eval);
            case "HoleAndHeadjointObjectiveFunction":
                return new HoleAndHeadjointObjectiveFunction(calc, tun, eval);

            default:
                throw new IllegalArgumentException("Unknown objective function: " + name);
        }
    }

    // ── Helpers ──────────────────────────────────────────────────

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
