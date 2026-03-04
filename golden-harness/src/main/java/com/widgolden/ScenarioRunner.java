package com.widgolden;

import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.geometry.calculation.DefaultHoleCalculator;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Fingering;
import com.wwidesigner.note.Note;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.optimization.*;
import com.wwidesigner.optimization.bind.OptimizationBindFactory;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;
import org.apache.commons.math3.complex.Complex;
import org.apache.commons.math3.optim.*;
import org.apache.commons.math3.optim.nonlinear.scalar.GoalType;
import org.apache.commons.math3.optim.nonlinear.scalar.ObjectiveFunction;
import org.apache.commons.math3.optim.univariate.*;

import java.io.File;
import java.util.ArrayList;
import java.util.List;

/// Core orchestrator: loads instruments/tunings via upstream JAXB factories,
/// dispatches each action, and writes JSON outputs to the expected directory.
///
/// Wires calculator/tuner/evaluator exactly as NafStudyModel does:
/// - PhysicalParameters(72.0, TemperatureType.F)
/// - NAFCalculator (DefaultFippleMouthpiece, ThickFlangedOpenEnd, DefaultHole(0.9605))
/// - SimpleInstrumentTuner
/// - CentDeviationEvaluator
public class ScenarioRunner {

    private final File scenariosDir;
    private final File expectedDir;
    private final Scenario scenario;

    // Mutable state: instrument carries forward between actions
    private Instrument instrument;
    private Tuning tuning;
    private Constraints constraints;
    private PhysicalParameters params;
    private InstrumentCalculator calculator;
    private String instrumentPath; // for RELOAD_INSTRUMENT

    // Action output counters (eval_0, eval_1, etc.)
    private int evalCount = 0;
    private int zsampleCount = 0;
    private int calibrateCount = 0;
    private int optimizeCount = 0;
    private int constraintsCount = 0;
    private int internalsCount = 0;

    public ScenarioRunner(Scenario scenario, File scenariosDir, File expectedDir) {
        this.scenario = scenario;
        this.scenariosDir = scenariosDir;
        this.expectedDir = expectedDir;
    }

    public void run() throws Exception {
        File outputDir = new File(expectedDir, scenario.id);
        outputDir.mkdirs();

        System.out.println("Running scenario: " + scenario.id);

        loadInputs();

        for (Scenario.Action action : scenario.actions) {
            System.out.println("  Action: " + action.type);
            switch (action.type) {
                case "EVAL_TUNING":
                    runEvalTuning(outputDir);
                    break;
                case "ZSAMPLE":
                    runZSample(action, outputDir);
                    break;
                case "CALIBRATE":
                    runCalibrate(action, outputDir);
                    break;
                case "OPTIMIZE":
                    runOptimize(action, outputDir);
                    break;
                case "CREATE_DEFAULT_CONSTRAINTS":
                    runCreateConstraints(action, outputDir, BaseObjectiveFunction.DEFAULT_CONSTRAINTS_INTENT);
                    break;
                case "CREATE_BLANK_CONSTRAINTS":
                    runCreateConstraints(action, outputDir, BaseObjectiveFunction.BLANK_CONSTRAINTS_INTENT);
                    break;
                case "SET_FIPPLE_FACTOR":
                    InstrumentOverrides.setFippleFactor(instrument, action.value);
                    instrument.updateComponents();
                    calculator.setInstrument(instrument);
                    break;
                case "SET_WINDWAY_HEIGHT":
                    InstrumentOverrides.setWindwayHeight(instrument, action.value);
                    instrument.updateComponents();
                    calculator.setInstrument(instrument);
                    break;
                case "RELOAD_INSTRUMENT":
                    instrument = loadInstrument(instrumentPath);
                    calculator.setInstrument(instrument);
                    break;
                case "DUMP_INTERNALS":
                    runDumpInternals(outputDir);
                    break;
                default:
                    throw new IllegalArgumentException("Unknown action type: " + action.type);
            }
        }

        System.out.println("  Done. Outputs in: " + outputDir.getAbsolutePath());
    }

    private void loadInputs() throws Exception {
        // Load instrument
        instrumentPath = scenario.inputs.instrument;
        instrument = loadInstrument(instrumentPath);

        // Load tuning
        String tuningPath = scenario.inputs.tuning;
        File tuningFile = resolvePath(tuningPath);
        BindFactory noteFactory = NoteBindFactory.getInstance();
        tuning = (Tuning) noteFactory.unmarshalXml(tuningFile, true);

        // Load constraints (optional)
        if (scenario.inputs.constraints != null) {
            File constraintsFile = resolvePath(scenario.inputs.constraints);
            BindFactory optFactory = OptimizationBindFactory.getInstance();
            constraints = (Constraints) optFactory.unmarshalXml(constraintsFile, true);
        }

        // Create physical parameters: 72 degrees F, standard atmosphere
        params = new PhysicalParameters(72.0, TemperatureType.F);

        // Create calculator wired as NafStudyModel does
        calculator = createNafCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params);
    }

    private Instrument loadInstrument(String path) throws Exception {
        File instrumentFile = resolvePath(path);
        BindFactory geoFactory = GeometryBindFactory.getInstance();
        Instrument inst = (Instrument) geoFactory.unmarshalXml(instrumentFile, true);
        inst.updateComponents();
        return inst;
    }

    private File resolvePath(String path) {
        File f = new File(path);
        if (f.isAbsolute()) return f;
        return new File(scenariosDir, path);
    }

    private InstrumentCalculator createNafCalculator() {
        return new NAFCalculator();
    }

    // ── EVAL_TUNING ──────────────────────────────────────────────────

    private void runEvalTuning(File outputDir) throws Exception {
        SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        List<Fingering> fingeringTargets = tuning.getFingering();
        // calculateErrorVector also sets the tuning on the tuner internally
        double[] errorVector = evaluator.calculateErrorVector(fingeringTargets);

        List<String> noteNames = new ArrayList<>();
        List<Double> targetFreqs = new ArrayList<>();
        List<Double> predictedFreqs = new ArrayList<>();

        for (int i = 0; i < fingeringTargets.size(); i++) {
            Fingering f = fingeringTargets.get(i);
            if (f.getNote() != null) {
                noteNames.add(f.getNote().getName() != null ? f.getNote().getName() : "");
                targetFreqs.add(f.getNote().getFrequency());
                // Tuner now has tuning set (from calculateErrorVector above)
                predictedFreqs.add(tuner.predictedFrequency(f));
            } else {
                noteNames.add("");
                targetFreqs.add(null);
                predictedFreqs.add(null);
            }
        }

        ArrayNode result = OutputFormatter.formatEvalResult(
                noteNames, targetFreqs, predictedFreqs, errorVector);

        File outFile = new File(outputDir, "eval_" + evalCount + ".json");
        OutputFormatter.writeJson(result, outFile);
        evalCount++;
    }

    // ── ZSAMPLE ──────────────────────────────────────────────────────

    private void runZSample(Scenario.Action action, File outputDir) throws Exception {
        // Build fingering for the sample
        Fingering fingering;
        if (Boolean.TRUE.equals(action.fingeringAllClosed)) {
            fingering = createAllClosedFingering();
        } else if (action.fingeringIndex != null) {
            fingering = tuning.getFingering().get(action.fingeringIndex);
        } else {
            fingering = tuning.getFingering().get(0);
        }

        List<Double> frequencies = action.frequencies;
        List<Complex> impedances = new ArrayList<>();

        for (double freq : frequencies) {
            Complex z = calculator.calcZ(freq, fingering);
            impedances.add(z);
        }

        ArrayNode result = OutputFormatter.formatZSample(frequencies, impedances);
        File outFile = new File(outputDir, "zsample_" + zsampleCount + ".json");
        OutputFormatter.writeJson(result, outFile);
        zsampleCount++;
    }

    private Fingering createAllClosedFingering() {
        Fingering f = new Fingering();
        int nHoles = instrument.getHole().size();
        List<Boolean> openHoles = new ArrayList<>();
        for (int i = 0; i < nHoles; i++) {
            openHoles.add(false);
        }
        f.setOpenHole(openHoles);
        f.setOpenEnd(true);
        // Set a note with a nominal frequency for the playing range search
        Note note = new Note();
        note.setName("ZSample");
        f.setNote(note);
        return f;
    }

    // ── CALIBRATE ────────────────────────────────────────────────────

    private void runCalibrate(Scenario.Action action, File outputDir) throws Exception {
        // Get initial fipple factor
        double initialFippleFactor = instrument.getMouthpiece().getFipple().getFippleFactor();

        // Create evaluator + tuner
        SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        // Create objective function
        BaseObjectiveFunction objective = createObjectiveFunction(
                action.objectiveFunction, calculator, tuning, evaluator);

        // Load constraints for this objective function from the oracle
        Constraints calConstraints = loadDefaultConstraints(action.objectiveFunction);
        if (calConstraints != null) {
            objective.setConstraintsBounds(calConstraints);
        }

        // Get initial norm
        double[] initialPoint = objective.getGeometryPoint();
        double initialNorm = objective.value(initialPoint);

        // Run Brent optimizer (1D calibration)
        double[] lowerBounds = objective.getLowerBounds();
        double[] upperBounds = objective.getUpperBounds();

        BrentOptimizer optimizer = new BrentOptimizer(1e-4, 1e-4);
        UnivariatePointValuePair result = optimizer.optimize(
                new MaxEval(100),
                new UnivariateObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new SearchInterval(lowerBounds[0], upperBounds[0], initialPoint[0]));

        // Apply the result
        objective.setGeometryPoint(new double[]{result.getPoint()});
        double finalNorm = result.getValue();
        double finalFippleFactor = instrument.getMouthpiece().getFipple().getFippleFactor();

        // Update the calculator with the mutated instrument
        calculator.setInstrument(instrument);

        ObjectNode out = OutputFormatter.formatCalibrationResult(
                initialFippleFactor, finalFippleFactor, initialNorm, finalNorm);

        File outFile = new File(outputDir, "calibrate_" + calibrateCount + ".json");
        OutputFormatter.writeJson(out, outFile);
        calibrateCount++;
    }

    // ── OPTIMIZE ─────────────────────────────────────────────────────

    private void runOptimize(Scenario.Action action, File outputDir) throws Exception {
        SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        BaseObjectiveFunction objective = createObjectiveFunction(
                action.objectiveFunction, calculator, tuning, evaluator);

        // Apply external constraints bounds
        if (constraints != null) {
            objective.setConstraintsBounds(constraints);
        }

        double[] initialPoint = objective.getInitialPoint();
        double[] initialGeometry = objective.getGeometryPoint().clone();
        double initialNorm = objective.value(initialPoint);

        int nrDims = objective.getNrDimensions();
        int maxEval = objective.getMaxEvaluations();
        double initTrust = objective.getInitialTrustRegionRadius(initialPoint);
        double stopTrust = objective.getStoppingTrustRegionRadius();

        org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer optimizer =
                new org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer(
                        2 * nrDims + 1, initTrust, stopTrust);

        PointValuePair result = optimizer.optimize(
                new MaxEval(maxEval),
                new ObjectiveFunction(objective),
                GoalType.MINIMIZE,
                new InitialGuess(initialPoint),
                new SimpleBounds(objective.getLowerBounds(), objective.getUpperBounds()));

        // Apply the result
        objective.setGeometryPoint(result.getPoint());
        double finalNorm = result.getValue();
        double[] finalGeometry = objective.getGeometryPoint();
        int evaluations = optimizer.getEvaluations();

        // Update the calculator with the mutated instrument
        calculator.setInstrument(instrument);

        ObjectNode out = OutputFormatter.formatOptimizationResult(
                initialNorm, finalNorm, evaluations, initialGeometry, finalGeometry);

        File outFile = new File(outputDir, "optimize_" + optimizeCount + ".json");
        OutputFormatter.writeJson(out, outFile);

        // Write the mutated instrument XML
        File xmlFile = new File(outputDir, "instrument_after_optimize_" + optimizeCount + ".xml");
        BindFactory geoFactory = GeometryBindFactory.getInstance();
        geoFactory.marshalToXml(instrument, xmlFile);

        optimizeCount++;
    }

    // ── CREATE CONSTRAINTS ───────────────────────────────────────────

    private void runCreateConstraints(Scenario.Action action, File outputDir, int intent)
            throws Exception {
        SimpleInstrumentTuner tuner = new SimpleInstrumentTuner();
        CentDeviationEvaluator evaluator = new CentDeviationEvaluator(calculator, tuner);

        BaseObjectiveFunction objective = createObjectiveFunction(
                action.objectiveFunction, calculator, tuning, evaluator);

        // Get the constraints from the objective function
        // The Constraints object extracts bounds from its constraint list
        Constraints objConstraints = objective.getConstraints();
        double[] lowerBounds = objConstraints.getLowerBounds();
        double[] upperBounds = objConstraints.getUpperBounds();

        ObjectNode out = OutputFormatter.formatConstraintsResult(lowerBounds, upperBounds);

        File outFile = new File(outputDir, "constraints_" + constraintsCount + ".json");
        OutputFormatter.writeJson(out, outFile);

        // Write the constraints XML
        File xmlFile = new File(outputDir, "constraints_" + constraintsCount + ".xml");
        BindFactory optFactory = OptimizationBindFactory.getInstance();
        optFactory.marshalToXml(objConstraints, xmlFile);

        constraintsCount++;
    }

    // ── DUMP_INTERNALS ───────────────────────────────────────────────

    private void runDumpInternals(File outputDir) throws Exception {
        ObjectNode out = OutputFormatter.newObject();

        // Physical parameters at 72F
        out.put("speedOfSound", params.getSpeedOfSound());
        out.put("rho", params.getRho());
        out.put("eta", params.getEta());
        out.put("gamma", params.getGamma());
        out.put("specificHeat", params.getSpecificHeat());
        out.put("alphaConstant", params.getAlphaConstant());
        out.put("temperature_C", params.getTemperature());

        // Wave number at 440 Hz
        double waveNumber440 = params.calcWaveNumber(440.0);
        out.put("waveNumber_440Hz", waveNumber440);

        // Z0 at bore radius (0.75" / 2 = 0.375" = 0.009525 m)
        double boreRadius = instrument.getBorePoint().get(0).getBoreDiameter() / 2.0;
        out.put("z0_boreRadius", params.calcZ0(boreRadius));
        out.put("boreRadius_m", boreRadius);

        // Component count after compilation
        out.put("componentCount", instrument.getComponents().size());

        // Headspace info
        if (instrument.getMouthpiece() != null && instrument.getMouthpiece().getHeadspace() != null) {
            out.put("headspaceSections", instrument.getMouthpiece().getHeadspace().size());
        }

        File outFile = new File(outputDir, "internals_" + internalsCount + ".json");
        OutputFormatter.writeJson(out, outFile);
        internalsCount++;
    }

    // ── Constraint Loading ─────────────────────────────────────────

    /// Load default constraints for a given objective function from the oracle.
    /// Looks up constraints by objective function name and hole count.
    private Constraints loadDefaultConstraints(String objectiveFunctionName) throws Exception {
        int nHoles = instrument.getHole().size();
        // Try the oracle constraints directory
        File constraintsFile;
        if ("FippleFactorObjectiveFunction".equals(objectiveFunctionName)) {
            constraintsFile = new File(scenariosDir,
                    "../../oracle/v2.6.0/constraints/NafStudyModel/FippleFactorObjectiveFunction/"
                            + nHoles + "/" + nHoles + "_holes.xml");
        } else {
            // For other objective functions, constraints should be provided in the scenario inputs
            return constraints;
        }

        if (!constraintsFile.exists()) {
            System.err.println("  WARNING: Constraints file not found: " + constraintsFile);
            return null;
        }

        BindFactory optFactory = OptimizationBindFactory.getInstance();
        return (Constraints) optFactory.unmarshalXml(constraintsFile, true);
    }

    // ── Objective Function Factory ───────────────────────────────────

    private BaseObjectiveFunction createObjectiveFunction(
            String name,
            InstrumentCalculator calc,
            Tuning tun,
            EvaluatorInterface eval) {
        try {
            switch (name) {
                case "FippleFactorObjectiveFunction":
                    return new FippleFactorObjectiveFunction(calc, tun, eval);
                case "HoleFromTopObjectiveFunction":
                    return new HoleFromTopObjectiveFunction(calc, tun, eval,
                            BoreLengthAdjustmentInterface.BoreLengthAdjustmentType.PRESERVE_BORE);
                case "NafHoleSizeObjectiveFunction":
                    return new NafHoleSizeObjectiveFunction(calc, tun, eval);
                case "HoleGroupFromTopObjectiveFunction":
                    return new HoleGroupFromTopObjectiveFunction(calc, tun, eval,
                            getHoleGroups(),
                            BoreLengthAdjustmentInterface.BoreLengthAdjustmentType.PRESERVE_BORE);
                case "SingleTaperNoHoleGroupingFromTopObjectiveFunction":
                    return new SingleTaperNoHoleGroupingFromTopObjectiveFunction(calc, tun, eval);
                case "SingleTaperHoleGroupFromTopObjectiveFunction":
                    return new SingleTaperHoleGroupFromTopObjectiveFunction(calc, tun, eval,
                            getHoleGroups(),
                            BoreLengthAdjustmentInterface.BoreLengthAdjustmentType.PRESERVE_TAPER);
                default:
                    throw new IllegalArgumentException("Unknown objective function: " + name);
            }
        } catch (Exception e) {
            throw new RuntimeException("Failed to create objective function: " + name, e);
        }
    }

    /// Extract hole groups from constraints, or use default for 6-hole NAF.
    private int[][] getHoleGroups() {
        if (constraints != null) {
            int[][] groups = constraints.getHoleGroupsArray();
            if (groups != null) {
                return groups;
            }
        }
        // Default 6-hole NAF groups
        int nHoles = instrument.getHole().size();
        if (nHoles == 6) {
            return new int[][]{{0, 1, 2}, {3, 4, 5}};
        } else if (nHoles == 7) {
            return new int[][]{{0, 1, 2}, {3, 4, 5}, {6}};
        }
        // Fallback: each hole in its own group
        int[][] groups = new int[nHoles][];
        for (int i = 0; i < nHoles; i++) {
            groups[i] = new int[]{i};
        }
        return groups;
    }
}
