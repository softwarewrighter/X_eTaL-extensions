//! Helpers shared by the lexer test files.

use xetal_base::Span;
use xetal_lex::lex;

/// The token dump for `src`, one `start..end Kind` line per token.
pub fn dump(src: &str) -> Vec<String> {
    match lex(src) {
        Ok(tokens) => tokens.iter().map(|t| t.to_string()).collect(),
        Err(e) => panic!("{src:?} should lex, got {e:?}"),
    }
}

/// Token kinds only (no spans).
pub fn kinds(src: &str) -> Vec<String> {
    dump(src)
        .into_iter()
        .map(|line| line.split_once(' ').expect("span").1.to_string())
        .collect()
}

/// The error code and span for a source that must be rejected.
pub fn reject(src: &str) -> (String, Span) {
    match lex(src) {
        Ok(tokens) => panic!("{src:?} should be rejected, lexed as {tokens:?}"),
        Err(e) => (e.code().to_string(), e.span),
    }
}
