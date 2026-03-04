# FEATURE_MATRIX.md

## Purpose

Enumerate every tool + optimizer per study model and map it to:
(a) fixture coverage and (b) implementation milestone.

Legend:

* Baseline: present in v2.6.0
* Fixture: scenario IDs in golden/scenarios/
* Milestone: M2/M3/M4/M5 as defined in PORT_SPEC.md

---

## Global Tools (all study models)

| Feature                   | Baseline | Fixture(s)        | Milestone | Notes                                |
| ------------------------- | -------: | ----------------- | --------- | ------------------------------------ |
| Open/save Instrument XML  |        ✅ | XML-RT-01 ¹       | M4        | Round-trip + semantic validation     |
| Open/save Tuning XML      |        ✅ | XML-RT-02 ¹       | M4        | Includes measured + min/max variants |
| Open/save Constraints XML |        ✅ | CONSTRAINTS-01/02 | M3        | Constraints ordering ABI             |
| Drag & drop open          |        ✅ | UX-01             | M4        | Web drop zone parity                 |
| Calculate tuning (table)  |        ✅ | EVAL-*, NAF-BULK-EVAL, WHISTLE-BULK-EVAL, FLUTE-BULK-EVAL | M2/M5.2/M5.3 | NAF: 36 combos; Whistle: 16; Flute: 8 |
| Graph tuning              |        ✅ | GRAPH-01          | M5        | Curve samples                        |
| Note spectrum graph       |        ✅ | SPEC-01           | M5        | Curve samples + markers              |
| Supplementary info table  |        ✅ | SUP-01            | M5        | Numeric outputs                      |
| Sketch instrument         |        ✅ | SKETCH-01         | M5        | Numeric geometry export              |
| Compare instruments       |        ✅ | CMP-01            | M5        | Diff summary                         |

---

## NAF Study Model

| Optimizer / Calibrator          | Baseline | Fixture(s)   | Milestone | Notes        |
| ------------------------------- | -------: | ------------ | --------- | ------------ |
| Fipple factor calibrator        |        ✅ | NAF-FF-02/03 | M3        | Load-bearing |
| Hole size                       |        ✅ | NAF-HS-01    | M3        |              |
| Hole size + position (from top) |        ✅ | NAF-OPT-01   | M3        |              |
| Grouped hole size + position    |        ✅ | NAF-GRP-01   | M5        |              |
| Taper variants                  |        ✅ | NAF-TAPER-01 | M5        |              |

---

## Whistle Study Model

| Feature / Optimizer     | Baseline | Fixture(s)  | Milestone | Notes                       |
| ----------------------- | -------: | ----------- | --------- | --------------------------- |
| Evaluation parity       |        ✅ | WHISTLE-BULK-EVAL, WH-ZSAMPLE | M5.2 | 2 instruments × 8 tunings = 16 combos, 272 fingerings |
| Window height calibrator|        ✅ | WH-CAL/calib_window_height | M5.4 | 1D Brent, FmaxEvaluator |
| Beta calibrator         |        ✅ | WH-CAL/calib_beta | M5.4 | 1D Brent, FminEvaluator |
| Joint calibrator        |        ✅ | WH-CAL/calib_joint | M5.4 | 2D BOBYQA, FminmaxEvaluator |
| Hole size               |        ✅ | WH-OPT/opt_hole_size | M5.4 | N-dim BOBYQA |
| Hole position           |        ✅ | WH-OPT/opt_hole_position | M5.4 | (N+1)-dim BOBYQA |
| Hole size + position    |        ✅ | WH-OPT/opt_hole | M5.4 | (2N+1)-dim merged BOBYQA |
| Hole spacing (global)   |        ✅ | —            | M5        | DIRECT-C→BOBYQA, session dispatched |
| Hole size+spacing (global)|      ✅ | —            | M5        | DIRECT-C→BOBYQA, session dispatched |
| Basic taper             |        ✅ | —            | M5        | 2D BOBYQA |
| Bore diameter from top  |        ✅ | WH-BORE-02   | M5        | N-dim, auto Brent for 1D |
| Bore diameter from bottom|       ✅ | WH-BORE-01   | M5        | N-dim, auto Brent for 1D |
| Bore spacing from top   |        ✅ | —            | M5        | Upper bound clamping |
| Stopper position        |        ✅ | —            | M5        | 1D Brent, preserveTaper flag |
| Headjoint               |        ✅ | —            | M5        | Stopper + bore dia from top, 40K evals |
| Hole + taper            |        ✅ | —            | M5        | MOVE_BOTTOM, 20K evals |
| Hole + bore dia top     |        ✅ | —            | M5        | PRESERVE_TAPER, 50K evals |
| Hole + bore dia bottom  |        ✅ | —            | M5        | MOVE_BOTTOM, 50K evals |
| Hole + bore spacing     |        ✅ | —            | M5        | PRESERVE_TAPER, 0.9e-6 stopping |
| Hole + bore position    |        ✅ | —            | M5        | PRESERVE_BELL, 0.9e-6 stopping |
| Hole + bore from bottom |        ✅ | —            | M5        | PRESERVE_BELL, 60K evals |
| Hole + headjoint        |        ✅ | —            | M5        | PRESERVE_TAPER, 50K evals |
| Global hole + bore (2)  |        ✅ | —            | M5        | DIRECT-C→BOBYQA |

---

## Flute (Transverse)

| Feature / Optimizer         | Baseline | Fixture(s)  | Milestone | Notes                       |
| --------------------------- | -------: | ----------- | --------- | --------------------------- |
| Evaluation parity           |        ✅ | FLUTE-BULK-EVAL, FL-ZSAMPLE | M5.3 | 2 instruments × 4 tunings = 8 combos, 110 fingerings |
| Airstream length calibrator |        ✅ | FL-CAL/calib_airstream_length | M5.5 | 1D Brent, FmaxEvaluator |
| Beta calibrator             |        ✅ | FL-CAL/calib_beta | M5.5 | 1D Brent, FminEvaluator (reused from Whistle) |
| Joint calibrator            |        ✅ | FL-CAL/calib_joint | M5.5 | 2D BOBYQA, FminmaxEvaluator |
| Hole size                   |        ✅ | FL-OPT/opt_hole_size | M5.5 | N-dim BOBYQA (reused from Whistle) |
| Hole position               |        ✅ | FL-OPT/opt_hole_position | M5.5 | (N+1)-dim BOBYQA (reused from Whistle) |
| Hole size + position        |        ✅ | FL-OPT/opt_hole | M5.5 | (2N+1)-dim merged BOBYQA (reused from Whistle) |
| Stopper position        |        ✅ | —            | M5        | 1D Brent, preserveTaper flag |
| Headjoint               |        ✅ | —            | M5        | Stopper + bore dia from top |
| Bore diameter/spacing   |        ✅ | —            | M5        | Inherited from Whistle |
| Hole + bore (4 merged)  |        ✅ | —            | M5        | 15-20 dims, various modes |
| Global hole + bore (2)  |        ✅ | —            | M5        | DIRECT-C→BOBYQA |

---

## Reed

| Feature / Optimizer          | Baseline | Fixture(s) | Milestone | Notes                    |
| ---------------------------- | -------: | ---------- | --------- | ------------------------ |
| Evaluation parity            |        ✅ | RD-BULK-EVAL, RD-ZSAMPLE | M5.6 | 7 combos, 72 fingerings, 0.000011 cents |
| Reed mouthpiece model        |        ✅ | — | M5.6 | SimpleReed: X = alpha × 1e-3 × freq + beta |
| Reed calibrator (alpha/beta) |        ✅ | RD-CAL/calib_joint | M5.7 | 2D BOBYQA, CentDeviationEvaluator |
| Hole size                    |        ✅ | — | M5.7 | N-dim BOBYQA (reused from Whistle) |
| Hole position                |        ✅ | — | M5.7 | (N+1)-dim BOBYQA (reused from Whistle) |
| Hole size + position         |        ✅ | — | M5.7 | (2N+1)-dim merged BOBYQA (reused from Whistle) |
| Hole size+spacing (global)   |        ✅ | — | M5        | DIRECT-C→BOBYQA, session dispatched |
| Bore diameter from bottom    |        ✅ | — | M5        | N-dim BOBYQA, auto Brent for 1D |
| Bore position                |        ✅ | RD-BORE-01 | M5        | Mixed dims (1 abs + fractions) |
| Bore from bottom (merged)    |        ✅ | — | M5        | Position + diameter, 40K evals |
| Hole + bore (3 merged)       |        ✅ | — | M5        | PRESERVE_BELL mode |
| Global hole + bore dia       |        ✅ | — | M5        | DIRECT-C→BOBYQA |
| Reed validity rule           |        ✅ | RD-VAL-01  | M5        | mouthpiece position rule |

---

## Optimization Engine (shared)

| Engine Feature         | Baseline | Fixture(s) | Milestone | Notes          |
| ---------------------- | -------: | ---------- | --------- | -------------- |
| Brent (1D)             |        ✅ | NAF-FF-02  | M3        |                |
| BOBYQA (bounded local) |        ✅ | NAF-OPT-01 | M3        | high fidelity  |
| DIRECT-C + refine      |        ✅ | DIRECT-01  | M5        | Engine done; golden fixture pending |
| Multi-start            |        ✅ | MS-01      | M5        | Engine done; golden fixture pending |
| Two-stage multi-start  |        ✅ | MS-02      | M5        | seeded         |

---

## Tuning Wizard Components

| Component                | Baseline | Fixture(s) | Milestone | Notes                |
| ------------------------ | -------: | ---------- | --------- | -------------------- |
| Symbol lists             |        ✅ | WIZ-02     | M5        | reusable             |
| Temperaments             |        ✅ | WIZ-02     | M5        | reusable             |
| Scales (intervals/freqs) |        ✅ | WIZ-01/02  | M5        | reusable             |
| Fingering patterns       |        ✅ | WIZ-02     | M5        | reusable             |
| Final tuning generation  |        ✅ | WIZ-01     | M5        | parity of output XML |

---

¹ Fixture IDs without existing scenario files in `golden/scenarios/` are planned for their target milestone.
