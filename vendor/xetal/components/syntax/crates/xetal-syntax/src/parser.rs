//! The parser state and the entry point.

use xetal_ast::Program;
use xetal_base::{Diagnostic, Span};
use xetal_lex::{Token, lex};

/// The deepest bracket nesting the parser accepts.
pub const MAX_NESTING: usize = 64;

/// Lex and parse a whole program.
pub fn parse(src: &str) -> Result<Program, Diagnostic> {
    let tokens = lex(src)?;
    let mut parser = Parser {
        tokens,
        pos: 0,
        end: src.len(),
        newline_is_space: vec![false],
        nesting: 0,
    };
    parser.program()
}

pub(crate) fn err(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}

pub(crate) struct Parser {
    pub(crate) tokens: Vec<Token>,
    pub(crate) pos: usize,
    pub(crate) end: usize,
    /// Inside `( )` and `[ ]` a newline is whitespace; elsewhere it
    /// separates statements (S2).
    pub(crate) newline_is_space: Vec<bool>,
    /// Open brackets around the current point.
    pub(crate) nesting: usize,
}
