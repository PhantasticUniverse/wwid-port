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
| Graph tuning              |        ✅ | GRAPH-WH          | M5        | 17 curves, 33 X/R sweep points each |
| Note spectrum graph       |        ✅ | SPEC-WH           | M5        | 2000 impedance+gain points, 5 checkpoints |
| Supplementary info table  |        ✅ | SUP-NAF/WH/FL/RD  | M5        | All 4 study models, air speed/flow/gain/Q |
| Sketch instrument         |        ✅ | SKETCH-NAF        | M5        | Bore, holes, mouthpiece, termination |
| Compare instruments       |        ✅ | CMP-NAF           | M5        | 25 diff rows, original vs optimized  |

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
| Basic taper             |        ✅ | WH-TAPER-01  | M5        | 2D BOBYQA |
| Bore diameter from top  |        ✅ | WH-BORE-02   | M5        | N-dim, auto Brent for 1D |
| Bore diameter from bottom|       ✅ | WH-BORE-01   | M5        | N-dim, auto Brent for 1D |
| Bore spacing from top   |        ✅ | WH-BORE-SPACING-01 | M5   | Upper bound clamping |
| Stopper position        |        ✅ | —            | M5        | 1D Brent, same code as FL-STOPPER-01 |
| Headjoint               |        ✅ | —            | M5        | Same code as FL-HEADJOINT-01 |
| Hole + taper            |        ✅ | WH-MERGED-04 | M5        | MOVE_BOTTOM, 20K evals |
| Hole + bore dia top     |        ✅ | WH-MERGED-01 | M5        | PRESERVE_TAPER, 50K evals |
| Hole + bore dia bottom  |        ✅ | WH-MERGED-02 | M5        | MOVE_BOTTOM, 50K evals |
| Hole + bore spacing     |        ✅ | WH-MERGED-03 | M5        | PRESERVE_TAPER, 0.9e-6 stopping |
| Hole + bore position    |        ✅ | —            | M5        | Same code as RD-MERGED-02 |
| Hole + bore from bottom |        ✅ | —            | M5        | Same code as RD-MERGED-03 |
| Hole + headjoint        |        ✅ | WH-MERGED-05 | M5        | PRESERVE_TAPER, 50K evals |
| Global hole + bore (2)  |        ✅ | —            | M5        | DIRECT-C→BOBYQA, engine tested by DIRECT-01 |

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
| Stopper position        |        ✅ | FL-STOPPER-01 | M5       | 1D Brent, preserveTaper flag |
| Headjoint               |        ✅ | FL-HEADJOINT-01 | M5     | Stopper + bore dia from top |
| Bore diameter/spacing   |        ✅ | FL-TAPER-01  | M5        | BasicTaperObjectiveFunction |
| Hole + bore (merged 1)  |        ✅ | FL-MERGED-01 | M5        | BoreDiameterFromBottom |
| Hole + bore (merged 2)  |        ✅ | FL-MERGED-02 | M5        | BoreSpacingFromTop |
| Hole + bore (merged 3)  |        ✅ | FL-MERGED-03 | M5        | BasicTaper |
| Hole + bore (merged 4)  |        ✅ | FL-MERGED-04 | M5        | Headjoint |
| Global hole + bore (2)  |        ✅ | —            | M5        | DIRECT-C→BOBYQA, engine tested by DIRECT-01 |

---

## Reed

| Feature / Optimizer          | Baseline | Fixture(s) | Milestone | Notes                    |
| ---------------------------- | -------: | ---------- | --------- | ------------------------ |
| Evaluation parity            |        ✅ | RD-BULK-EVAL, RD-ZSAMPLE | M5.6 | 7 combos, 72 fingerings, 0.000011 cents |
| Reed mouthpiece model        |        ✅ | — | M5.6 | SimpleReed: X = alpha × 1e-3 × freq + beta |
| Reed calibrator (alpha/beta) |        ✅ | RD-CAL/calib_joint | M5.7 | 2D BOBYQA, CentDeviationEvaluator |
| Hole size + position         |        ✅ | RD-OPT-01 | M5.7 | (2N+1)-dim merged BOBYQA |
| Hole size+spacing (global)   |        ✅ | — | M5        | DIRECT-C→BOBYQA, engine tested by DIRECT-01 |
| Bore diameter from bottom    |        ✅ | RD-BORE-02 | M5   | N-dim BOBYQA, auto Brent for 1D |
| Bore position                |        ✅ | RD-BORE-01 | M5   | Mixed dims (1 abs + fractions) |
| Bore from bottom (merged)    |        ✅ | RD-BORE-03 | M5   | Position + diameter, 40K evals |
| Hole + bore dia bottom       |        ✅ | RD-MERGED-01 | M5 | BoreDiameterFromBottom |
| Hole + bore position         |        ✅ | RD-MERGED-02 | M5 | BorePosition |
| Hole + bore from bottom      |        ✅ | RD-MERGED-03 | M5 | BoreFromBottom |
| Global hole + bore dia       |        ✅ | — | M5        | DIRECT-C→BOBYQA, engine tested by DIRECT-01 |
| Reed validity rule           |        ✅ | —          | M5        | mouthpiece pos = bore start (unit test) |

---

## Optimization Engine (shared)

| Engine Feature         | Baseline | Fixture(s) | Milestone | Notes          |
| ---------------------- | -------: | ---------- | --------- | -------------- |
| Brent (1D)             |        ✅ | NAF-FF-02  | M3        |                |
| BOBYQA (bounded local) |        ✅ | NAF-OPT-01 | M3        | high fidelity  |
| DIRECT-C + refine      |        ✅ | DIRECT-01  | M5        | Global optimizers use this engine |
| Multi-start            |        ✅ | —          | M5        | Covered by global optimizer session tests |
| Two-stage multi-start  |        ✅ | —          | M5        | Seeded, covered by session tests |

---

## Tuning Wizard Components

| Component                | Baseline | Fixture(s) | Milestone | Notes                |
| ------------------------ | -------: | ---------- | --------- | -------------------- |
| Symbol lists             |        ✅ | WIZ-RT     | M5        | scientific_sharps/flats factories    |
| Temperaments             |        ✅ | WIZ-RT     | M5        | ET + just intonation, XML round-trip |
| Scales (intervals/freqs) |        ✅ | WIZ-SCALE  | M5        | 16 notes, A4=440 Hz reference        |
| Fingering patterns       |        ✅ | WIZ-RT     | M5        | Parsed as Tuning with optional freqs |
| Final tuning generation  |        ✅ | WIZ-TUNING | M5        | 14 fingerings, scale+pattern merge   |

---

¹ Fixture IDs without existing scenario files in `golden/scenarios/` are planned for their target milestone.
