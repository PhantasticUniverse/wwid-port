//! Instrument XML model.

use serde::{Deserialize, Serialize};

/// Top-level instrument, as deserialized from WIDesigner XML.
///
/// All dimensional values are in the units specified by `length_type`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename = "instrument")]
pub struct InstrumentRaw {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "lengthType")]
    pub length_type: LengthType,
    pub mouthpiece: MouthpieceRaw,
    #[serde(rename = "borePoint", default)]
    pub bore_points: Vec<BorePointRaw>,
    #[serde(rename = "hole", default)]
    pub holes: Vec<HoleRaw>,
    pub termination: TerminationRaw,
}

/// Unit system for dimensional values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
pub enum LengthType {
    #[serde(rename = "in")]
    Inches,
    #[serde(rename = "cm")]
    Centimeters,
    #[serde(rename = "mm")]
    Millimeters,
    #[serde(rename = "m")]
    Metres,
    #[serde(rename = "ft")]
    Feet,
}

impl LengthType {
    /// Multiplier to convert from this unit to metres.
    pub fn to_metres(self) -> f64 {
        match self {
            LengthType::Inches => 0.0254,
            LengthType::Centimeters => 0.01,
            LengthType::Millimeters => 0.001,
            LengthType::Metres => 1.0,
            LengthType::Feet => 0.3048,
        }
    }
}

/// Mouthpiece definition. Contains exactly one of the mouthpiece type variants.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MouthpieceRaw {
    pub position: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub beta: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fipple: Option<FippleRaw>,
    #[serde(rename = "embouchureHole", default, skip_serializing_if = "Option::is_none")]
    pub embouchure_hole: Option<EmbouchureHoleRaw>,
    #[serde(rename = "singleReed", default, skip_serializing_if = "Option::is_none")]
    pub single_reed: Option<SingleReedRaw>,
    #[serde(rename = "doubleReed", default, skip_serializing_if = "Option::is_none")]
    pub double_reed: Option<DoubleReedRaw>,
    #[serde(rename = "lipReed", default, skip_serializing_if = "Option::is_none")]
    pub lip_reed: Option<LipReedRaw>,
}

/// Fipple (edge-blown) mouthpiece, used by NAF and recorders.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FippleRaw {
    #[serde(rename = "windowLength")]
    pub window_length: f64,
    #[serde(rename = "windowWidth")]
    pub window_width: f64,
    #[serde(rename = "fippleFactor", default, skip_serializing_if = "Option::is_none")]
    pub fipple_factor: Option<f64>,
    #[serde(rename = "windowHeight", default, skip_serializing_if = "Option::is_none")]
    pub window_height: Option<f64>,
    #[serde(rename = "windwayLength", default, skip_serializing_if = "Option::is_none")]
    pub windway_length: Option<f64>,
    #[serde(rename = "windwayHeight", default, skip_serializing_if = "Option::is_none")]
    pub windway_height: Option<f64>,
}

/// Embouchure hole mouthpiece, used by transverse flutes.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbouchureHoleRaw {
    pub length: f64,
    pub width: f64,
    pub height: f64,
    #[serde(rename = "airstreamLength")]
    pub airstream_length: f64,
    #[serde(rename = "airstreamHeight")]
    pub airstream_height: f64,
}

/// Single reed mouthpiece.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SingleReedRaw {
    pub alpha: f64,
}

/// Double reed mouthpiece.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DoubleReedRaw {
    pub alpha: f64,
    #[serde(rename = "crowFreq")]
    pub crow_freq: f64,
}

/// Lip reed (brass) mouthpiece.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LipReedRaw {
    pub alpha: f64,
}

/// A point on the bore profile.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BorePointRaw {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "borePosition")]
    pub bore_position: f64,
    #[serde(rename = "boreDiameter")]
    pub bore_diameter: f64,
}

/// A tonehole in the bore wall.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HoleRaw {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(rename = "borePosition")]
    pub bore_position: f64,
    pub diameter: f64,
    pub height: f64,
    #[serde(rename = "innerCurvatureRadius", default, skip_serializing_if = "Option::is_none")]
    pub inner_curvature_radius: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<KeyRaw>,
}

/// Key mechanism covering a tonehole.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KeyRaw {
    pub diameter: f64,
    #[serde(rename = "holeDiameter")]
    pub hole_diameter: f64,
    pub height: f64,
    pub thickness: f64,
    #[serde(rename = "wallThickness")]
    pub wall_thickness: f64,
    #[serde(rename = "chimneyHeight")]
    pub chimney_height: f64,
}

/// End termination of the bore.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TerminationRaw {
    #[serde(rename = "flangeDiameter")]
    pub flange_diameter: f64,
}
