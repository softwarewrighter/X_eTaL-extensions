//! Lexer errors; every one carries the span of the offending text.

use xetal_base::{Diagnostic, Span};

/// What went wrong; `code()` is the stable diagnostic code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    BadName,
    BadMark,
    BadAxis,
    BadNamespace,
    BadExponent,
    BadPower,
    ReservedSuperscript,
    AmbiguousBang,
    NoNiladicSugar,
    BadLambdaArg,
    BadGuard,
    BadQuote,
    BadLazy,
    BadString,
    AmbiguousMinus,
    BadNumber,
    NumberOutOfRange,
    UnexpectedChar,
    NonAscii,
}

impl ErrorKind {
    pub fn code(self) -> &'static str {
        match self {
            ErrorKind::BadName => "bad-name",
            ErrorKind::BadMark => "bad-mark",
            ErrorKind::BadAxis => "bad-axis",
            ErrorKind::BadNamespace => "bad-namespace",
            ErrorKind::BadExponent => "bad-exponent",
            ErrorKind::BadPower => "bad-power",
            ErrorKind::ReservedSuperscript => "reserved-superscript",
            ErrorKind::AmbiguousBang => "ambiguous-bang",
            ErrorKind::NoNiladicSugar => "no-niladic-sugar",
            ErrorKind::BadLambdaArg => "bad-lambda-arg",
            ErrorKind::BadGuard => "bad-guard",
            ErrorKind::BadQuote => "bad-quote",
            ErrorKind::BadLazy => "bad-lazy",
            ErrorKind::BadString => "bad-string",
            ErrorKind::AmbiguousMinus => "ambiguous-minus",
            ErrorKind::BadNumber => "bad-number",
            ErrorKind::NumberOutOfRange => "number-out-of-range",
            ErrorKind::UnexpectedChar => "unexpected-char",
            ErrorKind::NonAscii => "non-ascii",
        }
    }
}

/// A lexing failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub kind: ErrorKind,
    pub span: Span,
    pub message: String,
}

impl LexError {
    pub fn new(kind: ErrorKind, span: Span, message: impl Into<String>) -> Self {
        Self {
            kind,
            span,
            message: message.into(),
        }
    }

    /// An error covering the single byte at `pos`.
    pub fn at(kind: ErrorKind, pos: usize, message: impl Into<String>) -> Self {
        Self::new(kind, Span::new(pos, pos + 1), message)
    }

    pub fn code(&self) -> &'static str {
        self.kind.code()
    }

    /// `!=` touching a name (review R2): ask for a space.
    pub fn bang_equals(pos: usize) -> Self {
        Self::new(
            ErrorKind::AmbiguousBang,
            Span::new(pos, pos + 2),
            "`!=` touching a name is ambiguous: write `x != 3` (not equal) or `x! = 3` (compare a mutable variable)",
        )
    }
}

impl From<LexError> for Diagnostic {
    fn from(err: LexError) -> Self {
        Diagnostic::new(err.code(), err.message).with_span(err.span)
    }
}
