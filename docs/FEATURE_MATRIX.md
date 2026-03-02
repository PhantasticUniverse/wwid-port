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
| Calculate tuning (table)  |        ✅ | EVAL-*            | M2        | Whistle/Flute include min/max        |
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

| Optimizer / Calibrator  | Baseline | Fixture(s)  | Milestone | Notes                       |
| ----------------------- | -------: | ----------- | --------- | --------------------------- |
| Whistle calibrator      |        ✅ | WH-CAL-01   | M5        | min/max evaluator           |
| Hole size               |        ✅ | WH-HS-01    | M5        |                             |
| Hole spacing            |        ✅ | WH-SP-01    | M5        |                             |
| Hole size + spacing     |        ✅ | WH-HSSP-01  | M5        |                             |
| Taper / bore optimizers |        ✅ | WH-TAPER-01 | M5        | Naming rules like Head/Body |

---

## Flute (Transverse)

| Optimizer / Calibrator      | Baseline | Fixture(s) | Milestone | Notes                  |
| --------------------------- | -------: | ---------- | --------- | ---------------------- |
| Flute calibrator            |        ✅ | FL-CAL-01  | M5        |                        |
| Stopper position optimizers |        ✅ | FL-STOP-01 | M5        | if present in baseline |
| Other flute optimizers      |        ✅ | FL-OPT-*   | M5        |                        |

---

## Reed

| Optimizer / Calibrator       | Baseline | Fixture(s) | Milestone | Notes                    |
| ---------------------------- | -------: | ---------- | --------- | ------------------------ |
| Reed calibrator (alpha/beta) |        ✅ | RD-CAL-01  | M5        |                          |
| Reed validity rule           |        ✅ | RD-VAL-01  | M5        | mouthpiece position rule |

---

## Optimization Engine (shared)

| Engine Feature         | Baseline | Fixture(s) | Milestone | Notes          |
| ---------------------- | -------: | ---------- | --------- | -------------- |
| Brent (1D)             |        ✅ | NAF-FF-02  | M3        |                |
| BOBYQA (bounded local) |        ✅ | NAF-OPT-01 | M3        | high fidelity  |
| DIRECT-C + refine      |        ✅ | DIRECT-01  | M5        | match baseline |
| Multi-start            |        ✅ | MS-01      | M5        | seeded         |
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
