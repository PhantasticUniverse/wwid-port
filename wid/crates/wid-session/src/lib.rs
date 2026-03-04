//! Session API for the WIDesigner port.
//!
//! [`StudySession`] is the central orchestrator that mirrors the Java
//! `NafStudyModel` behavior. It owns all state (documents, selection,
//! physical parameters) and exposes a command/query API used by the
//! WASM bindings, tests, and optional CLI.
//!
//! # Selection-driven design
//!
//! Operations are gated by the current selection:
//! - [`can_tune`](StudySession::can_tune) — instrument + tuning + matching hole counts
//! - [`can_optimize`](StudySession::can_optimize) — can_tune + optimizer + constraints
//! - [`can_sketch`](StudySession::can_sketch) — only needs an instrument
//!
//! # Example
//!
//! ```ignore
//! use wid_session::{StudySession, StudyKind};
//!
//! let mut session = StudySession::new(StudyKind::NAF);
//! let inst = session.open_xml(instrument_xml).unwrap();
//! let tuning = session.open_xml(tuning_xml).unwrap();
//! session.select_instrument(inst.doc_id);
//! session.select_tuning(tuning.doc_id);
//! let result = session.evaluate_tuning().unwrap();
//! ```

pub mod doc_store;
pub mod flute;
pub mod naf;
pub mod reed;
pub mod types;
pub mod whistle;

use bobyqa::BobyqaProgress;
use doc_store::{DocContent, DocStore};
use wid_compile::{compile, get_fipple_factor};
use wid_eval::{CalculatorParams, LinearVTuner, cents, predicted_frequency};
use wid_eval::calculator_params::MouthpieceModel;
use wid_optimize::fingering_weights;
pub use wid_physics::{PhysicalParameters, TemperatureType};
use wid_types::{
    Constraints, InstrumentRaw, Tuning,
    parse_constraints_xml, parse_instrument_xml, parse_tuning_xml,
};

// Re-export key types for convenience.
pub use types::{
    CalibResult, DocId, DocKind, EvalRow, OpenResult, OptProgress,
    OptimizeResult, OptimizerInfo, Selection, SessionError, StudyKind,
    TuningResult,
};

/// The central session orchestrator.
///
/// Mirrors the Java `NafStudyModel` — owns documents, selection state,
/// and physical parameters. All operations delegate to the appropriate
/// crate (wid-eval, wid-optimize, etc.).
pub struct StudySession {
    pub study_kind: StudyKind,
    calc_params: CalculatorParams,
    docs: DocStore,
    selection: Selection,
    params: PhysicalParameters,
    next_untitled: u32,
}

impl StudySession {
    /// Create a new session for the given study kind.
    ///
    /// Defaults to 72°F, matching the Java core's `PhysicalParameters(72.0, F)`
    /// and all golden fixture data. Note: the Java GUI's preferences system
    /// overrides this to 20°C (`OptimizationPreferences.DEFAULT_TEMPERATURE`),
    /// so users of the Java app see 20°C. We'll add a similar override when
    /// we implement the settings/preferences UI.
    pub fn new(study_kind: StudyKind) -> Self {
        let calc_params = match study_kind {
            StudyKind::NAF => CalculatorParams::NAF,
            StudyKind::Whistle => CalculatorParams::WHISTLE,
            StudyKind::Flute => CalculatorParams::FLUTE,
            StudyKind::Reed => CalculatorParams::REED,
        };
        StudySession {
            study_kind,
            calc_params,
            docs: DocStore::new(),
            selection: Selection::default(),
            params: PhysicalParameters::new(72.0, TemperatureType::F),
            next_untitled: 1,
        }
    }

    /// Get the current physical parameters.
    pub fn params(&self) -> &PhysicalParameters {
        &self.params
    }

    /// Override the physical parameters (e.g., to change temperature).
    pub fn set_params(&mut self, params: PhysicalParameters) {
        self.params = params;
    }

    /// Get the current selection state.
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    // ── Document I/O ────────────────────────────────────────────────

    /// Open an XML document, auto-detecting its kind.
    ///
    /// Tries to parse as instrument, tuning, then constraints.
    /// Returns the assigned document ID and detected kind.
    pub fn open_xml(&mut self, xml: &str) -> Result<OpenResult, SessionError> {
        // Try instrument first
        if let Ok(inst) = parse_instrument_xml(xml) {
            let name = inst.name.clone();
            let id = self.docs.insert(
                DocKind::Instrument,
                name.clone(),
                DocContent::Instrument(inst),
            );
            return Ok(OpenResult {
                doc_id: id,
                doc_kind: DocKind::Instrument,
                name,
            });
        }

        // Try tuning
        if let Ok(tuning) = parse_tuning_xml(xml) {
            let name = tuning.name.clone();
            let id = self.docs.insert(
                DocKind::Tuning,
                name.clone(),
                DocContent::Tuning(tuning),
            );
            return Ok(OpenResult {
                doc_id: id,
                doc_kind: DocKind::Tuning,
                name,
            });
        }

        // Try constraints
        if let Ok(constraints) = parse_constraints_xml(xml) {
            let name = constraints.name.clone();
            let id = self.docs.insert(
                DocKind::Constraints,
                name.clone(),
                DocContent::Constraints(constraints),
            );
            return Ok(OpenResult {
                doc_id: id,
                doc_kind: DocKind::Constraints,
                name,
            });
        }

        Err(SessionError::InvalidXml(
            "Could not parse as instrument, tuning, or constraints".to_string(),
        ))
    }

    /// Serialize a document back to WIDesigner-compatible XML.
    pub fn export_xml(&self, doc_id: DocId) -> Result<String, SessionError> {
        let doc = self.docs.get(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))?;

        match &doc.content {
            DocContent::Instrument(inst) => {
                serialize_instrument_xml(inst)
                    .map_err(|e| SessionError::InvalidXml(e.to_string()))
            }
            DocContent::Tuning(tuning) => {
                serialize_tuning_xml(tuning)
                    .map_err(|e| SessionError::InvalidXml(e.to_string()))
            }
            DocContent::Constraints(constraints) => {
                serialize_constraints_xml(constraints)
                    .map_err(|e| SessionError::InvalidXml(e.to_string()))
            }
        }
    }

    // ── Selection ───────────────────────────────────────────────────

    /// Select an instrument document.
    pub fn select_instrument(&mut self, doc_id: DocId) {
        self.selection.instrument_id = Some(doc_id);
    }

    /// Select a tuning document.
    pub fn select_tuning(&mut self, doc_id: DocId) {
        self.selection.tuning_id = Some(doc_id);
    }

    /// Select an optimizer by key.
    pub fn select_optimizer(&mut self, key: &str) {
        self.selection.optimizer_key = Some(key.to_string());
    }

    /// Select a constraints document.
    pub fn select_constraints(&mut self, doc_id: DocId) {
        self.selection.constraints_id = Some(doc_id);
    }

    /// Clear all selections.
    pub fn clear_selection(&mut self) {
        self.selection = Selection::default();
    }

    // ── Gating predicates ───────────────────────────────────────────

    /// Check if tuning evaluation is possible.
    ///
    /// Requires: instrument + tuning selected, hole counts match.
    pub fn can_tune(&self) -> bool {
        let Some(inst_id) = self.selection.instrument_id else {
            return false;
        };
        let Some(tun_id) = self.selection.tuning_id else {
            return false;
        };
        let Some(inst) = self.docs.get_instrument(inst_id) else {
            return false;
        };
        let Some(tuning) = self.docs.get_tuning(tun_id) else {
            return false;
        };
        inst.holes.len() as u32 == tuning.number_of_holes
    }

    /// Check if optimization is possible.
    ///
    /// Requires: can_tune + optimizer selected + constraints selected.
    pub fn can_optimize(&self) -> bool {
        if !self.can_tune() {
            return false;
        }
        let Some(ref key) = self.selection.optimizer_key else {
            return false;
        };
        match self.study_kind {
            StudyKind::NAF => {
                if !naf::is_valid_optimizer(key) {
                    return false;
                }
                // Fipple calibration doesn't require constraints
                if naf::is_fipple_optimizer(key) {
                    return true;
                }
                self.selection.constraints_id.is_some()
            }
            StudyKind::Whistle => {
                if !whistle::is_valid_optimizer(key) {
                    return false;
                }
                // Calibrators don't require constraints
                if whistle::is_calibrator(key) {
                    return true;
                }
                self.selection.constraints_id.is_some()
            }
            StudyKind::Flute => {
                if !flute::is_valid_optimizer(key) {
                    return false;
                }
                if flute::is_calibrator(key) {
                    return true;
                }
                self.selection.constraints_id.is_some()
            }
            StudyKind::Reed => {
                if !reed::is_valid_optimizer(key) {
                    return false;
                }
                if reed::is_calibrator(key) {
                    return true;
                }
                self.selection.constraints_id.is_some()
            }
        }
    }

    /// Check if instrument sketching is possible (only needs instrument).
    pub fn can_sketch(&self) -> bool {
        self.selection
            .instrument_id
            .and_then(|id| self.docs.get_instrument(id))
            .is_some()
    }

    // ── Available optimizers ────────────────────────────────────────

    /// Returns the list of available optimizers for the current study model.
    pub fn available_optimizers(&self) -> Vec<OptimizerInfo> {
        match self.study_kind {
            StudyKind::NAF => naf::available_optimizers(),
            StudyKind::Whistle => whistle::available_optimizers(),
            StudyKind::Flute => flute::available_optimizers(),
            StudyKind::Reed => reed::available_optimizers(),
        }
    }

    // ── Evaluate tuning ─────────────────────────────────────────────

    /// Evaluate the current instrument against the current tuning.
    ///
    /// Returns per-fingering results including predicted frequency and
    /// cents deviation, plus summary statistics (Net Error and Mean Deviation).
    pub fn evaluate_tuning(&self) -> Result<TuningResult, SessionError> {
        let inst_id = self.selection.instrument_id
            .ok_or(SessionError::MissingSelection("instrument"))?;
        let tun_id = self.selection.tuning_id
            .ok_or(SessionError::MissingSelection("tuning"))?;

        let inst = self.docs.get_instrument(inst_id)
            .ok_or(SessionError::DocNotFound(inst_id))?;
        let tuning = self.docs.get_tuning(tun_id)
            .ok_or(SessionError::DocNotFound(tun_id))?;

        if inst.holes.len() as u32 != tuning.number_of_holes {
            return Err(SessionError::HoleCountMismatch {
                instrument: inst.holes.len() as u32,
                tuning: tuning.number_of_holes,
            });
        }

        let compiled = compile(inst)
            .map_err(|e| SessionError::CompileError(e.to_string()))?;

        let weights = fingering_weights(&tuning.fingerings);

        // Pre-build LinearV tuner for Whistle instruments
        let linear_v_tuner = match self.calc_params.mouthpiece_model {
            MouthpieceModel::SimpleFipple => Some(LinearVTuner::new(
                &compiled,
                &tuning.fingerings,
                &self.params,
                &self.calc_params,
                self.calc_params.blowing_level,
            )),
            _ => None,
        };

        let mut rows = Vec::with_capacity(tuning.fingerings.len());
        for (i, fingering) in tuning.fingerings.iter().enumerate() {
            let target_freq = fingering.note.frequency.unwrap_or(0.0);
            let weight = weights[i];

            let pred = match &linear_v_tuner {
                Some(tuner) => wid_eval::linear_v::predicted_frequency_linear_v(
                    tuner, &compiled, fingering, &self.params, &self.calc_params,
                ),
                None => predicted_frequency(&compiled, fingering, &self.params, &self.calc_params),
            };

            let (predicted_freq, cent_dev) = if let Some(pred) = pred {
                (pred, cents(target_freq, pred))
            } else {
                (0.0, 1200.0)
            };

            rows.push(EvalRow {
                note: fingering.note.name.clone(),
                target_freq,
                predicted_freq,
                cents: cent_dev,
                weight,
            });
        }

        // Summary: net error = signed mean of weighted cents,
        // deviation = mean absolute cents (weighted)
        let weighted_rows: Vec<&EvalRow> = rows.iter().filter(|r| r.weight > 0).collect();
        let n_weighted = weighted_rows.len() as f64;
        let net_error = if n_weighted > 0.0 {
            weighted_rows.iter().map(|r| r.cents).sum::<f64>() / n_weighted
        } else {
            0.0
        };
        let mean_deviation = if n_weighted > 0.0 {
            weighted_rows.iter().map(|r| r.cents.abs()).sum::<f64>() / n_weighted
        } else {
            0.0
        };

        Ok(TuningResult {
            rows,
            net_error,
            mean_deviation,
        })
    }

    // ── Calibrate fipple factor ─────────────────────────────────────

    /// Calibrate mouthpiece parameters using the selected instrument and tuning.
    ///
    /// Modifies the instrument in place.
    /// - NAF: calibrates fipple factor (lowest fingering only)
    /// - Whistle: calibrates window height, beta, or both (all fingerings)
    pub fn calibrate(&mut self) -> Result<CalibResult, SessionError> {
        let inst_id = self.selection.instrument_id
            .ok_or(SessionError::MissingSelection("instrument"))?;
        let tun_id = self.selection.tuning_id
            .ok_or(SessionError::MissingSelection("tuning"))?;
        let optimizer_key = self.selection.optimizer_key.clone()
            .ok_or(SessionError::MissingSelection("optimizer"))?;

        // Clone tuning first (before mutable borrow of instrument)
        let tuning = self.docs.get_tuning(tun_id)
            .ok_or(SessionError::DocNotFound(tun_id))?
            .clone();

        // Get bounds from constraints if available
        let constraint_bounds = if let Some(c_id) = self.selection.constraints_id {
            self.docs.get_constraints(c_id).map(|c| {
                (c.lower_bounds(), c.upper_bounds())
            })
        } else {
            None
        };

        let inst = self.docs.get_instrument_mut(inst_id)
            .ok_or(SessionError::DocNotFound(inst_id))?;

        match self.study_kind {
            StudyKind::NAF => {
                let (lower, upper) = match &constraint_bounds {
                    Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                    _ => (wid_optimize::fipple::DEFAULT_FF_LOWER, wid_optimize::fipple::DEFAULT_FF_UPPER),
                };

                let result = wid_optimize::fipple::calibrate_fipple(
                    inst, &tuning, &self.params, lower, upper, &self.calc_params,
                );

                Ok(CalibResult {
                    initial_fipple_factor: Some(result.initial_fipple_factor),
                    final_fipple_factor: Some(result.final_fipple_factor),
                    initial_window_height: None,
                    final_window_height: None,
                    initial_airstream_length: None,
                    final_airstream_length: None,
                    initial_alpha: None,
                    final_alpha: None,
                    initial_beta: None,
                    final_beta: None,
                    initial_norm: result.initial_norm,
                    final_norm: result.final_norm,
                })
            }
            StudyKind::Whistle => {
                calibrate_whistle_impl(inst, &tuning, &self.params, &self.calc_params, &optimizer_key, &constraint_bounds)
            }
            StudyKind::Flute => {
                calibrate_flute_impl(inst, &tuning, &self.params, &self.calc_params, &optimizer_key, &constraint_bounds)
            }
            StudyKind::Reed => {
                calibrate_reed_impl(inst, &tuning, &self.params, &self.calc_params, &constraint_bounds)
            }
        }
    }

    // ── Optimize holes ──────────────────────────────────────────────

    /// Run hole optimization with progress callback.
    ///
    /// Creates a new instrument document with the optimized geometry.
    /// The original instrument is not modified.
    pub fn optimize(
        &mut self,
        on_progress: &mut dyn FnMut(OptProgress) -> bool,
    ) -> Result<OptimizeResult, SessionError> {
        let inst_id = self.selection.instrument_id
            .ok_or(SessionError::MissingSelection("instrument"))?;
        let tun_id = self.selection.tuning_id
            .ok_or(SessionError::MissingSelection("tuning"))?;
        let optimizer_key = self.selection.optimizer_key.clone()
            .ok_or(SessionError::MissingSelection("optimizer"))?;
        let constraints_id = self.selection.constraints_id
            .ok_or(SessionError::MissingSelection("constraints"))?;

        // Calibrators should use calibrate(), not optimize()
        let is_calibrator = match self.study_kind {
            StudyKind::NAF => naf::is_fipple_optimizer(&optimizer_key),
            StudyKind::Whistle => whistle::is_calibrator(&optimizer_key),
            StudyKind::Flute => flute::is_calibrator(&optimizer_key),
            StudyKind::Reed => reed::is_calibrator(&optimizer_key),
        };
        if is_calibrator {
            return Err(SessionError::CannotOptimize(
                "Use calibrate() for calibration-type optimizers".to_string(),
            ));
        }

        let inst = self.docs.get_instrument(inst_id)
            .ok_or(SessionError::DocNotFound(inst_id))?;
        let tuning = self.docs.get_tuning(tun_id)
            .ok_or(SessionError::DocNotFound(tun_id))?;
        let constraints = self.docs.get_constraints(constraints_id)
            .ok_or(SessionError::DocNotFound(constraints_id))?;

        // Clone instrument for optimization (non-destructive)
        let mut work_inst = inst.clone();
        let tuning = tuning.clone();
        let constraints = constraints.clone();

        // Map BobyqaProgress to OptProgress
        let mut progress_adapter = |bp: BobyqaProgress| -> bool {
            on_progress(OptProgress {
                evaluations: bp.evaluations,
                best_norm: bp.best_value,
            })
        };

        let result = match self.study_kind {
            StudyKind::NAF => {
                wid_optimize::hole_from_top::optimize_holes_with_progress(
                    &mut work_inst, &tuning, &constraints, &self.params,
                    &self.calc_params, &mut progress_adapter,
                )
            }
            StudyKind::Whistle => {
                match optimizer_key.as_str() {
                    whistle::HOLE_SIZE => {
                        wid_optimize::hole_size::optimize_hole_size_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    whistle::HOLE_POSITION => {
                        wid_optimize::hole_position::optimize_hole_position_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    whistle::HOLE => {
                        wid_optimize::hole_combined::optimize_holes_combined_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    _ => return Err(SessionError::CannotOptimize(
                        format!("Unknown Whistle optimizer: {}", optimizer_key),
                    )),
                }
            }
            StudyKind::Flute => {
                match optimizer_key.as_str() {
                    flute::HOLE_SIZE => {
                        wid_optimize::hole_size::optimize_hole_size_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    flute::HOLE_POSITION => {
                        wid_optimize::hole_position::optimize_hole_position_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    flute::HOLE => {
                        wid_optimize::hole_combined::optimize_holes_combined_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    _ => return Err(SessionError::CannotOptimize(
                        format!("Unknown Flute optimizer: {}", optimizer_key),
                    )),
                }
            }
            StudyKind::Reed => {
                match optimizer_key.as_str() {
                    reed::HOLE_SIZE => {
                        wid_optimize::hole_size::optimize_hole_size_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    reed::HOLE_POSITION => {
                        wid_optimize::hole_position::optimize_hole_position_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    reed::HOLE => {
                        wid_optimize::hole_combined::optimize_holes_combined_with_progress(
                            &mut work_inst, &tuning, &constraints, &self.params,
                            &self.calc_params, &mut progress_adapter,
                        )
                    }
                    _ => return Err(SessionError::CannotOptimize(
                        format!("Unknown Reed optimizer: {}", optimizer_key),
                    )),
                }
            }
        };

        // Store optimized instrument as a new document
        let name = format!("Untitled {}", self.next_untitled);
        self.next_untitled += 1;
        let new_id = self.docs.insert(
            DocKind::Instrument,
            name,
            DocContent::Instrument(work_inst),
        );

        // Auto-select the new instrument
        self.selection.instrument_id = Some(new_id);

        Ok(OptimizeResult {
            new_instrument_id: new_id,
            initial_norm: result.initial_norm,
            final_norm: result.final_norm,
            evaluations: result.evaluations,
        })
    }

    /// Run optimization without progress callback (for tests).
    pub fn optimize_sync(&mut self) -> Result<OptimizeResult, SessionError> {
        self.optimize(&mut |_| true)
    }

    // ── Constraint generation ───────────────────────────────────────

    /// Create default constraints for the current optimizer and instrument.
    pub fn create_default_constraints(
        &mut self,
        optimizer_key: &str,
    ) -> Result<OpenResult, SessionError> {
        let n_holes = self.instrument_hole_count()?;
        let constraints = match self.study_kind {
            StudyKind::NAF => naf::create_default_constraints(optimizer_key, n_holes),
            StudyKind::Whistle => whistle::create_default_constraints(optimizer_key, n_holes),
            StudyKind::Flute => flute::create_default_constraints(optimizer_key, n_holes),
            StudyKind::Reed => reed::create_default_constraints(optimizer_key, n_holes),
        };
        let name = constraints.name.clone();
        let id = self.docs.insert(
            DocKind::Constraints,
            name.clone(),
            DocContent::Constraints(constraints),
        );
        Ok(OpenResult {
            doc_id: id,
            doc_kind: DocKind::Constraints,
            name,
        })
    }

    /// Create blank constraints for the current optimizer and instrument.
    pub fn create_blank_constraints(
        &mut self,
        optimizer_key: &str,
    ) -> Result<OpenResult, SessionError> {
        let n_holes = self.instrument_hole_count()?;
        let constraints = match self.study_kind {
            StudyKind::NAF => naf::create_blank_constraints(optimizer_key, n_holes),
            StudyKind::Whistle => whistle::create_blank_constraints(optimizer_key, n_holes),
            StudyKind::Flute => flute::create_blank_constraints(optimizer_key, n_holes),
            StudyKind::Reed => reed::create_blank_constraints(optimizer_key, n_holes),
        };
        let name = constraints.name.clone();
        let id = self.docs.insert(
            DocKind::Constraints,
            name.clone(),
            DocContent::Constraints(constraints),
        );
        Ok(OpenResult {
            doc_id: id,
            doc_kind: DocKind::Constraints,
            name,
        })
    }

    // ── Instrument mutation ─────────────────────────────────────────

    /// Delete all holes from the selected instrument (for fipple calibration workflow).
    pub fn delete_instrument_holes(&mut self, doc_id: DocId) -> Result<(), SessionError> {
        let inst = self.docs.get_instrument_mut(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))?;
        inst.holes.clear();
        Ok(())
    }

    /// Get the fipple factor from the selected instrument.
    pub fn get_fipple_factor(&self, doc_id: DocId) -> Result<Option<f64>, SessionError> {
        let inst = self.docs.get_instrument(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))?;
        Ok(get_fipple_factor(inst))
    }

    // ── Document get/set ────────────────────────────────────────────

    /// Get the instrument data for a given doc ID.
    pub fn get_instrument(&self, doc_id: DocId) -> Result<&InstrumentRaw, SessionError> {
        self.docs
            .get_instrument(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    /// Get the tuning data for a given doc ID.
    pub fn get_tuning(&self, doc_id: DocId) -> Result<&Tuning, SessionError> {
        self.docs
            .get_tuning(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    /// Get the constraints data for a given doc ID.
    pub fn get_constraints(&self, doc_id: DocId) -> Result<&Constraints, SessionError> {
        self.docs
            .get_constraints(doc_id)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    /// Replace the instrument content for a given doc ID.
    pub fn set_instrument(
        &mut self,
        doc_id: DocId,
        inst: InstrumentRaw,
    ) -> Result<(), SessionError> {
        self.docs
            .replace_instrument(doc_id, inst)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    /// Replace the tuning content for a given doc ID.
    pub fn set_tuning(&mut self, doc_id: DocId, tuning: Tuning) -> Result<(), SessionError> {
        self.docs
            .replace_tuning(doc_id, tuning)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    /// Replace the constraints content for a given doc ID.
    pub fn set_constraints(
        &mut self,
        doc_id: DocId,
        constraints: Constraints,
    ) -> Result<(), SessionError> {
        self.docs
            .replace_constraints(doc_id, constraints)
            .ok_or(SessionError::DocNotFound(doc_id))
    }

    // ── Document access ─────────────────────────────────────────────

    /// Get the document store (for tests and inspection).
    pub fn docs(&self) -> &DocStore {
        &self.docs
    }

    /// List documents of a given kind.
    pub fn list_docs(&self, kind: DocKind) -> Vec<(DocId, String)> {
        self.docs
            .list(kind)
            .iter()
            .map(|d| (d.id, d.name.clone()))
            .collect()
    }

    // ── Private helpers ─────────────────────────────────────────────

    fn instrument_hole_count(&self) -> Result<u32, SessionError> {
        let inst_id = self.selection.instrument_id
            .ok_or(SessionError::MissingSelection("instrument"))?;
        let inst = self.docs.get_instrument(inst_id)
            .ok_or(SessionError::DocNotFound(inst_id))?;
        Ok(inst.holes.len() as u32)
    }
}

// ── Whistle calibration dispatch ────────────────────────────────────

fn calibrate_whistle_impl(
    inst: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    optimizer_key: &str,
    constraint_bounds: &Option<(Vec<f64>, Vec<f64>)>,
) -> Result<CalibResult, SessionError> {
    match optimizer_key {
        whistle::WINDOW_HEIGHT => {
            let (lower, upper) = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::window_height::DEFAULT_WH_LOWER, wid_optimize::window_height::DEFAULT_WH_UPPER),
            };
            let result = wid_optimize::window_height::calibrate_window_height(
                inst, tuning, params, lower, upper, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: result.initial_window_height,
                final_window_height: result.final_window_height,
                initial_airstream_length: None, final_airstream_length: None,
                initial_alpha: None, final_alpha: None,
                initial_beta: None, final_beta: None,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        whistle::BETA => {
            let (lower, upper) = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::beta::DEFAULT_BETA_LOWER, wid_optimize::beta::DEFAULT_BETA_UPPER),
            };
            let result = wid_optimize::beta::calibrate_beta(
                inst, tuning, params, lower, upper, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: None, final_window_height: None,
                initial_airstream_length: None, final_airstream_length: None,
                initial_alpha: None, final_alpha: None,
                initial_beta: result.initial_beta, final_beta: result.final_beta,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        whistle::WHISTLE_CALIB => {
            let wh_bounds = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::window_height::DEFAULT_WH_LOWER, wid_optimize::window_height::DEFAULT_WH_UPPER),
            };
            let beta_bounds = match constraint_bounds {
                Some((lb, ub)) if lb.len() > 1 && lb[1] > 0.0 => (lb[1], ub[1]),
                _ => (wid_optimize::beta::DEFAULT_BETA_LOWER, wid_optimize::beta::DEFAULT_BETA_UPPER),
            };
            let result = wid_optimize::whistle_calib::calibrate_whistle(
                inst, tuning, params, wh_bounds, beta_bounds, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: result.initial_window_height,
                final_window_height: result.final_window_height,
                initial_airstream_length: None, final_airstream_length: None,
                initial_alpha: None, final_alpha: None,
                initial_beta: result.initial_beta, final_beta: result.final_beta,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        _ => Err(SessionError::CannotOptimize(
            format!("Unknown Whistle calibrator: {}", optimizer_key),
        )),
    }
}

// ── Flute calibration dispatch ─────────────────────────────────────

fn calibrate_flute_impl(
    inst: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    optimizer_key: &str,
    constraint_bounds: &Option<(Vec<f64>, Vec<f64>)>,
) -> Result<CalibResult, SessionError> {
    match optimizer_key {
        flute::AIRSTREAM_LENGTH => {
            let (lower, upper) = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::airstream_length::DEFAULT_AL_LOWER, wid_optimize::airstream_length::DEFAULT_AL_UPPER),
            };
            let result = wid_optimize::airstream_length::calibrate_airstream_length(
                inst, tuning, params, lower, upper, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: None, final_window_height: None,
                initial_airstream_length: result.initial_airstream_length,
                final_airstream_length: result.final_airstream_length,
                initial_alpha: None, final_alpha: None,
                initial_beta: None, final_beta: None,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        flute::BETA => {
            let (lower, upper) = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::beta::DEFAULT_BETA_LOWER, wid_optimize::beta::DEFAULT_BETA_UPPER),
            };
            let result = wid_optimize::beta::calibrate_beta(
                inst, tuning, params, lower, upper, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: None, final_window_height: None,
                initial_airstream_length: None, final_airstream_length: None,
                initial_alpha: None, final_alpha: None,
                initial_beta: result.initial_beta, final_beta: result.final_beta,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        flute::FLUTE_CALIB => {
            let al_bounds = match constraint_bounds {
                Some((lb, ub)) if !lb.is_empty() && lb[0] > 0.0 => (lb[0], ub[0]),
                _ => (wid_optimize::airstream_length::DEFAULT_AL_LOWER, wid_optimize::airstream_length::DEFAULT_AL_UPPER),
            };
            let beta_bounds = match constraint_bounds {
                Some((lb, ub)) if lb.len() > 1 && lb[1] > 0.0 => (lb[1], ub[1]),
                _ => (wid_optimize::beta::DEFAULT_BETA_LOWER, wid_optimize::beta::DEFAULT_BETA_UPPER),
            };
            let result = wid_optimize::flute_calib::calibrate_flute(
                inst, tuning, params, al_bounds, beta_bounds, calc_params,
            );
            Ok(CalibResult {
                initial_fipple_factor: None, final_fipple_factor: None,
                initial_window_height: None, final_window_height: None,
                initial_airstream_length: result.initial_airstream_length,
                final_airstream_length: result.final_airstream_length,
                initial_alpha: None, final_alpha: None,
                initial_beta: result.initial_beta, final_beta: result.final_beta,
                initial_norm: result.initial_norm, final_norm: result.final_norm,
            })
        }
        _ => Err(SessionError::CannotOptimize(
            format!("Unknown Flute calibrator: {}", optimizer_key),
        )),
    }
}

// ── Reed calibration dispatch ──────────────────────────────────────

fn calibrate_reed_impl(
    inst: &mut InstrumentRaw,
    tuning: &Tuning,
    params: &PhysicalParameters,
    calc_params: &CalculatorParams,
    constraint_bounds: &Option<(Vec<f64>, Vec<f64>)>,
) -> Result<CalibResult, SessionError> {
    let alpha_bounds = match constraint_bounds {
        Some((lb, ub)) if !lb.is_empty() => (lb[0], ub[0]),
        _ => (wid_optimize::reed_calib::DEFAULT_ALPHA_LOWER, wid_optimize::reed_calib::DEFAULT_ALPHA_UPPER),
    };
    let beta_bounds = match constraint_bounds {
        Some((lb, ub)) if lb.len() > 1 => (lb[1], ub[1]),
        _ => (wid_optimize::reed_calib::DEFAULT_BETA_LOWER, wid_optimize::reed_calib::DEFAULT_BETA_UPPER),
    };
    let result = wid_optimize::reed_calib::calibrate_reed(
        inst, tuning, params, alpha_bounds, beta_bounds, calc_params,
    );
    Ok(CalibResult {
        initial_fipple_factor: None, final_fipple_factor: None,
        initial_window_height: None, final_window_height: None,
        initial_airstream_length: None, final_airstream_length: None,
        initial_alpha: result.initial_alpha, final_alpha: result.final_alpha,
        initial_beta: result.initial_beta, final_beta: result.final_beta,
        initial_norm: result.initial_norm, final_norm: result.final_norm,
    })
}

// ── XML serialization helpers ───────────────────────────────────────

fn serialize_instrument_xml(inst: &InstrumentRaw) -> Result<String, quick_xml::SeError> {
    let inner = quick_xml::se::to_string(inst)?;
    // Wrap with namespace prefix to match WIDesigner format
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{}",
        add_namespace(&inner, "instrument", "http://www.wwidesigner.com/Instrument")
    );
    Ok(xml)
}

fn serialize_tuning_xml(tuning: &Tuning) -> Result<String, quick_xml::SeError> {
    let inner = quick_xml::se::to_string(tuning)?;
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{}",
        add_namespace(&inner, "tuning", "http://www.wwidesigner.com/Tuning")
    );
    Ok(xml)
}

fn serialize_constraints_xml(constraints: &Constraints) -> Result<String, quick_xml::SeError> {
    let inner = quick_xml::se::to_string(constraints)?;
    let xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{}",
        add_namespace(&inner, "constraints", "http://www.wwidesigner.com/Constraints")
    );
    Ok(xml)
}

/// Add WIDesigner namespace prefix to the root element.
fn add_namespace(xml: &str, root_tag: &str, namespace: &str) -> String {
    let open = format!("<{root_tag}");
    let replacement = format!("<ns2:{root_tag} xmlns:ns2=\"{namespace}\"");
    let close = format!("</{root_tag}>");
    let close_replacement = format!("</ns2:{root_tag}>");
    xml.replacen(&open, &replacement, 1).replace(&close, &close_replacement)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAF_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/instruments/0.75-bore_6-hole_NAF_starter.xml"
    );
    const TUNING_6HOLE_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/NafStudy/tunings/F#4_ET_6-hole_NAF_chromatic_tuning.xml"
    );
    const CONSTRAINTS_XML: &str = include_str!(
        "../../../../oracle/v2.6.0/constraints/NafStudyModel/HoleFromTopObjectiveFunction/6/1.25_max_hole_spacing.xml"
    );

    // ── Session lifecycle ───────────────────────────────────────────

    #[test]
    fn new_session_has_empty_selection() {
        let session = StudySession::new(StudyKind::NAF);
        assert!(session.selection().instrument_id.is_none());
        assert!(session.selection().tuning_id.is_none());
        assert!(session.selection().optimizer_key.is_none());
        assert!(session.selection().constraints_id.is_none());
    }

    // ── Document I/O ────────────────────────────────────────────────

    #[test]
    fn open_instrument_xml() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(NAF_6HOLE_XML).unwrap();
        assert_eq!(result.doc_kind, DocKind::Instrument);
        assert_eq!(result.name, "3/4\" bore, 6-hole NAF start");
    }

    #[test]
    fn open_tuning_xml() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(TUNING_6HOLE_XML).unwrap();
        assert_eq!(result.doc_kind, DocKind::Tuning);
        assert_eq!(result.name, "F#4 ET 6-hole NAF chromatic tuning");
    }

    #[test]
    fn open_constraints_xml() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(CONSTRAINTS_XML).unwrap();
        assert_eq!(result.doc_kind, DocKind::Constraints);
    }

    #[test]
    fn open_invalid_xml_fails() {
        let mut session = StudySession::new(StudyKind::NAF);
        assert!(session.open_xml("<not-valid>oops</not-valid>").is_err());
    }

    // ── Gating ──────────────────────────────────────────────────────

    #[test]
    fn cannot_tune_without_selection() {
        let session = StudySession::new(StudyKind::NAF);
        assert!(!session.can_tune());
    }

    #[test]
    fn can_tune_with_matching_docs() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        let tuning = session.open_xml(TUNING_6HOLE_XML).unwrap();
        session.select_instrument(inst.doc_id);
        session.select_tuning(tuning.doc_id);
        assert!(session.can_tune());
    }

    #[test]
    fn can_sketch_with_instrument_only() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        session.select_instrument(inst.doc_id);
        assert!(session.can_sketch());
        assert!(!session.can_tune()); // No tuning yet
    }

    #[test]
    fn can_optimize_requires_all() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        let tuning = session.open_xml(TUNING_6HOLE_XML).unwrap();
        let constraints = session.open_xml(CONSTRAINTS_XML).unwrap();

        session.select_instrument(inst.doc_id);
        session.select_tuning(tuning.doc_id);
        assert!(!session.can_optimize()); // No optimizer yet

        session.select_optimizer(naf::HOLE_FROM_TOP);
        assert!(!session.can_optimize()); // No constraints yet

        session.select_constraints(constraints.doc_id);
        assert!(session.can_optimize());
    }

    #[test]
    fn fipple_optimizer_doesnt_need_constraints() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        let tuning = session.open_xml(TUNING_6HOLE_XML).unwrap();
        session.select_instrument(inst.doc_id);
        session.select_tuning(tuning.doc_id);
        session.select_optimizer(naf::FIPPLE_FACTOR);
        assert!(session.can_optimize());
    }

    // ── Available optimizers ────────────────────────────────────────

    #[test]
    fn naf_has_four_optimizers() {
        let session = StudySession::new(StudyKind::NAF);
        let opts = session.available_optimizers();
        assert_eq!(opts.len(), 4);
        let keys: Vec<&str> = opts.iter().map(|o| o.key.as_str()).collect();
        assert!(keys.contains(&naf::FIPPLE_FACTOR));
        assert!(keys.contains(&naf::HOLE_FROM_TOP));
        assert!(keys.contains(&naf::NAF_HOLE_SIZE));
        assert!(keys.contains(&naf::HOLE_GROUP_FROM_TOP));
    }

    // ── Evaluate tuning ─────────────────────────────────────────────

    #[test]
    fn evaluate_fails_without_selection() {
        let session = StudySession::new(StudyKind::NAF);
        assert!(session.evaluate_tuning().is_err());
    }

    #[test]
    fn evaluate_tuning_matches_golden() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        let tuning = session.open_xml(TUNING_6HOLE_XML).unwrap();
        session.select_instrument(inst.doc_id);
        session.select_tuning(tuning.doc_id);

        let result = session.evaluate_tuning().unwrap();
        assert_eq!(result.rows.len(), 15);

        // Compare against golden NAF-FF-01/eval_0.json
        #[derive(serde::Deserialize)]
        #[allow(dead_code)]
        struct GoldenEval {
            note: String,
            #[serde(rename = "targetFreq")]
            target_freq: f64,
            #[serde(rename = "predictedFreq")]
            predicted_freq: f64,
            cents: f64,
        }

        let golden: Vec<GoldenEval> = serde_json::from_str(include_str!(
            "../../../../golden/expected/NAF-FF-01/eval_0.json"
        )).unwrap();

        for (row, exp) in result.rows.iter().zip(golden.iter()) {
            assert_eq!(row.note, exp.note);
            assert!(
                (row.cents - exp.cents).abs() < 0.5,
                "{}: expected {:.4} cents, got {:.4}",
                row.note, exp.cents, row.cents
            );
        }
    }

    // ── Constraint generation ───────────────────────────────────────

    #[test]
    fn create_default_constraints_matches_golden() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        session.select_instrument(inst.doc_id);

        let result = session
            .create_default_constraints(naf::HOLE_FROM_TOP)
            .unwrap();
        assert_eq!(result.doc_kind, DocKind::Constraints);

        let constraints = session.docs().get_constraints(result.doc_id).unwrap();
        assert_eq!(constraints.name, "Default");
        assert_eq!(
            constraints.objective_function_name,
            "HoleFromTopObjectiveFunction"
        );
        assert_eq!(constraints.number_of_holes, 6);
        assert_eq!(constraints.constraint_list.len(), 13);

        // All bounds should be None/0.0 for default constraints
        let lb = constraints.lower_bounds();
        let ub = constraints.upper_bounds();
        assert_eq!(lb.len(), 13);
        assert_eq!(ub.len(), 13);
        for i in 0..13 {
            assert_eq!(lb[i], 0.0);
            assert_eq!(ub[i], 0.0);
        }
    }

    // ── XML round-trip ──────────────────────────────────────────────

    #[test]
    fn instrument_xml_roundtrip() {
        let mut session = StudySession::new(StudyKind::NAF);
        let orig = session.open_xml(NAF_6HOLE_XML).unwrap();

        // Export
        let xml = session.export_xml(orig.doc_id).unwrap();
        assert!(xml.contains("ns2:instrument"));
        assert!(xml.contains("wwidesigner.com"));

        // Re-import
        let reimported = session.open_xml(&xml).unwrap();
        assert_eq!(reimported.doc_kind, DocKind::Instrument);

        // Compare key fields
        let inst1 = session.docs().get_instrument(orig.doc_id).unwrap();
        let inst2 = session.docs().get_instrument(reimported.doc_id).unwrap();
        assert_eq!(inst1.name, inst2.name);
        assert_eq!(inst1.holes.len(), inst2.holes.len());
        assert_eq!(inst1.bore_points.len(), inst2.bore_points.len());
    }

    #[test]
    fn tuning_xml_roundtrip() {
        let mut session = StudySession::new(StudyKind::NAF);
        let orig = session.open_xml(TUNING_6HOLE_XML).unwrap();
        let xml = session.export_xml(orig.doc_id).unwrap();
        let reimported = session.open_xml(&xml).unwrap();
        assert_eq!(reimported.doc_kind, DocKind::Tuning);

        let t1 = session.docs().get_tuning(orig.doc_id).unwrap();
        let t2 = session.docs().get_tuning(reimported.doc_id).unwrap();
        assert_eq!(t1.name, t2.name);
        assert_eq!(t1.fingerings.len(), t2.fingerings.len());
    }

    // ── Delete holes ────────────────────────────────────────────────

    #[test]
    fn delete_instrument_holes() {
        let mut session = StudySession::new(StudyKind::NAF);
        let inst = session.open_xml(NAF_6HOLE_XML).unwrap();
        assert_eq!(
            session.docs().get_instrument(inst.doc_id).unwrap().holes.len(),
            6
        );

        session.delete_instrument_holes(inst.doc_id).unwrap();
        assert_eq!(
            session.docs().get_instrument(inst.doc_id).unwrap().holes.len(),
            0
        );
    }

    // ── Document get/set ──────────────────────────────────────────

    #[test]
    fn get_and_set_instrument_roundtrip() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(NAF_6HOLE_XML).unwrap();
        let doc_id = result.doc_id;

        let mut inst = session.get_instrument(doc_id).unwrap().clone();
        assert_eq!(inst.holes.len(), 6);

        // Modify and set back
        inst.name = "Modified NAF".to_string();
        inst.holes[0].diameter = 0.999;
        session.set_instrument(doc_id, inst).unwrap();

        let updated = session.get_instrument(doc_id).unwrap();
        assert_eq!(updated.name, "Modified NAF");
        assert!((updated.holes[0].diameter - 0.999).abs() < 1e-10);
    }

    #[test]
    fn get_and_set_tuning_roundtrip() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(TUNING_6HOLE_XML).unwrap();
        let doc_id = result.doc_id;

        let mut tuning = session.get_tuning(doc_id).unwrap().clone();
        assert_eq!(tuning.fingerings.len(), 15);

        tuning.name = "Modified Tuning".to_string();
        session.set_tuning(doc_id, tuning).unwrap();

        let updated = session.get_tuning(doc_id).unwrap();
        assert_eq!(updated.name, "Modified Tuning");
        assert_eq!(updated.fingerings.len(), 15);
    }

    #[test]
    fn get_and_set_constraints_roundtrip() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(CONSTRAINTS_XML).unwrap();
        let doc_id = result.doc_id;

        let mut constraints = session.get_constraints(doc_id).unwrap().clone();
        constraints.name = "Modified Constraints".to_string();
        session.set_constraints(doc_id, constraints).unwrap();

        let updated = session.get_constraints(doc_id).unwrap();
        assert_eq!(updated.name, "Modified Constraints");
    }

    #[test]
    fn set_instrument_updates_doc_name() {
        let mut session = StudySession::new(StudyKind::NAF);
        let result = session.open_xml(NAF_6HOLE_XML).unwrap();
        let doc_id = result.doc_id;

        let mut inst = session.get_instrument(doc_id).unwrap().clone();
        inst.name = "New Name".to_string();
        session.set_instrument(doc_id, inst).unwrap();

        // list_docs should reflect the new name
        let docs = session.list_docs(DocKind::Instrument);
        let entry = docs.iter().find(|(id, _)| *id == doc_id).unwrap();
        assert_eq!(entry.1, "New Name");
    }

    #[test]
    fn set_nonexistent_doc_fails() {
        let mut session = StudySession::new(StudyKind::NAF);
        let fake_id = DocId(999);
        let inst = parse_instrument_xml(NAF_6HOLE_XML).unwrap();
        assert!(session.set_instrument(fake_id, inst).is_err());
    }
}
