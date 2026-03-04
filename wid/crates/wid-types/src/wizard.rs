//! Tuning wizard types: Scale, Temperament, and ScaleSymbolList.
//!
//! These types support the tuning wizard workflow:
//! 1. Pick note symbols (`ScaleSymbolList`)
//! 2. Pick interval ratios (`Temperament`)
//! 3. Generate a `Scale` (named notes with frequencies)
//! 4. Combine Scale + FingeringPattern → Tuning
//!
//! Java references: `com.wwidesigner.note.{Scale, Temperament, ScaleSymbolList}`.

use serde::{Deserialize, Serialize};

/// A named collection of notes with absolute frequencies.
///
/// Java reference: `Scale.java`. XML root: `<scale>`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "scale")]
pub struct Scale {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "note")]
    pub notes: Vec<ScaleNote>,
}

/// A single note in a scale: name + frequency.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ScaleNote {
    pub name: String,
    pub frequency: f64,
}

/// A set of frequency ratios defining interval relationships.
///
/// All ratios are >= 1.0; first ratio is always 1.0 (unison).
/// Java reference: `Temperament.java`. XML root: `<temperament>`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "temperament")]
pub struct Temperament {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "ratio")]
    pub ratios: Vec<f64>,
}

impl Temperament {
    /// 12-tone equal temperament, 3 octaves (37 ratios).
    ///
    /// `ratio[i] = 2^(i/12)` for `i = 0..36`.
    /// Matches Java's `Temperament.makeTET_12()`.
    pub fn equal_temperament_12() -> Self {
        let ratios: Vec<f64> = (0..37).map(|i| 2.0_f64.powf(i as f64 / 12.0)).collect();
        Temperament {
            name: "12-Tone Equal Temperament".to_string(),
            comment: Some("Chromatic, 12-tone, equal temperament, 3 octaves.".to_string()),
            ratios,
        }
    }

    /// 12-tone just intonation, 3 octaves (37 ratios).
    ///
    /// Uses traditional 5-limit ratios with a 7-limit tritone (7/5).
    /// Matches Java's `Temperament.makeJI_12()`.
    pub fn just_intonation_12() -> Self {
        // One octave of just intonation ratios (12 intervals)
        let octave: [f64; 12] = [
            1.0,
            16.0 / 15.0,
            9.0 / 8.0,
            6.0 / 5.0,
            5.0 / 4.0,
            4.0 / 3.0,
            7.0 / 5.0,
            3.0 / 2.0,
            8.0 / 5.0,
            5.0 / 3.0,
            9.0 / 5.0,
            15.0 / 8.0,
        ];

        let mut ratios = Vec::with_capacity(37);
        for oct in 0..3 {
            let mult = 2.0_f64.powi(oct);
            for &r in &octave {
                ratios.push(r * mult);
            }
        }
        // Add the final unison of the 4th octave
        ratios.push(8.0);

        Temperament {
            name: "12-Tone Just Intonation".to_string(),
            comment: Some("Chromatic, 12-tone, just intonation, 3 octaves.".to_string()),
            ratios,
        }
    }
}

/// A named set of note symbols for labeling scale degrees.
///
/// Java reference: `ScaleSymbolList.java`. XML root: `<scaleSymbolList>`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "scaleSymbolList")]
pub struct ScaleSymbolList {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "scaleSymbol")]
    pub symbols: Vec<String>,
}

impl ScaleSymbolList {
    /// Standard scientific pitch notation with sharps, 3 octaves starting at C0.
    ///
    /// Produces note names like: C0, C#0, D0, ..., B2.
    /// Covers the range typically used in woodwind instruments.
    pub fn scientific_sharps() -> Self {
        let notes = [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ];
        let mut symbols = Vec::new();
        for octave in 0..11 {
            for name in &notes {
                symbols.push(format!("{name}{octave}"));
            }
        }
        ScaleSymbolList {
            name: "Scientific pitch, sharps only".to_string(),
            comment: Some("Scientific pitch notation with sharps, C0 through B10.".to_string()),
            symbols,
        }
    }

    /// Standard scientific pitch notation with flats, 3 octaves starting at C0.
    pub fn scientific_flats() -> Self {
        let notes = [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ];
        let mut symbols = Vec::new();
        for octave in 0..11 {
            for name in &notes {
                symbols.push(format!("{name}{octave}"));
            }
        }
        ScaleSymbolList {
            name: "Scientific pitch, flats only".to_string(),
            comment: Some("Scientific pitch notation with flats, C0 through B10.".to_string()),
            symbols,
        }
    }
}

// ── Scale generation ──────────────────────────────────────────────

/// Build a Scale from a Temperament, symbols, a reference note name, and frequency.
///
/// Algorithm (matching Java `ScalePage.createScaleButton`):
/// 1. Find `ref_name` in `symbols` to get `ref_index`
/// 2. `multiplier = ref_frequency / temperament.ratio[ref_index]`
/// 3. `frequency[i] = temperament.ratio[i] * multiplier`
///
/// The number of notes equals `temperament.ratios.len()`, and note names
/// are taken from `symbols` starting at an offset such that `ref_name`
/// falls at position `ref_index`.
pub fn scale_from_temperament(
    temperament: &Temperament,
    symbols: &ScaleSymbolList,
    ref_name: &str,
    ref_frequency: f64,
    scale_name: &str,
) -> Result<Scale, String> {
    // Find the reference note's position in the symbol list
    let sym_index = symbols
        .symbols
        .iter()
        .position(|s| s == ref_name)
        .ok_or_else(|| format!("Reference note '{ref_name}' not found in symbol list"))?;

    let n_ratios = temperament.ratios.len();
    if n_ratios == 0 {
        return Err("Temperament has no ratios".to_string());
    }

    // The reference note corresponds to ratio index 0 (unison).
    // Symbol offset: symbols[sym_index] = ratio[0], symbols[sym_index+1] = ratio[1], etc.
    let multiplier = ref_frequency; // ratio[0] is 1.0, so multiplier = ref_frequency / 1.0

    let mut notes = Vec::with_capacity(n_ratios);
    for (i, &ratio) in temperament.ratios.iter().enumerate() {
        let name_idx = sym_index + i;
        let name = if name_idx < symbols.symbols.len() {
            symbols.symbols[name_idx].clone()
        } else {
            format!("Note{i}")
        };
        notes.push(ScaleNote {
            name,
            frequency: ratio * multiplier,
        });
    }

    Ok(Scale {
        name: scale_name.to_string(),
        comment: None,
        notes,
    })
}

/// Combine a Scale and a FingeringPattern into a Tuning.
///
/// For each fingering in the pattern, if the pattern has a note name
/// matching a scale note, sets the frequency from the scale. Fingerings
/// without note names get assigned scale notes in order.
///
/// Java reference: `TuningPage` in the wizard.
pub fn tuning_from_scale_and_pattern(
    scale: &Scale,
    pattern: &super::Tuning,
    tuning_name: &str,
) -> super::Tuning {
    use super::{Fingering, Note};

    let mut fingerings = Vec::with_capacity(pattern.fingerings.len());

    for (i, pf) in pattern.fingerings.iter().enumerate() {
        // If the pattern fingering has a note name, look it up in the scale
        let (name, freq) = if !pf.note.name.is_empty() {
            let scale_freq = scale
                .notes
                .iter()
                .find(|n| n.name == pf.note.name)
                .map(|n| n.frequency);
            (pf.note.name.clone(), scale_freq)
        } else if i < scale.notes.len() {
            // No note name in pattern — assign scale notes in order
            (
                scale.notes[i].name.clone(),
                Some(scale.notes[i].frequency),
            )
        } else {
            (format!("Note {}", i + 1), None)
        };

        fingerings.push(Fingering {
            note: Note {
                name,
                frequency: freq,
                frequency_min: pf.note.frequency_min,
                frequency_max: pf.note.frequency_max,
            },
            open_holes: pf.open_holes.clone(),
            open_end: pf.open_end,
            optimization_weight: pf.optimization_weight,
        });
    }

    super::Tuning {
        name: tuning_name.to_string(),
        comment: Some(format!(
            "Generated from scale '{}' and pattern '{}'",
            scale.name, pattern.name
        )),
        number_of_holes: pattern.number_of_holes,
        fingerings,
    }
}
