package com.widgolden;

import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import java.util.List;

/// Jackson POJO for scenario JSON files.
///
/// A scenario describes a study model, input files, and a sequence of
/// actions to execute against the oracle. Actions are processed in order;
/// instrument state carries forward between actions (e.g. CALIBRATE mutates
/// the instrument, and the next EVAL_TUNING uses the mutated version).
@JsonIgnoreProperties(ignoreUnknown = true)
public class Scenario {
    public String id;
    public String studyKind;
    public Inputs inputs;
    public List<Action> actions;

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class Inputs {
        /// Relative path to instrument XML (from golden/scenarios/).
        public String instrument;
        /// Relative path to tuning XML.
        public String tuning;
        /// Relative path to constraints XML (optional, needed for OPTIMIZE).
        public String constraints;
    }

    @JsonIgnoreProperties(ignoreUnknown = true)
    public static class Action {
        /// Action type: EVAL_TUNING, ZSAMPLE, CALIBRATE, OPTIMIZE,
        /// CREATE_DEFAULT_CONSTRAINTS, CREATE_BLANK_CONSTRAINTS,
        /// SET_FIPPLE_FACTOR, SET_WINDWAY_HEIGHT, RELOAD_INSTRUMENT,
        /// DUMP_INTERNALS.
        public String type;

        /// For ZSAMPLE: list of frequencies to sample.
        public List<Double> frequencies;
        /// For ZSAMPLE: if true, use all-closed fingering.
        public Boolean fingeringAllClosed;
        /// For ZSAMPLE: specific fingering index from tuning (0-based).
        public Integer fingeringIndex;

        /// For SET_FIPPLE_FACTOR / SET_WINDWAY_HEIGHT: value to set (null = set null).
        public Double value;

        /// For CALIBRATE / OPTIMIZE / CREATE_*_CONSTRAINTS: objective function class name.
        public String objectiveFunction;
    }
}
