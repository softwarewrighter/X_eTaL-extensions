//! Notes the lowering leaves for errors at particular spans (D47): an
//! error inside a train is explained by what the train means there.

use xetal_base::{Diagnostic, Span};

use crate::ir::Program;

/// Notes to add to an error reported at exactly `span`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SpanNote {
    pub span: Span,
    pub notes: Vec<String>,
}

impl Program {
    /// `d` with the notes left for its span.
    pub fn annotate(&self, mut d: Diagnostic) -> Diagnostic {
        if let Some(span) = d.span {
            for note in self.notes.iter().filter(|n| n.span == span) {
                d.notes.extend(note.notes.iter().cloned());
            }
        }
        d
    }
}
