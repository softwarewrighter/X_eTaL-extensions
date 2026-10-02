//! A file's top-level definitions: exports (`l:`) and private names in
//! a library; what may not be defined where (MC8 rows 9, 12, 16).

use std::collections::HashSet;

use xetal_base::{Diagnostic, Span};
use xetal_lex::{Token, TokenKind};

use crate::errors::{L_IN_PROGRAM, PRINTS, fail};
use crate::imports::statements;
use crate::names::Context;
use crate::rename::name;

/// What a file defines at the top level.
#[derive(Default)]
pub(crate) struct Defs {
    pub privates: HashSet<String>,
    pub exports: Vec<String>,
    functions: HashSet<(Option<String>, String)>,
}

/// The definitions of `tokens`, checking each top-level statement.
pub(crate) fn top_level(tokens: &[Token], cx: &Context) -> Result<Defs, Diagnostic> {
    let mut defs = Defs::default();
    for statement in statements(tokens) {
        let (Some(first), Some(last)) = (statement.first(), statement.last()) else {
            continue;
        };
        let span = Span::new(first.span.start, last.span.end);
        if !cx.imports.contains(&span) {
            defs.statement(statement, span, cx)?;
        }
    }
    Ok(defs)
}

impl Defs {
    /// A binding is recorded; any other statement is an expression.
    fn statement(
        &mut self,
        statement: &[Token],
        span: Span,
        cx: &Context,
    ) -> Result<(), Diagnostic> {
        let first = &statement[0];
        let target = match statement.get(1).map(|t| &t.kind) {
            Some(TokenKind::Assign) => name(first),
            _ => None,
        };
        match (target, cx.library.is_some()) {
            (Some((ns, key)), _) => self.record(first, ns, key, cx),
            (None, true) => Err(fail("expression-in-library", span, PRINTS)),
            (None, false) => Ok(()),
        }
    }

    fn record(
        &mut self,
        first: &Token,
        ns: Option<String>,
        key: String,
        cx: &Context,
    ) -> Result<(), Diagnostic> {
        let function = matches!(first.kind, TokenKind::Func(_));
        if function && !self.functions.insert((ns.clone(), key.clone())) {
            let written = ns
                .as_deref()
                .map_or(key.clone(), |ns| format!("{ns}:{key}"));
            let message = format!("{written} is already defined in this file");
            return Err(fail("duplicate-definition", first.span, message));
        }
        match (ns.as_deref(), cx.library.is_some()) {
            (Some("l"), false) => {
                return Err(fail("library-name-in-program", first.span, L_IN_PROGRAM));
            }
            (Some("l"), true) => self.exports.push(key),
            (None, true) => drop(self.privates.insert(key)),
            _ => {}
        }
        Ok(())
    }
}
