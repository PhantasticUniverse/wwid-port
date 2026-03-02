package com.widgolden;

import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.File;
import java.util.Arrays;
import java.util.List;
import java.util.stream.Collectors;

/// CLI entry point for golden fixture generation.
///
/// Usage:
///   ./gradlew run --args="--all"             Run all scenarios
///   ./gradlew run --args="NAF-FF-01"         Run a single scenario
///   ./gradlew run --args="NAF-FF-01 NAF-FF-02"  Run multiple scenarios
public class GoldenHarnessMain {

    private static final String SCENARIOS_DIR = "../golden/scenarios";
    private static final String EXPECTED_DIR = "../golden/expected";

    public static void main(String[] args) throws Exception {
        if (args.length == 0) {
            System.err.println("Usage: golden-harness [--all | <scenario-id> ...]");
            System.exit(1);
        }

        File scenariosDir = new File(SCENARIOS_DIR).getCanonicalFile();
        File expectedDir = new File(EXPECTED_DIR).getCanonicalFile();

        if (!scenariosDir.isDirectory()) {
            System.err.println("Scenarios directory not found: " + scenariosDir);
            System.exit(1);
        }

        List<String> scenarioIds;
        if (args.length == 1 && "--all".equals(args[0])) {
            // Find all .json files in the scenarios directory
            File[] jsonFiles = scenariosDir.listFiles((dir, name) -> name.endsWith(".json"));
            if (jsonFiles == null || jsonFiles.length == 0) {
                System.err.println("No scenario files found in: " + scenariosDir);
                System.exit(1);
                return;
            }
            Arrays.sort(jsonFiles);
            scenarioIds = Arrays.stream(jsonFiles)
                    .map(f -> f.getName().replace(".json", ""))
                    .collect(Collectors.toList());
        } else {
            scenarioIds = Arrays.asList(args);
        }

        System.out.println("Golden Harness: running " + scenarioIds.size() + " scenario(s)");

        ObjectMapper jsonMapper = OutputFormatter.mapper();
        int failures = 0;

        for (String id : scenarioIds) {
            File scenarioFile = new File(scenariosDir, id + ".json");
            if (!scenarioFile.exists()) {
                System.err.println("Scenario file not found: " + scenarioFile);
                failures++;
                continue;
            }

            try {
                Scenario scenario = jsonMapper.readValue(scenarioFile, Scenario.class);
                scenario.id = id;
                ScenarioRunner runner = new ScenarioRunner(scenario, scenariosDir, expectedDir);
                runner.run();
            } catch (Exception e) {
                System.err.println("FAILED: " + id);
                e.printStackTrace();
                failures++;
            }
        }

        System.out.println("\nResults: " + (scenarioIds.size() - failures) + "/" +
                scenarioIds.size() + " scenarios succeeded");

        if (failures > 0) {
            System.exit(1);
        }
    }
}
