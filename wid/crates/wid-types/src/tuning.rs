//! Tuning and fingering XML model.

use serde::Deserialize;

/// A tuning defines a set of fingerings (note + hole-open pattern).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "tuning")]
pub struct Tuning {
    pub name: String,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(rename = "numberOfHoles")]
    pub number_of_holes: u32,
    #[serde(rename = "fingering", default)]
    pub fingerings: Vec<Fingering>,
}

/// A single fingering: which note is targeted and which holes are open.
#[derive(Debug, Clone, Deserialize)]
pub struct Fingering {
    pub note: Note,
    #[serde(rename = "openHole", default)]
    pub open_holes: Vec<bool>,
    #[serde(rename = "openEnd", default)]
    pub open_end: Option<bool>,
    #[serde(rename = "optimizationWeight", default)]
    pub optimization_weight: Option<i32>,
}

/// A note with optional frequency bounds.
#[derive(Debug, Clone, Deserialize)]
pub struct Note {
    pub name: String,
    #[serde(default)]
    pub frequency: Option<f64>,
    #[serde(rename = "frequencyMin", default)]
    pub frequency_min: Option<f64>,
    #[serde(rename = "frequencyMax", default)]
    pub frequency_max: Option<f64>,
}
