//! Session result types and error definitions.

use serde::{Deserialize, Serialize};

/// Unique identifier for a document within a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DocId(pub u32);

/// Kind of document managed by the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocKind {
    Instrument,
    Tuning,
    Constraints,
    Scale,
    Temperament,
    ScaleSymbolList,
    FingeringPattern,
}

/// Study model kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudyKind {
    NAF,
    Whistle,
    Flute,
    Reed,
}

/// Result of opening an XML document.
#[derive(Debug, Clone, Serialize)]
pub struct OpenResult {
    pub doc_id: DocId,
    pub doc_kind: DocKind,
    pub name: String,
}

/// A single evaluation row (one fingering).
#[derive(Debug, Clone, Serialize)]
pub struct EvalRow {
    pub note: String,
    pub target_freq: f64,
    pub predicted_freq: f64,
    pub cents: f64,
    pub weight: i32,
}

/// Result of evaluating tuning.
#[derive(Debug, Clone, Serialize)]
pub struct TuningResult {
    pub rows: Vec<EvalRow>,
    pub net_error: f64,
    pub mean_deviation: f64,
}

/// Optimization progress information.
#[derive(Debug, Clone, Serialize)]
pub struct OptProgress {
    pub evaluations: usize,
    pub best_norm: f64,
}

/// Result of an optimization run.
#[derive(Debug, Clone, Serialize)]
pub struct OptimizeResult {
    pub new_instrument_id: DocId,
    pub initial_norm: f64,
    pub final_norm: f64,
    pub evaluations: usize,
}

/// Result of a calibration run.
///
/// Fields are optional to support different calibration types:
/// - NAF fipple: `fipple_factor` fields set
/// - Whistle window height: `window_height` fields set
/// - Whistle beta: `beta` fields set
/// - Whistle joint: both `window_height` and `beta` set
/// - Flute airstream length: `airstream_length` fields set
/// - Flute joint: both `airstream_length` and `beta` set
/// - Reed: both `alpha` and `beta` set
#[derive(Debug, Clone, Serialize)]
pub struct CalibResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_fipple_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_fipple_factor: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_window_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_window_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_airstream_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_airstream_length: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_alpha: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_alpha: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_beta: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_beta: Option<f64>,
    pub initial_norm: f64,
    pub final_norm: f64,
}

/// Information about an available optimizer.
#[derive(Debug, Clone, Serialize)]
pub struct OptimizerInfo {
    pub key: String,
    pub display_name: String,
    pub objective_function_name: String,
}

/// Current session selection state.
#[derive(Debug, Clone, Default, Serialize)]
pub struct Selection {
    pub instrument_id: Option<DocId>,
    pub tuning_id: Option<DocId>,
    pub optimizer_key: Option<String>,
    pub constraints_id: Option<DocId>,
}

// ── Sketch types ────────────────────────────────────────────────

/// Extracted geometry data for sketching an instrument.
///
/// Contains all the physical dimensions needed to draw a cross-section
/// of the instrument: bore profile, hole locations, and mouthpiece.
/// All dimensions are in the instrument's native length units.
#[derive(Debug, Clone, Serialize)]
pub struct SketchData {
    /// Instrument name from the XML document.
    pub name: String,
    /// Length unit system ("Inches", "Millimetres", etc.).
    pub length_type: String,
    /// Total bore length (position of the last bore point).
    pub bore_length: f64,
    /// Bore profile as position/diameter pairs, sorted by position.
    pub bore_points: Vec<SketchBorePoint>,
    /// Tone holes with position, diameter, and height.
    pub holes: Vec<SketchHole>,
    /// Mouthpiece geometry (type-discriminated: Fipple, Embouchure, or Reed).
    pub mouthpiece: SketchMouthpiece,
    /// Termination flange diameter.
    pub flange_diameter: f64,
}

/// A single bore profile point.
#[derive(Debug, Clone, Serialize)]
pub struct SketchBorePoint {
    /// Distance from the head end of the instrument.
    pub position: f64,
    /// Internal bore diameter at this position.
    pub diameter: f64,
}

/// A single tone hole.
#[derive(Debug, Clone, Serialize)]
pub struct SketchHole {
    /// Optional hole name (e.g., "Thumb", "R1").
    pub name: Option<String>,
    /// Distance from the head end of the instrument.
    pub position: f64,
    /// Hole diameter.
    pub diameter: f64,
    /// Hole chimney height (wall thickness at hole location).
    pub height: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum SketchMouthpiece {
    Fipple {
        position: f64,
        window_length: f64,
        window_width: f64,
        fipple_factor: Option<f64>,
        window_height: Option<f64>,
        windway_height: Option<f64>,
        windway_length: Option<f64>,
    },
    Embouchure {
        position: f64,
        length: f64,
        width: f64,
        height: f64,
        airstream_length: f64,
        airstream_height: f64,
    },
    SingleReed {
        position: f64,
        alpha: f64,
    },
    DoubleReed {
        position: f64,
        alpha: f64,
        crow_freq: f64,
    },
    LipReed {
        position: f64,
        alpha: f64,
    },
}

// ── Compare types ───────────────────────────────────────────────

/// Result of comparing two instruments field by field.
///
/// Only dimensions that differ (above the precision threshold) are included.
/// Java reference: `InstrumentComparisonTable.java`.
#[derive(Debug, Clone, Serialize)]
pub struct CompareResult {
    /// Name of the baseline (old) instrument.
    pub old_name: String,
    /// Name of the modified (new) instrument.
    pub new_name: String,
    /// Rows for dimensions that differ between the two instruments.
    pub rows: Vec<CompareRow>,
}

/// A single dimension comparison row.
///
/// Categories include: "Mouthpiece", "Hole 1", "Hole 2", ...,
/// "Bore Point 1", ..., "Termination".
#[derive(Debug, Clone, Serialize)]
pub struct CompareRow {
    /// Category grouping (e.g., "Mouthpiece", "Hole 3", "Bore Point 2").
    pub category: String,
    /// Dimension name (e.g., "Position", "Diameter", "Window Length").
    pub field: String,
    /// Old instrument value (None if dimension doesn't exist in old).
    pub old_value: Option<f64>,
    /// New instrument value (None if dimension doesn't exist in new).
    pub new_value: Option<f64>,
    /// Absolute difference (new - old).
    pub difference: Option<f64>,
    /// Percent change: `100 * (new - old) / old`.
    pub percent_change: Option<f64>,
}

// ── Supplementary info types ────────────────────────────────────

/// Supplementary acoustic info for the current tuning.
///
/// Java reference: `SupplementaryInfoTable.java`.
#[derive(Debug, Clone, Serialize)]
pub struct SupplementaryResult {
    /// One row per fingering in the tuning.
    pub rows: Vec<SupplementaryRow>,
}

/// Supplementary acoustic data for a single fingering.
///
/// All values are computed at the predicted playing frequency unless
/// otherwise noted.
#[derive(Debug, Clone, Serialize)]
pub struct SupplementaryRow {
    /// Note name from the tuning (e.g., "F#4").
    pub note: String,
    /// Target frequency (Hz) from the tuning.
    pub freq: f64,
    /// Im(Z) correction: `Im(Z(target)) - Im(Z(predicted))`.
    /// Indicates how far the predicted frequency's reactance is from
    /// the target frequency's reactance.
    pub im_z_correction: f64,
    /// Air jet velocity (m/s) from the Strouhal model.
    /// Only available for Whistle (fipple) and Flute (embouchure) instruments.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_speed: Option<f64>,
    /// Volumetric air flow rate (mm²·m/s = mm²/s × 1000).
    /// Computed as velocity × windway_area. Only for fipple/embouchure.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub air_flow_rate: Option<f64>,
    /// Loop gain at the predicted frequency.
    /// G = gain_factor × f × ρ / |Z|. Values > 1.0 indicate the instrument
    /// can sustain oscillation.
    pub gain: f64,
    /// Quality factor from Yaghjian & Best (2005):
    /// `Q = 0.25 × (f + f') × (ratio' - ratio) / (f' - f)`,
    /// where `f' = f × (1 + 0.0012)`.
    pub q_factor: f64,
}

// ── Graph tuning types ──────────────────────────────────────────

/// Playing range curves for all fingerings in a tuning.
///
/// Each curve shows the Im(Z)/Re(Z) impedance ratio across the fingering's
/// playing range. Java reference: `PlotPlayingRanges.java`.
#[derive(Debug, Clone, Serialize)]
pub struct GraphTuningResult {
    /// One curve per fingering in the tuning.
    pub curves: Vec<TuningCurve>,
}

/// Playing range curve for a single fingering.
///
/// Contains 33 frequency-swept points showing how Im(Z)/Re(Z) varies
/// across the playing range, plus the target and predicted frequencies.
#[derive(Debug, Clone, Serialize)]
pub struct TuningCurve {
    /// Note name from the tuning (e.g., "F#4").
    pub note_name: String,
    /// Target frequency (Hz) from the tuning.
    pub target_freq: f64,
    /// Predicted playing frequency (Hz) from the acoustic model.
    pub predicted_freq: f64,
    /// Lower bound of the playing range (fmin). None if no range found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq_min: Option<f64>,
    /// Upper bound of the playing range (fmax = reactance zero). None if not found.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq_max: Option<f64>,
    /// Frequency sweep points: `[frequency_hz, im_z_over_re_z]`.
    pub points: Vec<[f64; 2]>,
}

/// Impedance and gain spectrum for a single fingering.
///
/// Contains 2000 points spanning [0.45×target, 3.17×target] frequency range.
/// Java reference: `PlayingRangeSpectrum.java`.
#[derive(Debug, Clone, Serialize)]
pub struct NoteSpectrumResult {
    /// Note name from the tuning (e.g., "F#4").
    pub note_name: String,
    /// Target frequency (Hz) from the tuning.
    pub target_freq: f64,
    /// Frequency-swept spectrum points.
    pub points: Vec<SpectrumPoint>,
}

/// A single point in the note spectrum.
#[derive(Debug, Clone, Serialize)]
pub struct SpectrumPoint {
    /// Frequency (Hz).
    pub freq: f64,
    /// Impedance ratio: Im(Z)/Re(Z) at this frequency.
    pub impedance_ratio: f64,
    /// Loop gain: G = gain_factor × f × ρ / |Z|.
    /// Values > 1.0 indicate the instrument can sustain oscillation.
    pub loop_gain: f64,
}

/// Session errors.
#[derive(Debug, Clone)]
pub enum SessionError {
    InvalidXml(String),
    UnknownDocKind(String),
    MissingSelection(&'static str),
    HoleCountMismatch { instrument: u32, tuning: u32 },
    DocNotFound(DocId),
    OptimizerNotFound(String),
    CannotTune(String),
    CannotOptimize(String),
    CompileError(String),
    EvalError(String),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::InvalidXml(e) => write!(f, "Invalid XML: {e}"),
            SessionError::UnknownDocKind(e) => write!(f, "Unknown document kind: {e}"),
            SessionError::MissingSelection(what) => write!(f, "No {what} selected"),
            SessionError::HoleCountMismatch { instrument, tuning } => {
                write!(f, "Hole count mismatch: instrument has {instrument}, tuning has {tuning}")
            }
            SessionError::DocNotFound(id) => write!(f, "Document not found: {:?}", id),
            SessionError::OptimizerNotFound(key) => write!(f, "Optimizer not found: {key}"),
            SessionError::CannotTune(reason) => write!(f, "Cannot tune: {reason}"),
            SessionError::CannotOptimize(reason) => write!(f, "Cannot optimize: {reason}"),
            SessionError::CompileError(e) => write!(f, "Compile error: {e}"),
            SessionError::EvalError(e) => write!(f, "Evaluation error: {e}"),
        }
    }
}

impl std::error::Error for SessionError {}
