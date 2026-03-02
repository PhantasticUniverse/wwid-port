package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.apache.commons.math3.analysis.MultivariateFunction;
import org.apache.commons.math3.optim.*;
import org.apache.commons.math3.optim.nonlinear.scalar.GoalType;
import org.apache.commons.math3.optim.nonlinear.scalar.ObjectiveFunction;
import org.apache.commons.math3.optim.nonlinear.scalar.noderiv.BOBYQAOptimizer;

import java.io.File;

/**
 * Generates reference data from Apache Commons Math 3's BOBYQAOptimizer
 * for parity testing with the Rust port.
 *
 * Writes JSON with exact final point, value, and evaluation count for each test.
 */
public class BobyqaReferenceDriver {

    private static final ObjectMapper mapper = new ObjectMapper();

    public static void main(String[] args) throws Exception {
        ArrayNode results = mapper.createArrayNode();

        // ── 1. Powell's "points in square" test ─────────────────────────

        // M=5 points (N=10), NPT=16
        results.add(runPowell(5, 16, "powell_m5_npt16"));
        // M=5 points (N=10), NPT=21
        results.add(runPowell(5, 21, "powell_m5_npt21"));
        // M=10 points (N=20), NPT=26
        results.add(runPowell(10, 26, "powell_m10_npt26"));
        // M=10 points (N=20), NPT=41
        results.add(runPowell(10, 41, "powell_m10_npt41"));

        // ── 2. ACM3 test functions (dim=13, unbounded) ──────────────────

        int dim = 13;
        double[] startOnes = new double[dim];
        double[] startTenth = new double[dim];
        java.util.Arrays.fill(startOnes, 1.0);
        java.util.Arrays.fill(startTenth, 0.1);

        // Unbounded tests use very wide bounds
        double[] lo = new double[dim];
        double[] hi = new double[dim];
        java.util.Arrays.fill(lo, -1e6);
        java.util.Arrays.fill(hi, 1e6);

        results.add(runTest("sphere_13d", x -> sphere(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("cigar_13d", x -> cigar(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("tablet_13d", x -> tablet(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("cigtab_13d", x -> cigtab(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("twoaxes_13d", x -> twoAxes(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("elli_13d", x -> elli(x), dim, startOnes, lo, hi, 10.0, 1e-8, 1000));
        results.add(runTest("rosenbrock_13d", x -> rosenbrock(x), dim, startTenth, lo, hi, 10.0, 1e-8, 5000));
        results.add(runTest("ackley_13d", x -> ackley(x), dim, startTenth, lo, hi, 10.0, 1e-8, 5000));
        results.add(runTest("rastrigin_13d", x -> rastrigin(x), dim, startOnes, lo, hi, 10.0, 1e-8, 5000));

        // DiffPow uses dim=6
        double[] startOnes6 = new double[6];
        java.util.Arrays.fill(startOnes6, 1.0);
        double[] lo6 = new double[6];
        double[] hi6 = new double[6];
        java.util.Arrays.fill(lo6, -1e6);
        java.util.Arrays.fill(hi6, 1e6);
        results.add(runTest("diffpow_6d", x -> diffPow(x), 6, startOnes6, lo6, hi6, 10.0, 1e-8, 25000));

        // ── 3. Constrained Rosenbrock (dim=13, [-1, 2]) ─────────────────

        double[] loConst = new double[dim];
        double[] hiConst = new double[dim];
        java.util.Arrays.fill(loConst, -1.0);
        java.util.Arrays.fill(hiConst, 2.0);
        results.add(runTest("rosen_bounded_13d", x -> rosenbrock(x), dim, startTenth, loConst, hiConst, 10.0, 1e-8, 5000));

        // ── 4. Simple bounded quadratics for exact parity ───────────────

        results.add(runTest("quadratic_2d_bounded", x -> (x[0]-3)*(x[0]-3)+(x[1]-4)*(x[1]-4),
                2, new double[]{1.0, 1.0}, new double[]{0.0, 0.0}, new double[]{2.0, 2.0}, 0.5, 1e-8, 1000));

        results.add(runTest("quadratic_2d", x -> (x[0]-3)*(x[0]-3)+(x[1]-4)*(x[1]-4),
                2, new double[]{0.0, 0.0}, new double[]{-10.0, -10.0}, new double[]{10.0, 10.0}, 1.0, 1e-8, 1000));

        // ── Write output ────────────────────────────────────────────────

        File outDir = new File("../golden/expected/BOBYQA-REF");
        outDir.mkdirs();
        mapper.writerWithDefaultPrettyPrinter()
              .writeValue(new File(outDir, "reference_results.json"), results);

        System.out.println("Wrote " + results.size() + " test results to " + outDir.getAbsolutePath());

        // Also dump results to stdout for quick inspection
        for (int i = 0; i < results.size(); i++) {
            ObjectNode r = (ObjectNode) results.get(i);
            System.out.printf("%-25s  f=%.15e  evals=%d%n",
                r.get("name").asText(), r.get("value").asDouble(), r.get("evaluations").asInt());
        }
    }

    // ── Test runner ─────────────────────────────────────────────────────

    private static ObjectNode runTest(String name, MultivariateFunction func,
            int dim, double[] start, double[] lower, double[] upper,
            double initialTrust, double stoppingTrust, int maxEval) {
        int nInterp = 2 * dim + 1;
        BOBYQAOptimizer optimizer = new BOBYQAOptimizer(nInterp, initialTrust, stoppingTrust);

        PointValuePair result = optimizer.optimize(
            new MaxEval(maxEval),
            new ObjectiveFunction(func),
            GoalType.MINIMIZE,
            new InitialGuess(start),
            new SimpleBounds(lower, upper)
        );

        ObjectNode node = mapper.createObjectNode();
        node.put("name", name);
        node.put("dim", dim);
        node.put("nInterp", nInterp);
        node.put("initialTrust", initialTrust);
        node.put("stoppingTrust", stoppingTrust);
        node.put("maxEval", maxEval);
        node.put("value", result.getValue());
        node.put("evaluations", optimizer.getEvaluations());

        ArrayNode startArr = mapper.createArrayNode();
        for (double v : start) startArr.add(v);
        node.set("startPoint", startArr);

        ArrayNode lowerArr = mapper.createArrayNode();
        for (double v : lower) lowerArr.add(v);
        node.set("lowerBounds", lowerArr);

        ArrayNode upperArr = mapper.createArrayNode();
        for (double v : upper) upperArr.add(v);
        node.set("upperBounds", upperArr);

        ArrayNode pointArr = mapper.createArrayNode();
        for (double v : result.getPoint()) pointArr.add(v);
        node.set("point", pointArr);

        return node;
    }

    // ── Powell "points in square" test ───────────────────────────────────

    private static ObjectNode runPowell(int m, int npt, String name) {
        int n = 2 * m;
        double[] start = new double[n];
        for (int j = 0; j < m; j++) {
            double temp = (j + 1) * 2.0 * Math.PI / m;
            start[2 * j] = Math.cos(temp);
            start[2 * j + 1] = Math.sin(temp);
        }

        double[] lower = new double[n];
        double[] upper = new double[n];
        java.util.Arrays.fill(lower, -1.0);
        java.util.Arrays.fill(upper, 1.0);

        // Clamp start to bounds
        for (int i = 0; i < n; i++) {
            start[i] = Math.max(lower[i], Math.min(upper[i], start[i]));
        }

        MultivariateFunction func = x -> {
            double f = 0;
            for (int i = 1; i < m; i++) {
                for (int j = 0; j < i; j++) {
                    double dx = x[2*i] - x[2*j];
                    double dy = x[2*i+1] - x[2*j+1];
                    double temp = dx*dx + dy*dy;
                    temp = Math.max(temp, 1e-6);
                    f += 1.0 / Math.sqrt(temp);
                }
            }
            return f;
        };

        return runTest(name, func, n, start, lower, upper, 0.1, 1e-6, 500000);
    }

    // ── Test functions ──────────────────────────────────────────────────

    private static double sphere(double[] x) {
        double sum = 0;
        for (double v : x) sum += v * v;
        return sum;
    }

    private static double cigar(double[] x) {
        double factor = 1e6;
        double sum = x[0] * x[0];
        for (int i = 1; i < x.length; i++) sum += factor * x[i] * x[i];
        return sum;
    }

    private static double tablet(double[] x) {
        double factor = 1e6;
        double sum = factor * x[0] * x[0];
        for (int i = 1; i < x.length; i++) sum += x[i] * x[i];
        return sum;
    }

    private static double cigtab(double[] x) {
        double factor = 1e4;
        int n = x.length;
        double sum = x[0] * x[0] / factor + factor * x[n-1] * x[n-1];
        for (int i = 1; i < n - 1; i++) sum += x[i] * x[i];
        return sum;
    }

    private static double twoAxes(double[] x) {
        double factor = 1e12;
        int n = x.length;
        double sum = 0;
        for (int i = 0; i < n; i++) {
            if (i < n / 2) {
                sum += factor * x[i] * x[i];
            } else {
                sum += x[i] * x[i];
            }
        }
        return sum;
    }

    private static double elli(double[] x) {
        double factor = 1e6;
        int n = x.length;
        double sum = 0;
        for (int i = 0; i < n; i++) {
            sum += Math.pow(factor, (double) i / (n - 1)) * x[i] * x[i];
        }
        return sum;
    }

    private static double rosenbrock(double[] x) {
        double sum = 0;
        for (int i = 0; i < x.length - 1; i++) {
            double t = x[i] * x[i] - x[i + 1];
            sum += 100.0 * t * t + (x[i] - 1.0) * (x[i] - 1.0);
        }
        return sum;
    }

    private static double ackley(double[] x) {
        int n = x.length;
        double sum1 = 0, sum2 = 0;
        for (int i = 0; i < n; i++) {
            sum1 += x[i] * x[i];
            sum2 += Math.cos(2 * Math.PI * x[i]);
        }
        return 20.0 - 20.0 * Math.exp(-0.2 * Math.sqrt(sum1 / n))
                + Math.E - Math.exp(sum2 / n);
    }

    private static double rastrigin(double[] x) {
        double A = 10.0;
        double sum = 0;
        for (int i = 0; i < x.length; i++) {
            sum += x[i] * x[i] + A * (1.0 - Math.cos(2.0 * Math.PI * x[i]));
        }
        return sum;
    }

    private static double diffPow(double[] x) {
        int n = x.length;
        double sum = 0;
        for (int i = 0; i < n; i++) {
            sum += Math.pow(Math.abs(x[i]), 2.0 + 10.0 * i / (n - 1.0));
        }
        return sum;
    }
}
