//! Finding the imports of one file: a top-level statement of exactly
//! `"alias:" u_se< "Library"` (MC3); any other use of `u_se<` is an
//! error from the table (MC8 rows 3, 4, 5, 13, 15). Other macros are
//! expanded before (xetal-expand).

use xetal_base::{Diagnostic, Span};
use xetal_lex::{Token, TokenKind, lex};

use crate::errors::{MISPLACED, MISSING, STRINGS, fail, valid_alias};

/// One import statement.
#[derive(Debug, Clone)]
pub struct Import {
    pub alias: String,
    pub spec: String,
    /// The whole statement, removed from the program text.
    pub span: Span,
}

/// The imports of `text`; text that does not lex has none (the parser
/// reports it later).
pub fn imports(text: &str) -> Result<Vec<Import>, Diagnostic> {
    let Ok(tokens) = lex(text) else {
        return Ok(Vec::new());
    };
    let mut found = Vec::new();
    for statement in statements(&tokens) {
        for (i, t) in statement.iter().enumerate() {
            if let TokenKind::Func(f) = &t.kind
                && f.is_macro()
                && f.ns.is_none()
                && f.spelled() == "u_se<"
            {
                found.push(import(statement, i, t)?);
            }
        }
    }
    Ok(found)
}

/// Top-level statements: tokens between newlines and `;` outside
/// brackets. A macro inside brackets makes the whole bracket part of
/// its statement, so it is caught as misplaced.
pub(crate) fn statements(tokens: &[Token]) -> Vec<&[Token]> {
    let (mut out, mut start, mut depth) = (Vec::new(), 0, 0i32);
    for (i, t) in tokens.iter().enumerate() {
        match t.kind {
            TokenKind::LParen | TokenKind::LBrace | TokenKind::LBracket => depth += 1,
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => depth -= 1,
            TokenKind::Newline | TokenKind::Semi if depth <= 0 => {
                out.push(&tokens[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&tokens[start..]);
    out
}

/// The import in `statement`, whose token `at` is the macro `name`.
fn import(statement: &[Token], at: usize, token: &Token) -> Result<Import, Diagnostic> {
    let (first, last) = (&statement[0], &statement[statement.len() - 1]);
    shape(
        statement,
        at,
        token.span,
        Span::new(first.span.start, last.span.end),
    )
}

/// A `u_se<` statement must be exactly `"alias:" u_se< "Library"`.
fn shape(
    statement: &[Token],
    at: usize,
    macro_span: Span,
    whole: Span,
) -> Result<Import, Diagnostic> {
    let ends = (
        statement.first().map(|t| &t.kind),
        statement.last().map(|t| &t.kind),
    );
    match (at, statement.len(), ends) {
        (0, _, _) => Err(fail("missing-alias", whole, MISSING)),
        (1, 3, (Some(TokenKind::Str(alias)), Some(TokenKind::Str(spec)))) => {
            valid_alias(alias, statement[0].span)?;
            let (alias, spec) = (alias.clone(), spec.clone());
            Ok(Import {
                alias,
                spec,
                span: whole,
            })
        }
        (1, 3, _) => Err(fail("bad-import", whole, STRINGS)),
        _ => Err(fail("misplaced-macro", macro_span, MISPLACED)),
    }
}
