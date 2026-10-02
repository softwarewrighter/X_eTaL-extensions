//! Moving through the tokens: newlines as whitespace inside brackets,
//! and the bracket nesting limit.

use xetal_base::{Diagnostic, Span};
use xetal_lex::{Token, TokenKind};

use crate::parser::{MAX_NESTING, Parser, err};

impl Parser {
    /// The next token, skipping newlines where they are whitespace.
    pub(crate) fn peek(&mut self) -> Option<&Token> {
        if self.newline_is_space.last() == Some(&true) {
            while self
                .tokens
                .get(self.pos)
                .is_some_and(|t| t.kind == TokenKind::Newline)
            {
                self.pos += 1;
            }
        }
        self.tokens.get(self.pos)
    }

    pub(crate) fn next(&mut self) -> Option<Token> {
        let token = self.peek().cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Enter a bracket; too many open brackets are `too-deep`.
    pub(crate) fn enter(&mut self, open: Span) -> Result<(), Diagnostic> {
        self.nesting += 1;
        if self.nesting > MAX_NESTING {
            self.nesting -= 1;
            return Err(err(
                "too-deep",
                open,
                format!("brackets nest more than {MAX_NESTING} levels deep"),
            ));
        }
        Ok(())
    }

    pub(crate) fn leave(&mut self) {
        self.nesting -= 1;
    }

    /// The span of the next token, or an empty span at the end.
    pub(crate) fn here(&mut self) -> Span {
        let end = self.end;
        self.peek().map_or(Span::new(end, end), |t| t.span)
    }
}
