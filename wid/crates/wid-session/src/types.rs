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
}

/// Study model kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StudyKind {
    NAF,
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

/// Result of fipple factor calibration.
#[derive(Debug, Clone, Serialize)]
pub struct CalibResult {
    pub initial_fipple_factor: f64,
    pub final_fipple_factor: f64,
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
