package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.wwidesigner.geometry.Instrument;
import com.wwidesigner.geometry.bind.GeometryBindFactory;
import com.wwidesigner.modelling.*;
import com.wwidesigner.note.Fingering;
import com.wwidesigner.note.Tuning;
import com.wwidesigner.note.bind.NoteBindFactory;
import com.wwidesigner.util.BindFactory;
import com.wwidesigner.util.Constants.TemperatureType;
import com.wwidesigner.util.PhysicalParameters;
import org.apache.commons.math3.complex.Complex;

import java.io.File;
import java.util.*;

/**
 * Dumps impedance (Z) values for one representative Reed combo:
 * SampleChanter + A3-ClosedFingering.
 *
 * For each fingering in the tuning, computes Z at the target frequency
 * and records the real and imaginary parts. Used to verify the reed
 * mouthpiece transfer matrix implementation.
 *
 * Uses {@code SimpleReedCalculator}, matching the wiring in
 * {@code ReedStudyModel}.
 *
 * Output: golden/expected/RD-ZSAMPLE/z_samples.json
 */
public class ReedZSampleDriver {

    private static final String ORACLE_BASE = "../oracle/v2.6.0";
    private static final String OUTPUT_DIR = "../golden/expected/RD-ZSAMPLE";

    private static final String INSTRUMENT_FILE = "SampleChanter.xml";
    private static final String TUNING_FILE = "A3-ClosedFingering.xml";

    public static void main(String[] args) throws Exception {
        ObjectMapper mapper = OutputFormatter.mapper();

        PhysicalParameters params = new PhysicalParameters(72.0, TemperatureType.F);

        File instrDir = new File(ORACLE_BASE + "/ReedStudy/instruments").getCanonicalFile();
        File tuningDir = new File(ORACLE_BASE + "/ReedStudy/tunings").getCanonicalFile();
        File outDir = new File(OUTPUT_DIR);
        outDir.mkdirs();

        System.out.printf("Z-sample: %s + %s%n", INSTRUMENT_FILE, TUNING_FILE);

        // Load instrument
        BindFactory geoFactory = GeometryBindFactory.getInstance();
        Instrument instrument = (Instrument) geoFactory.unmarshalXml(
            new File(instrDir, INSTRUMENT_FILE), true);
        instrument.updateComponents();

        // Load tuning
        BindFactory noteFactory = NoteBindFactory.getInstance();
        Tuning tuning = (Tuning) noteFactory.unmarshalXml(
            new File(tuningDir, TUNING_FILE), true);

        // Create calculator (SimpleReedCalculator, no LinearV tuner)
        SimpleReedCalculator calculator = new SimpleReedCalculator();
        calculator.setInstrument(instrument);
        calculator.setPhysicalParameters(params);

        // For each fingering, compute Z at the target frequency
        List<Fingering> fingeringTargets = tuning.getFingering();
        ArrayNode results = mapper.createArrayNode();

        for (Fingering f : fingeringTargets) {
            if (f.getNote() != null && f.getNote().getFrequency() != null) {
                double targetFreq = f.getNote().getFrequency();
                Complex z = calculator.calcZ(targetFreq, f);

                ObjectNode entry = mapper.createObjectNode();
                entry.put("note", f.getNote().getName() != null
                    ? f.getNote().getName() : "");
                entry.put("frequency", targetFreq);
                entry.put("zReal", z.getReal());
                entry.put("zImag", z.getImaginary());
                results.add(entry);

                System.out.printf("  %s @ %.2f Hz: Z = %.6f + %.6fi%n",
                    f.getNote().getName(), targetFreq, z.getReal(), z.getImaginary());
            }
        }

        // Write output
        File outFile = new File(outDir, "z_samples.json");
        mapper.writerWithDefaultPrettyPrinter().writeValue(outFile, results);

        System.out.printf("%nZ-sample complete: %d fingerings%n", results.size());
        System.out.println("Output: " + outFile.getAbsolutePath());
    }
}
