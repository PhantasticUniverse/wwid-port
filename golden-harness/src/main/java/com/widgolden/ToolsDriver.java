package com.widgolden;

import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.*;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Fingering;
import com.wwidesigner.note.Note;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;
import org.apache.commons.math3.complex.Complex;

import java.io.File;
import java.util.List;

/// Golden fixture driver for analysis tools.
///
/// Generates fixtures for:
/// - SUP-NAF: Supplementary info for NAF
/// - SUP-WH: Supplementary info for Whistle
/// - SUP-FL: Supplementary info for Flute
/// - SUP-RD: Supplementary info for Reed
/// - GRAPH-WH: Graph tuning for Whistle
/// - SPEC-WH: Note spectrum for Whistle
/// - SKETCH-NAF: Sketch instrument geometry
/// - CMP-NAF: Compare instruments before/after optimization
///
/// Run: ./gradlew run -PmainClass=com.widgolden.ToolsDriver
public class ToolsDriver {

    private static final String ORACLE = "../oracle/v2.6.0";
    private static final String OUTPUT_BASE = "../golden/expected";
    private static final double DELTA_F = 0.0012; // ~2 cents for Q factor derivative

    public static void main(String[] args) throws Exception {
        System.out.println("=== Analysis Tools Golden Fixture Driver ===\n");

        generateSupNaf();
        generateSupWhistle();
        generateSupFlute();
        generateSupReed();
        generateGraphTuning();
        generateNoteSpectrum();
        generateSketch();
        generateCompare();

        System.out.println("\nAll tool fixtures generated successfully.");
    }

    // ── Helper: Load instrument ──────────────────────────────────────

    private static Instrument loadInstrument(String path) throws Exception {
        BindFactory geoFactory = GeometryBindFactory.getInstance();
        Instrument instrument = (Instrument) geoFactory.unmarshalXml(new File(path), true);
        instrument.updateComponents();
        return instrument;
    }

    private static Tuning loadTuning(String path) throws Exception {
        BindFactory noteFactory = NoteBindFactory.getInstance();
        return (Tuning) noteFactory.unmarshalXml(new File(path), true);
    }

    private static PhysicalParameters params() {
        return new PhysicalParameters(72.0, TemperatureType.F);
    }

    // ── Helper: Q factor ─────────────────────────────────────────────

    /// Q = f0/2 * d/df(Im(Z)/Re(Z)), using finite-difference with DELTA_F.
    /// Matches SupplementaryInfoTable.Q() exactly.
    private static double qFactor(double freq, Complex z,
            InstrumentCalculator calculator, Fingering fingering) {
        double freqPlus = freq * (1 + DELTA_F);
        Complex zPlus = calculator.calcZ(freqPlus, fingering);
        return 0.25
                * (freq + freqPlus)
                * (zPlus.getImaginary() / zPlus.getReal() - z.getImaginary() / z.getReal())
                / (freqPlus - freq);
    }

    // ── Helper: Supplementary info for one study model ───────────────

    /// Generate supplementary info fixture for a given study model.
    ///
    /// Matches Java SupplementaryInfoTable.buildTable() exactly:
    /// - targetFreq: overridden to predictedFreq when usePredicted=true
    /// - Im(Z) correction: Im(Z(target_note_freq)) - Im(Z(predicted_note_freq))
    ///   with frequencyMax priority if both target and predicted have it
    /// - Air speed: Strouhal model at targetFreq (only for fipple/embouchure)
    /// - Air flow: speed × windway_area (only for fipple with windway height)
    /// - Gain: at predicted frequency, using predicted fingering
    /// - Q: at predicted frequency, using predicted fingering
    private static void generateSupplementaryFixture(
            String fixtureId,
            String instrumentPath,
            String tuningPath,
            InstrumentCalculator calculator,
            InstrumentTuner tuner,
            boolean usePredicted) throws Exception {

        System.out.println("Generating " + fixtureId + " fixture...");

        File outputDir = new File(OUTPUT_BASE, fixtureId);
        outputDir.mkdirs();

        Instrument instrument = loadInstrument(instrumentPath);
        Tuning tuning = loadTuning(tuningPath);

        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params());

        // Set up tuner
        tuner.setCalculator(calculator);
        tuner.setInstrument(instrument);
        tuner.setTuning(tuning);
        tuner.setParams(params());

        Mouthpiece mouthpiece = instrument.getMouthpiece();
        List<Fingering> fingeringsTarget = tuning.getFingering();
        List<Fingering> fingeringsPredicted = tuner.getPredictedTuning().getFingering();

        // Determine window length and windway area
        Double windowLength = null;
        Double windwayArea = null;
        if (mouthpiece.getFipple() != null) {
            windowLength = mouthpiece.getFipple().getWindowLength();
            if (mouthpiece.getFipple().getWindwayHeight() != null) {
                windwayArea = 1.0e6 * mouthpiece.getFipple().getWindowWidth()
                        * mouthpiece.getFipple().getWindwayHeight();
                if (windwayArea == 0.0) {
                    windwayArea = null;
                }
            }
        } else if (mouthpiece.getEmbouchureHole() != null) {
            windowLength = mouthpiece.getEmbouchureHole().getAirstreamLength();
        }

        ArrayNode rows = OutputFormatter.newArray();

        for (int i = 0; i < fingeringsTarget.size(); i++) {
            Note note = fingeringsTarget.get(i).getNote();
            Note predicted = fingeringsPredicted.get(i).getNote();
            Double targetFreq = note.getFrequency();
            Double predictedFreq = predicted.getFrequency();

            // Override target with predicted when usePredicted=true
            Double displayFreq = targetFreq;
            if (usePredicted && predictedFreq != null) {
                displayFreq = predictedFreq;
            }

            ObjectNode row = OutputFormatter.newObject();
            row.put("note", note.getName() != null ? note.getName() : "");

            if (displayFreq != null) {
                row.put("freq", displayFreq);
            } else {
                row.putNull("freq");
            }

            // Im(Z) correction: Im(Z(target)) - Im(Z(predicted))
            // with frequencyMax priority
            if (note.getFrequencyMax() != null && predicted.getFrequencyMax() != null) {
                double correction = calculator.calcZ(note.getFrequencyMax(),
                        fingeringsTarget.get(i)).getImaginary()
                        - calculator.calcZ(predicted.getFrequencyMax(),
                                fingeringsPredicted.get(i)).getImaginary();
                row.put("imZCorrection", correction);
            } else if (note.getFrequency() != null && predicted.getFrequency() != null) {
                double correction = calculator.calcZ(note.getFrequency(),
                        fingeringsTarget.get(i)).getImaginary()
                        - calculator.calcZ(predicted.getFrequency(),
                                fingeringsPredicted.get(i)).getImaginary();
                row.put("imZCorrection", correction);
            } else {
                row.putNull("imZCorrection");
            }

            // Air speed (only for fipple/embouchure instruments with window length)
            if (displayFreq != null && windowLength != null) {
                Complex zTarget = calculator.calcZ(displayFreq, fingeringsTarget.get(i));
                double speed = LinearVInstrumentTuner.velocity(displayFreq, windowLength, zTarget);
                row.put("airSpeed", speed);
                if (windwayArea != null) {
                    row.put("airFlowRate", speed * windwayArea);
                } else {
                    row.putNull("airFlowRate");
                }
            } else {
                row.putNull("airSpeed");
                row.putNull("airFlowRate");
            }

            // Gain and Q at predicted frequency
            if (predictedFreq != null) {
                Complex z = calculator.calcZ(predictedFreq, fingeringsPredicted.get(i));
                double gain = calculator.calcGain(predictedFreq, z);
                row.put("gain", gain);
                row.put("qFactor", qFactor(predictedFreq, z, calculator,
                        fingeringsPredicted.get(i)));
            } else {
                row.putNull("gain");
                row.putNull("qFactor");
            }

            rows.add(row);
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("instrumentName", instrument.getName());
        result.put("tuningName", tuning.getName());
        result.put("usePredicted", usePredicted);
        result.put("fingeringCount", fingeringsTarget.size());
        result.put("hasAirSpeed", windowLength != null);
        result.put("hasAirFlowRate", windwayArea != null);
        result.set("rows", rows);

        OutputFormatter.writeJson(result, new File(outputDir, "supplementary.json"));
        System.out.println("  " + fingeringsTarget.size() + " fingerings");
    }

    // ── SUP-NAF ──────────────────────────────────────────────────────

    private static void generateSupNaf() throws Exception {
        generateSupplementaryFixture(
                "SUP-NAF",
                ORACLE + "/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml",
                ORACLE + "/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml",
                new NAFCalculator(),
                new SimpleInstrumentTuner(),
                true);  // NAF uses usePredicted=true
    }

    // ── SUP-WH ───────────────────────────────────────────────────────

    private static void generateSupWhistle() throws Exception {
        generateSupplementaryFixture(
                "SUP-WH",
                ORACLE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml",
                ORACLE + "/WhistleStudy/tunings/SamplePVC-tuning.xml",
                new WhistleCalculator(),
                new LinearVInstrumentTuner(5),
                false);  // Whistle uses usePredicted=false
    }

    // ── SUP-FL ───────────────────────────────────────────────────────

    private static void generateSupFlute() throws Exception {
        generateSupplementaryFixture(
                "SUP-FL",
                ORACLE + "/FluteStudy/instruments/SamplePVC-Flute.xml",
                ORACLE + "/FluteStudy/tunings/D4-Equal.xml",
                new FluteCalculator(),
                new LinearVInstrumentTuner(5),
                false);  // Flute uses usePredicted=false
    }

    // ── SUP-RD ───────────────────────────────────────────────────────

    private static void generateSupReed() throws Exception {
        generateSupplementaryFixture(
                "SUP-RD",
                ORACLE + "/ReedStudy/instruments/SampleChanter.xml",
                ORACLE + "/ReedStudy/tunings/A3-ClosedFingering.xml",
                new SimpleReedCalculator(),
                new SimpleInstrumentTuner(),
                true);  // Reed uses usePredicted=true
    }

    // ── GRAPH-WH ─────────────────────────────────────────────────────

    /// Graph tuning fixture: per-fingering playing range curves for Whistle.
    ///
    /// For each fingering:
    /// - predicted frequency (from LinearV tuner)
    /// - fmin, fmax (playing range bounds)
    /// - 33 sample points of Im(Z)/Re(Z) from 0.95*fmin to 1.05*fmax
    private static void generateGraphTuning() throws Exception {
        System.out.println("Generating GRAPH-WH fixture...");

        File outputDir = new File(OUTPUT_BASE, "GRAPH-WH");
        outputDir.mkdirs();

        String instrumentPath = ORACLE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml";
        String tuningPath = ORACLE + "/WhistleStudy/tunings/SamplePVC-tuning.xml";

        Instrument instrument = loadInstrument(instrumentPath);
        Tuning tuning = loadTuning(tuningPath);
        PhysicalParameters pp = params();

        InstrumentCalculator calculator = new WhistleCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(pp);

        LinearVInstrumentTuner tuner = new LinearVInstrumentTuner(5);
        tuner.setCalculator(calculator);
        tuner.setInstrument(instrument);
        tuner.setTuning(tuning);
        tuner.setParams(pp);

        List<Fingering> fingeringsTarget = tuning.getFingering();
        List<Fingering> fingeringsPredicted = tuner.getPredictedTuning().getFingering();

        ArrayNode curves = OutputFormatter.newArray();

        for (int i = 0; i < fingeringsTarget.size(); i++) {
            Fingering targetF = fingeringsTarget.get(i);
            Note targetNote = targetF.getNote();
            Note predictedNote = fingeringsPredicted.get(i).getNote();

            Double targetFreq = targetNote.getFrequency();
            Double predictedFreq = predictedNote.getFrequency();
            Double fmax = predictedNote.getFrequencyMax();
            Double fmin = predictedNote.getFrequencyMin();

            ObjectNode curve = OutputFormatter.newObject();
            curve.put("note", targetNote.getName() != null ? targetNote.getName() : "");

            if (targetFreq != null) curve.put("targetFreq", targetFreq);
            else curve.putNull("targetFreq");

            if (predictedFreq != null) curve.put("predictedFreq", predictedFreq);
            else curve.putNull("predictedFreq");

            if (fmin != null) curve.put("fmin", fmin);
            else curve.putNull("fmin");

            if (fmax != null) curve.put("fmax", fmax);
            else curve.putNull("fmax");

            // Sweep 33 points for X/R ratio
            ArrayNode points = OutputFormatter.newArray();
            if (fmin != null && fmax != null && fmin > 0 && fmax > fmin) {
                double sweepLo = 0.95 * fmin;
                double sweepHi = 1.05 * fmax;
                double step = (sweepHi - sweepLo) / 32.0;
                for (int j = 0; j <= 32; j++) {
                    double freq = sweepLo + j * step;
                    Complex z = calculator.calcZ(freq, targetF);
                    double xOverR = z.getImaginary() / z.getReal();
                    ArrayNode pt = OutputFormatter.newArray();
                    pt.add(freq);
                    pt.add(xOverR);
                    points.add(pt);
                }
            }
            curve.set("points", points);

            curves.add(curve);
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("instrumentName", instrument.getName());
        result.put("tuningName", tuning.getName());
        result.put("fingeringCount", fingeringsTarget.size());
        result.set("curves", curves);

        OutputFormatter.writeJson(result, new File(outputDir, "graph_tuning.json"));
        System.out.println("  " + fingeringsTarget.size() + " curves");
    }

    // ── SPEC-WH ──────────────────────────────────────────────────────

    /// Note spectrum fixture: impedance + gain spectrum for fingering 0.
    ///
    /// 2000 frequency points from 0.45*target to 3.17*target.
    /// At each point: Im(Z)/Re(Z) ratio and loop gain.
    private static void generateNoteSpectrum() throws Exception {
        System.out.println("Generating SPEC-WH fixture...");

        File outputDir = new File(OUTPUT_BASE, "SPEC-WH");
        outputDir.mkdirs();

        String instrumentPath = ORACLE + "/WhistleStudy/instruments/SamplePVC-Whistle.xml";
        String tuningPath = ORACLE + "/WhistleStudy/tunings/SamplePVC-tuning.xml";

        Instrument instrument = loadInstrument(instrumentPath);
        Tuning tuning = loadTuning(tuningPath);
        PhysicalParameters pp = params();

        InstrumentCalculator calculator = new WhistleCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(pp);

        int fingeringIndex = 0;
        Fingering fingering = tuning.getFingering().get(fingeringIndex);
        Double targetFreq = fingering.getNote().getFrequency();
        if (targetFreq == null) {
            targetFreq = 440.0;
        }

        double freqLo = 0.45 * targetFreq;
        double freqHi = 3.17 * targetFreq;
        int nPoints = 2000;
        double step = (freqHi - freqLo) / (nPoints - 1);

        // Output 5 evenly-spaced checkpoint indices for compact verification
        // Plus the full spectrum for thoroughness
        ArrayNode checkpoints = OutputFormatter.newArray();
        int[] checkIndices = {0, 499, 999, 1499, 1999};

        ArrayNode allPoints = OutputFormatter.newArray();
        for (int i = 0; i < nPoints; i++) {
            double freq = freqLo + i * step;
            Complex z = calculator.calcZ(freq, fingering);
            double xOverR = z.getImaginary() / z.getReal();
            double gain = calculator.calcGain(freq, z);

            ObjectNode pt = OutputFormatter.newObject();
            pt.put("freq", freq);
            pt.put("impedanceRatio", xOverR);
            pt.put("loopGain", gain);
            allPoints.add(pt);

            for (int idx : checkIndices) {
                if (i == idx) {
                    ObjectNode cp = OutputFormatter.newObject();
                    cp.put("index", i);
                    cp.put("freq", freq);
                    cp.put("impedanceRatio", xOverR);
                    cp.put("loopGain", gain);
                    checkpoints.add(cp);
                    break;
                }
            }
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("instrumentName", instrument.getName());
        result.put("note", fingering.getNote().getName());
        result.put("targetFreq", targetFreq);
        result.put("fingeringIndex", fingeringIndex);
        result.put("numPoints", nPoints);
        result.put("freqLo", freqLo);
        result.put("freqHi", freqHi);
        result.set("checkpoints", checkpoints);
        result.set("points", allPoints);

        OutputFormatter.writeJson(result, new File(outputDir, "spectrum.json"));
        System.out.println("  " + nPoints + " spectrum points, " + checkIndices.length + " checkpoints");
    }

    // ── SKETCH-NAF ───────────────────────────────────────────────────

    /// Sketch fixture: instrument geometry extraction.
    ///
    /// Captures bore profile, holes, mouthpiece parameters, and termination.
    private static void generateSketch() throws Exception {
        System.out.println("Generating SKETCH-NAF fixture...");

        File outputDir = new File(OUTPUT_BASE, "SKETCH-NAF");
        outputDir.mkdirs();

        String instrumentPath = ORACLE + "/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml";
        Instrument instrument = loadInstrument(instrumentPath);

        ObjectNode result = OutputFormatter.newObject();
        result.put("instrumentName", instrument.getName());

        // Bore points
        ArrayNode borePoints = OutputFormatter.newArray();
        List<BorePoint> bore = instrument.getBorePoint();
        for (BorePoint bp : bore) {
            ObjectNode pt = OutputFormatter.newObject();
            pt.put("position", bp.getBorePosition());
            pt.put("diameter", bp.getBoreDiameter());
            borePoints.add(pt);
        }
        result.set("borePoints", borePoints);
        result.put("boreLength", bore.get(bore.size() - 1).getBorePosition());

        // Holes
        ArrayNode holes = OutputFormatter.newArray();
        for (int i = 0; i < instrument.getHole().size(); i++) {
            Hole h = instrument.getHole().get(i);
            ObjectNode hole = OutputFormatter.newObject();
            hole.put("name", h.getName() != null ? h.getName() : "Hole " + (i + 1));
            hole.put("position", h.getBorePosition());
            hole.put("diameter", h.getDiameter());
            hole.put("height", h.getHeight());
            holes.add(hole);
        }
        result.set("holes", holes);

        // Mouthpiece
        Mouthpiece mp = instrument.getMouthpiece();
        ObjectNode mouthpiece = OutputFormatter.newObject();
        mouthpiece.put("position", mp.getPosition());
        if (mp.getBeta() != null) {
            mouthpiece.put("beta", mp.getBeta());
        }

        if (mp.getFipple() != null) {
            mouthpiece.put("type", "Fipple");
            Mouthpiece.Fipple fipple = mp.getFipple();
            mouthpiece.put("windowLength", fipple.getWindowLength());
            mouthpiece.put("windowWidth", fipple.getWindowWidth());
            if (fipple.getWindowHeight() != null) {
                mouthpiece.put("windowHeight", fipple.getWindowHeight());
            }
            if (fipple.getWindwayLength() != null) {
                mouthpiece.put("windwayLength", fipple.getWindwayLength());
            }
            if (fipple.getWindwayHeight() != null) {
                mouthpiece.put("windwayHeight", fipple.getWindwayHeight());
            }
            if (fipple.getFippleFactor() != null) {
                mouthpiece.put("fippleFactor", fipple.getFippleFactor());
            }
        } else if (mp.getEmbouchureHole() != null) {
            mouthpiece.put("type", "Embouchure");
            Mouthpiece.EmbouchureHole emb = mp.getEmbouchureHole();
            mouthpiece.put("length", emb.getLength());
            mouthpiece.put("width", emb.getWidth());
            mouthpiece.put("height", emb.getHeight());
            mouthpiece.put("airstreamLength", emb.getAirstreamLength());
            mouthpiece.put("airstreamHeight", emb.getAirstreamHeight());
        } else if (mp.getSingleReed() != null || mp.getDoubleReed() != null
                || mp.getLipReed() != null) {
            mouthpiece.put("type", "Reed");
        }
        result.set("mouthpiece", mouthpiece);

        // Termination
        ObjectNode termination = OutputFormatter.newObject();
        if (instrument.getTermination() != null) {
            termination.put("flangeDiameter", instrument.getTermination().getFlangeDiameter());
        }
        result.set("termination", termination);

        OutputFormatter.writeJson(result, new File(outputDir, "sketch.json"));
        System.out.println("  " + bore.size() + " bore points, "
                + instrument.getHole().size() + " holes");
    }

    // ── CMP-NAF ──────────────────────────────────────────────────────

    /// Compare fixture: field-by-field diff between original and optimized NAF.
    ///
    /// Uses the original 0.75-bore and the post-optimization instrument from
    /// the NAF-OPT-01 golden fixture.
    private static void generateCompare() throws Exception {
        System.out.println("Generating CMP-NAF fixture...");

        File outputDir = new File(OUTPUT_BASE, "CMP-NAF");
        outputDir.mkdirs();

        String oldPath = ORACLE + "/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml";
        String newPath = OUTPUT_BASE + "/NAF-OPT-01/instrument_after_optimize_0.xml";

        Instrument oldInst = loadInstrument(oldPath);
        Instrument newInst = loadInstrument(newPath);

        int precision = 5; // 5 decimal places (meters)
        double minDiff = Math.pow(10, -precision);

        ArrayNode rows = OutputFormatter.newArray();

        // Compare mouthpiece position
        addDiffRow(rows, "Mouthpiece", "Position",
                oldInst.getMouthpiece().getPosition(),
                newInst.getMouthpiece().getPosition(), minDiff);

        // Compare mouthpiece beta
        if (oldInst.getMouthpiece().getBeta() != null || newInst.getMouthpiece().getBeta() != null) {
            addDiffRow(rows, "Mouthpiece", "Beta",
                    oldInst.getMouthpiece().getBeta(),
                    newInst.getMouthpiece().getBeta(), minDiff);
        }

        // Compare fipple-specific fields
        if (oldInst.getMouthpiece().getFipple() != null && newInst.getMouthpiece().getFipple() != null) {
            Mouthpiece.Fipple oldF = oldInst.getMouthpiece().getFipple();
            Mouthpiece.Fipple newF = newInst.getMouthpiece().getFipple();
            addDiffRow(rows, "Mouthpiece", "Window Length",
                    oldF.getWindowLength(), newF.getWindowLength(), minDiff);
            addDiffRow(rows, "Mouthpiece", "Window Width",
                    oldF.getWindowWidth(), newF.getWindowWidth(), minDiff);
            if (oldF.getWindowHeight() != null || newF.getWindowHeight() != null) {
                addDiffRow(rows, "Mouthpiece", "Window Height",
                        oldF.getWindowHeight(), newF.getWindowHeight(), minDiff);
            }
            if (oldF.getFippleFactor() != null || newF.getFippleFactor() != null) {
                addDiffRow(rows, "Mouthpiece", "Fipple Factor",
                        oldF.getFippleFactor(), newF.getFippleFactor(), minDiff);
            }
        }

        // Compare holes (only if same count)
        if (oldInst.getHole().size() == newInst.getHole().size()) {
            for (int i = 0; i < oldInst.getHole().size(); i++) {
                Hole oldH = oldInst.getHole().get(i);
                Hole newH = newInst.getHole().get(i);
                String cat = "Hole " + (i + 1);
                addDiffRow(rows, cat, "Position", oldH.getBorePosition(), newH.getBorePosition(), minDiff);
                addDiffRow(rows, cat, "Diameter", oldH.getDiameter(), newH.getDiameter(), minDiff);
                addDiffRow(rows, cat, "Height", oldH.getHeight(), newH.getHeight(), minDiff);
            }
        }

        // Compare bore points
        int maxBore = Math.max(oldInst.getBorePoint().size(), newInst.getBorePoint().size());
        for (int i = 0; i < maxBore; i++) {
            String cat = "Bore Point " + (i + 1);
            Double oldPos = i < oldInst.getBorePoint().size()
                    ? oldInst.getBorePoint().get(i).getBorePosition() : null;
            Double newPos = i < newInst.getBorePoint().size()
                    ? newInst.getBorePoint().get(i).getBorePosition() : null;
            addDiffRow(rows, cat, "Position", oldPos, newPos, minDiff);

            Double oldDia = i < oldInst.getBorePoint().size()
                    ? oldInst.getBorePoint().get(i).getBoreDiameter() : null;
            Double newDia = i < newInst.getBorePoint().size()
                    ? newInst.getBorePoint().get(i).getBoreDiameter() : null;
            addDiffRow(rows, cat, "Diameter", oldDia, newDia, minDiff);
        }

        // Compare termination
        if (oldInst.getTermination() != null && newInst.getTermination() != null) {
            addDiffRow(rows, "Termination", "Flange Diameter",
                    oldInst.getTermination().getFlangeDiameter(),
                    newInst.getTermination().getFlangeDiameter(), minDiff);
        }

        ObjectNode result = OutputFormatter.newObject();
        result.put("oldName", oldInst.getName());
        result.put("newName", newInst.getName());
        result.put("precision", precision);
        result.put("rowCount", rows.size());
        result.set("rows", rows);

        OutputFormatter.writeJson(result, new File(outputDir, "compare.json"));
        System.out.println("  " + rows.size() + " diff rows");
    }

    /// Add a diff row if the values differ above the threshold.
    private static void addDiffRow(ArrayNode rows, String category, String field,
            Double oldVal, Double newVal, double minDiff) {
        if (oldVal == null && newVal == null) return;

        double diff;
        if (oldVal == null || newVal == null) {
            diff = Double.MAX_VALUE; // Always include if one is null
        } else {
            diff = Math.abs(newVal - oldVal);
        }
        if (diff < minDiff) return;

        ObjectNode row = OutputFormatter.newObject();
        row.put("category", category);
        row.put("field", field);
        if (oldVal != null) row.put("oldValue", oldVal);
        else row.putNull("oldValue");
        if (newVal != null) row.put("newValue", newVal);
        else row.putNull("newValue");
        if (oldVal != null && newVal != null) {
            row.put("difference", newVal - oldVal);
            if (oldVal != 0.0) {
                row.put("percentChange", 100.0 * (newVal - oldVal) / oldVal);
            } else {
                row.putNull("percentChange");
            }
        } else {
            row.putNull("difference");
            row.putNull("percentChange");
        }
        rows.add(row);
    }
}
