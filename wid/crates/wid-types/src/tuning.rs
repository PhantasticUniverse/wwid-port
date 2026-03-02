//! Tuning and fingering XML model.

use serde::{Deserialize, Serialize};

/// A tuning defines a set of fingerings (note + hole-open pattern).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "tuning")]
pub struct Tuning {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(rename = "numberOfHoles")]
    pub number_of_holes: u32,
    #[serde(rename = "fingering", default)]
    pub fingerings: Vec<Fingering>,
}

/// A single fingering: which note is targeted and which holes are open.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Fingering {
    pub note: Note,
    #[serde(rename = "openHole", default)]
    pub open_holes: Vec<bool>,
    #[serde(rename = "openEnd", default, skip_serializing_if = "Option::is_none")]
    pub open_end: Option<bool>,
    #[serde(rename = "optimizationWeight", default, skip_serializing_if = "Option::is_none")]
    pub optimization_weight: Option<i32>,
}

/// A note with optional frequency bounds.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Note {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f64>,
    #[serde(rename = "frequencyMin", default, skip_serializing_if = "Option::is_none")]
    pub frequency_min: Option<f64>,
    #[serde(rename = "frequencyMax", default, skip_serializing_if = "Option::is_none")]
    pub frequency_max: Option<f64>,
}
