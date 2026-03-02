//! WASM bindings for WIDesigner session API.
//!
//! Provides a thin JSON-based dispatch layer over [`wid_session::StudySession`].
//! All commands go through `execute(json)` for simplicity, except `optimize()`
//! which needs a JS progress callback.

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wid_session::{StudyKind, StudySession};

/// WASM-exposed session wrapper.
#[wasm_bindgen]
pub struct WasmSession {
    session: StudySession,
}

/// JSON command envelope for the execute() dispatch.
#[derive(Deserialize)]
struct Command {
    cmd: String,
    #[serde(default)]
    args: serde_json::Value,
}

/// JSON response envelope.
#[derive(Serialize)]
struct Response {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl Response {
    fn ok(data: impl Serialize) -> String {
        serde_json::to_string(&Response {
            ok: true,
            data: Some(serde_json::to_value(data).unwrap_or(serde_json::Value::Null)),
            error: None,
        })
        .unwrap_or_else(|_| r#"{"ok":false,"error":"serialization failed"}"#.to_string())
    }

    fn err(msg: impl std::fmt::Display) -> String {
        serde_json::to_string(&Response {
            ok: false,
            data: None,
            error: Some(msg.to_string()),
        })
        .unwrap_or_else(|_| r#"{"ok":false,"error":"serialization failed"}"#.to_string())
    }
}

#[wasm_bindgen]
impl WasmSession {
    /// Create a new session. study_kind should be "NAF".
    #[wasm_bindgen(constructor)]
    pub fn new(study_kind: &str) -> Result<WasmSession, JsValue> {
        let kind = match study_kind {
            "NAF" => StudyKind::NAF,
            _ => return Err(JsValue::from_str(&format!("Unknown study kind: {study_kind}"))),
        };
        Ok(WasmSession {
            session: StudySession::new(kind),
        })
    }

    /// Synchronous command dispatch. JSON in, JSON out.
    pub fn execute(&mut self, command_json: &str) -> String {
        let cmd: Command = match serde_json::from_str(command_json) {
            Ok(c) => c,
            Err(e) => return Response::err(format!("Invalid command JSON: {e}")),
        };

        match cmd.cmd.as_str() {
            "open_xml" => self.cmd_open_xml(&cmd.args),
            "export_xml" => self.cmd_export_xml(&cmd.args),
            "select_instrument" => self.cmd_select_instrument(&cmd.args),
            "select_tuning" => self.cmd_select_tuning(&cmd.args),
            "select_optimizer" => self.cmd_select_optimizer(&cmd.args),
            "select_constraints" => self.cmd_select_constraints(&cmd.args),
            "clear_selection" => self.cmd_clear_selection(),
            "can_tune" => Response::ok(self.session.can_tune()),
            "can_optimize" => Response::ok(self.session.can_optimize()),
            "can_sketch" => Response::ok(self.session.can_sketch()),
            "available_optimizers" => Response::ok(self.session.available_optimizers()),
            "evaluate_tuning" => self.cmd_evaluate_tuning(),
            "calibrate" => self.cmd_calibrate(),
            "create_default_constraints" => self.cmd_create_default_constraints(&cmd.args),
            "create_blank_constraints" => self.cmd_create_blank_constraints(&cmd.args),
            "delete_instrument_holes" => self.cmd_delete_holes(&cmd.args),
            "get_selection" => Response::ok(self.session.selection()),
            "list_instruments" => {
                Response::ok(self.session.list_docs(wid_session::DocKind::Instrument))
            }
            "list_tunings" => {
                Response::ok(self.session.list_docs(wid_session::DocKind::Tuning))
            }
            "list_constraints" => {
                Response::ok(self.session.list_docs(wid_session::DocKind::Constraints))
            }
            "get_params" => self.cmd_get_params(),
            _ => Response::err(format!("Unknown command: {}", cmd.cmd)),
        }
    }

    /// Run optimization with JS progress callback.
    ///
    /// The callback receives a JSON string `{"evaluations": N, "bestNorm": F}`
    /// and should return `true` to continue or `false` to cancel.
    pub fn optimize(&mut self, progress_callback: &js_sys::Function) -> String {
        let this = JsValue::NULL;
        let result = self.session.optimize(&mut |progress| {
            let json = serde_json::to_string(&progress).unwrap_or_default();
            let js_str = JsValue::from_str(&json);
            match progress_callback.call1(&this, &js_str) {
                Ok(val) => val.as_bool().unwrap_or(true),
                Err(_) => false,
            }
        });

        match result {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    // ── Command handlers ────────────────────────────────────────────

    fn cmd_open_xml(&mut self, args: &serde_json::Value) -> String {
        let xml = match args.get("xml").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Response::err("Missing 'xml' argument"),
        };
        match self.session.open_xml(xml) {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_export_xml(&self, args: &serde_json::Value) -> String {
        let doc_id = match args.get("docId").and_then(|v| v.as_u64()) {
            Some(id) => wid_session::DocId(id as u32),
            None => return Response::err("Missing 'docId' argument"),
        };
        match self.session.export_xml(doc_id) {
            Ok(xml) => Response::ok(xml),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_select_instrument(&mut self, args: &serde_json::Value) -> String {
        let doc_id = match args.get("docId").and_then(|v| v.as_u64()) {
            Some(id) => wid_session::DocId(id as u32),
            None => return Response::err("Missing 'docId' argument"),
        };
        self.session.select_instrument(doc_id);
        Response::ok(true)
    }

    fn cmd_select_tuning(&mut self, args: &serde_json::Value) -> String {
        let doc_id = match args.get("docId").and_then(|v| v.as_u64()) {
            Some(id) => wid_session::DocId(id as u32),
            None => return Response::err("Missing 'docId' argument"),
        };
        self.session.select_tuning(doc_id);
        Response::ok(true)
    }

    fn cmd_select_optimizer(&mut self, args: &serde_json::Value) -> String {
        let key = match args.get("key").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Response::err("Missing 'key' argument"),
        };
        self.session.select_optimizer(key);
        Response::ok(true)
    }

    fn cmd_select_constraints(&mut self, args: &serde_json::Value) -> String {
        let doc_id = match args.get("docId").and_then(|v| v.as_u64()) {
            Some(id) => wid_session::DocId(id as u32),
            None => return Response::err("Missing 'docId' argument"),
        };
        self.session.select_constraints(doc_id);
        Response::ok(true)
    }

    fn cmd_clear_selection(&mut self) -> String {
        self.session.clear_selection();
        Response::ok(true)
    }

    fn cmd_evaluate_tuning(&self) -> String {
        match self.session.evaluate_tuning() {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_calibrate(&mut self) -> String {
        match self.session.calibrate() {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_create_default_constraints(&mut self, args: &serde_json::Value) -> String {
        let key = match args.get("optimizerKey").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Response::err("Missing 'optimizerKey' argument"),
        };
        match self.session.create_default_constraints(key) {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_create_blank_constraints(&mut self, args: &serde_json::Value) -> String {
        let key = match args.get("optimizerKey").and_then(|v| v.as_str()) {
            Some(s) => s,
            None => return Response::err("Missing 'optimizerKey' argument"),
        };
        match self.session.create_blank_constraints(key) {
            Ok(r) => Response::ok(r),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_delete_holes(&mut self, args: &serde_json::Value) -> String {
        let doc_id = match args.get("docId").and_then(|v| v.as_u64()) {
            Some(id) => wid_session::DocId(id as u32),
            None => return Response::err("Missing 'docId' argument"),
        };
        match self.session.delete_instrument_holes(doc_id) {
            Ok(()) => Response::ok(true),
            Err(e) => Response::err(e),
        }
    }

    fn cmd_get_params(&self) -> String {
        let params = self.session.params();
        Response::ok(serde_json::json!({
            "temperature": params.temperature(),
            "pressure": params.pressure(),
            "humidity": params.humidity(),
            "co2Ppm": params.x_co2() * 1e6,
            "speedOfSound": params.speed_of_sound(),
            "density": params.rho(),
            "epsilonConstant": params.epsilon_constant(),
        }))
    }
}
