//! Document storage for session-managed instrument, tuning, and constraint files.

use crate::types::{DocId, DocKind};
use wid_types::{Constraints, InstrumentRaw, Scale, ScaleSymbolList, Temperament, Tuning};

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
    Scale(Scale),
    Temperament(Temperament),
    ScaleSymbolList(ScaleSymbolList),
}

/// Manages all documents within a session.
#[derive(Debug)]
pub struct DocStore {
    docs: Vec<StoredDoc>,
    next_id: u32,
}

impl Default for DocStore {
    fn default() -> Self {
        DocStore {
            docs: Vec::new(),
            next_id: 1,
        }
    }
}

impl DocStore {
    pub fn new() -> Self {
        Self::default()
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

    /// Get a mutable reference to the tuning content.
    pub fn get_tuning_mut(&mut self, id: DocId) -> Option<&mut Tuning> {
        match &mut self.get_mut(id)?.content {
            DocContent::Tuning(tuning) => Some(tuning),
            _ => None,
        }
    }

    /// Get a mutable reference to the constraints content.
    pub fn get_constraints_mut(&mut self, id: DocId) -> Option<&mut Constraints> {
        match &mut self.get_mut(id)?.content {
            DocContent::Constraints(c) => Some(c),
            _ => None,
        }
    }

    /// Replace the instrument content for a given doc ID.
    /// Returns `None` if the doc doesn't exist or isn't an instrument.
    pub fn replace_instrument(&mut self, id: DocId, inst: InstrumentRaw) -> Option<()> {
        let doc = self.get_mut(id)?;
        match &doc.content {
            DocContent::Instrument(_) => {
                doc.name = inst.name.clone();
                doc.content = DocContent::Instrument(inst);
                Some(())
            }
            _ => None,
        }
    }

    /// Replace the tuning content for a given doc ID.
    /// Returns `None` if the doc doesn't exist or isn't a tuning.
    pub fn replace_tuning(&mut self, id: DocId, tuning: Tuning) -> Option<()> {
        let doc = self.get_mut(id)?;
        match &doc.content {
            DocContent::Tuning(_) => {
                doc.name = tuning.name.clone();
                doc.content = DocContent::Tuning(tuning);
                Some(())
            }
            _ => None,
        }
    }

    /// Get the scale content for a given doc ID.
    pub fn get_scale(&self, id: DocId) -> Option<&Scale> {
        match &self.get(id)?.content {
            DocContent::Scale(s) => Some(s),
            _ => None,
        }
    }

    /// Get the temperament content for a given doc ID.
    pub fn get_temperament(&self, id: DocId) -> Option<&Temperament> {
        match &self.get(id)?.content {
            DocContent::Temperament(t) => Some(t),
            _ => None,
        }
    }

    /// Get the scale symbol list content for a given doc ID.
    pub fn get_scale_symbol_list(&self, id: DocId) -> Option<&ScaleSymbolList> {
        match &self.get(id)?.content {
            DocContent::ScaleSymbolList(s) => Some(s),
            _ => None,
        }
    }

    /// Replace the constraints content for a given doc ID.
    /// Returns `None` if the doc doesn't exist or isn't a constraints doc.
    pub fn replace_constraints(&mut self, id: DocId, constraints: Constraints) -> Option<()> {
        let doc = self.get_mut(id)?;
        match &doc.content {
            DocContent::Constraints(_) => {
                doc.name = constraints.name.clone();
                doc.content = DocContent::Constraints(constraints);
                Some(())
            }
            _ => None,
        }
    }
}
