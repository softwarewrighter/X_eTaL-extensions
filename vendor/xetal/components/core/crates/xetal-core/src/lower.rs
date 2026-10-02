//! The lowering state: fresh node ids and names, and local scopes.

use xetal_ir::{Expr, Kind, Program};

use std::collections::HashSet;

use xetal_base::{Diagnostic, NodeId, Span};

/// Parse and lower a whole program to Core.
pub fn lower(src: &str) -> Result<Program, Diagnostic> {
    let program = xetal_syntax::parse(src)?;
    let mut lower = Lower {
        next_id: 0,
        fresh: 0,
        scopes: vec![HashSet::new()],
        lambdas: 0,
        src: src.to_string(),
        notes: Vec::new(),
    };
    lower.program(&program)
}

pub(crate) fn err(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}

pub(crate) struct Lower {
    pub(crate) next_id: u32,
    pub(crate) fresh: u32,
    /// Names bound locally (parameters, local bindings), innermost last.
    pub(crate) scopes: Vec<HashSet<String>>,
    /// How many lambdas enclose the current point.
    pub(crate) lambdas: usize,
    /// The source, for spelling train elements in notes.
    pub(crate) src: String,
    /// Notes for errors at particular spans (D47).
    pub(crate) notes: Vec<xetal_ir::SpanNote>,
}

impl Lower {
    pub(crate) fn node(&mut self, span: Span, kind: Kind) -> Expr {
        self.next_id += 1;
        Expr {
            id: NodeId(self.next_id),
            span,
            kind,
        }
    }

    /// A fresh name no source program can spell (`%1`, `%2`, ...).
    pub(crate) fn fresh_name(&mut self) -> String {
        self.fresh += 1;
        format!("%{}", self.fresh)
    }

    pub(crate) fn is_bound(&self, name: &str) -> bool {
        self.scopes.iter().any(|s| s.contains(name))
    }

    pub(crate) fn bind(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string());
        }
    }
}
