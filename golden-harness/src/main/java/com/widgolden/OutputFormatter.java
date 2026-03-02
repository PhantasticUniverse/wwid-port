package com.widgolden;

import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.apache.commons.math3.complex.Complex;

import java.io.File;
import java.io.IOException;
import java.util.List;

/// Formats golden fixture outputs as deterministic JSON with full IEEE 754
/// precision. Uses Jackson with explicit double formatting to prevent
/// precision loss in fixture files.
public class OutputFormatter {

    private static final ObjectMapper mapper = createMapper();

    private static ObjectMapper createMapper() {
        ObjectMapper m = new ObjectMapper();
        m.enable(SerializationFeature.INDENT_OUTPUT);
        m.configure(JsonGenerator.Feature.WRITE_BIGDECIMAL_AS_PLAIN, true);
        return m;
    }

    public static ObjectMapper mapper() {
        return mapper;
    }

    public static ObjectNode newObject() {
        return mapper.createObjectNode();
    }

    public static ArrayNode newArray() {
        return mapper.createArrayNode();
    }

    /// Write a JSON node to a file.
    public static void writeJson(Object node, File file) throws IOException {
        file.getParentFile().mkdirs();
        mapper.writeValue(file, node);
    }

    /// Format an eval result: array of {note, targetFreq, predictedFreq, cents}.
    public static ArrayNode formatEvalResult(
            List<String> noteNames,
            List<Double> targetFreqs,
            List<Double> predictedFreqs,
            double[] centsDeviation) {
        ArrayNode arr = newArray();
        for (int i = 0; i < noteNames.size(); i++) {
            ObjectNode entry = newObject();
            entry.put("note", noteNames.get(i));
            entry.put("targetFreq", targetFreqs.get(i));
            if (predictedFreqs.get(i) != null) {
                entry.put("predictedFreq", predictedFreqs.get(i));
            } else {
                entry.putNull("predictedFreq");
            }
            entry.put("cents", centsDeviation[i]);
            arr.add(entry);
        }
        return arr;
    }

    /// Format a Z-sample result: array of {frequency, zReal, zImag}.
    public static ArrayNode formatZSample(
            List<Double> frequencies,
            List<Complex> impedances) {
        ArrayNode arr = newArray();
        for (int i = 0; i < frequencies.size(); i++) {
            ObjectNode entry = newObject();
            entry.put("frequency", frequencies.get(i));
            Complex z = impedances.get(i);
            entry.put("zReal", z.getReal());
            entry.put("zImag", z.getImaginary());
            arr.add(entry);
        }
        return arr;
    }

    /// Format a calibration result.
    public static ObjectNode formatCalibrationResult(
            double initialFippleFactor,
            double finalFippleFactor,
            double initialNorm,
            double finalNorm) {
        ObjectNode obj = newObject();
        obj.put("initialFippleFactor", initialFippleFactor);
        obj.put("finalFippleFactor", finalFippleFactor);
        obj.put("initialNorm", initialNorm);
        obj.put("finalNorm", finalNorm);
        return obj;
    }

    /// Format an optimization result.
    public static ObjectNode formatOptimizationResult(
            double initialNorm,
            double finalNorm,
            int evaluations,
            double[] initialGeometry,
            double[] finalGeometry) {
        ObjectNode obj = newObject();
        obj.put("initialNorm", initialNorm);
        obj.put("finalNorm", finalNorm);
        obj.put("evaluations", evaluations);
        obj.set("initialGeometry", doubleArrayToJson(initialGeometry));
        obj.set("finalGeometry", doubleArrayToJson(finalGeometry));
        return obj;
    }

    /// Format constraints bounds result.
    public static ObjectNode formatConstraintsResult(
            double[] lowerBounds,
            double[] upperBounds) {
        ObjectNode obj = newObject();
        obj.set("lowerBounds", doubleArrayToJson(lowerBounds));
        obj.set("upperBounds", doubleArrayToJson(upperBounds));
        return obj;
    }

    private static ArrayNode doubleArrayToJson(double[] arr) {
        ArrayNode node = newArray();
        for (double v : arr) {
            node.add(v);
        }
        return node;
    }
}
