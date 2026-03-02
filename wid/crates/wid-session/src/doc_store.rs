//! Document storage for session-managed instrument, tuning, and constraint files.

use crate::types::{DocId, DocKind};
use wid_types::{Constraints, InstrumentRaw, Tuning};

/// A stored document with its parsed content and original XML.
#[derive(Debug, Clone)]
pub struct StoredDoc {
    pub id: DocId,
    pub kind: DocKind,
    pub name: String,
    pub content: DocContent,
}

/// Parsed content of a stored document.
#[derive(Debug, Clone)]
pub enum DocContent {
    Instrument(InstrumentRaw),
    Tuning(Tuning),
    Constraints(Constraints),
}

/// Manages all documents within a session.
#[derive(Debug)]
pub struct DocStore {
    docs: Vec<StoredDoc>,
    next_id: u32,
}

impl DocStore {
    pub fn new() -> Self {
        DocStore {
            docs: Vec::new(),
            next_id: 1,
        }
    }

    /// Store a new document, returning its assigned ID.
    pub fn insert(&mut self, kind: DocKind, name: String, content: DocContent) -> DocId {
        let id = DocId(self.next_id);
        self.next_id += 1;
        self.docs.push(StoredDoc {
            id,
            kind,
            name,
            content,
        });
        id
    }

    /// Get a document by ID.
    pub fn get(&self, id: DocId) -> Option<&StoredDoc> {
        self.docs.iter().find(|d| d.id == id)
    }

    /// Get a mutable reference to a document by ID.
    pub fn get_mut(&mut self, id: DocId) -> Option<&mut StoredDoc> {
        self.docs.iter_mut().find(|d| d.id == id)
    }

    /// List documents of a given kind.
    pub fn list(&self, kind: DocKind) -> Vec<&StoredDoc> {
        self.docs.iter().filter(|d| d.kind == kind).collect()
    }

    /// Remove a document by ID. Returns true if removed.
    pub fn remove(&mut self, id: DocId) -> bool {
        let len = self.docs.len();
        self.docs.retain(|d| d.id != id);
        self.docs.len() < len
    }

    /// Get the instrument content for a given doc ID.
    pub fn get_instrument(&self, id: DocId) -> Option<&InstrumentRaw> {
        match &self.get(id)?.content {
            DocContent::Instrument(inst) => Some(inst),
            _ => None,
        }
    }

    /// Get a mutable reference to the instrument content.
    pub fn get_instrument_mut(&mut self, id: DocId) -> Option<&mut InstrumentRaw> {
        match &mut self.get_mut(id)?.content {
            DocContent::Instrument(inst) => Some(inst),
            _ => None,
        }
    }

    /// Get the tuning content for a given doc ID.
    pub fn get_tuning(&self, id: DocId) -> Option<&Tuning> {
        match &self.get(id)?.content {
            DocContent::Tuning(tuning) => Some(tuning),
            _ => None,
        }
    }

    /// Get the constraints content for a given doc ID.
    pub fn get_constraints(&self, id: DocId) -> Option<&Constraints> {
        match &self.get(id)?.content {
            DocContent::Constraints(c) => Some(c),
            _ => None,
        }
    }
}
