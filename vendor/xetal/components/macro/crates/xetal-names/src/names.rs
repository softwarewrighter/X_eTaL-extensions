//! Namespaces in one file (MC5, MC6, MC9): a library's `l:` names go
//! to its hidden namespace, an alias to the hidden namespace of the
//! library it names, and a library's unprefixed top-level names to its
//! private namespace unless a lambda parameter or local binding
//! shadows them.

use std::collections::{HashMap, HashSet};
use std::ops::Range;

use xetal_base::{Diagnostic, Span};
use xetal_lex::{Token, TokenKind, lex};

use crate::defs::top_level;
use crate::rename::{name, rename};

/// What a file sees: its own hidden and private namespaces (a library)
/// and, per alias letters, the hidden namespace and exports named.
pub struct Context<'a> {
    pub library: Option<(&'a str, &'a str)>,
    pub aliases: &'a HashMap<String, (String, Vec<String>)>,
    /// Import statements, which are removed rather than rewritten.
    pub imports: &'a [Span],
}

/// A token's replacement text.
pub type Edit = (Range<usize>, String);

/// The edits for `text`, and the names it exports.
pub fn rewrite(text: &str, cx: &Context) -> Result<(Vec<Edit>, Vec<String>), Diagnostic> {
    let Ok(tokens) = lex(text) else {
        return Ok((Vec::new(), Vec::new()));
    };
    let defs = top_level(&tokens, cx)?;
    let mut edits = Vec::new();
    let mut scopes: Vec<HashSet<String>> = vec![HashSet::new()];
    for (i, t) in tokens.iter().enumerate() {
        scope(&tokens, i, &mut scopes);
        let bound = name(t).is_some_and(|(_, k)| scopes[1..].iter().any(|s| s.contains(&k)));
        edits.extend(rename(text, t, cx, &defs.privates, bound)?);
    }
    Ok((edits, defs.exports))
}

/// Enter a lambda (its parameters), leave one, or note a local binding.
fn scope(tokens: &[Token], i: usize, scopes: &mut Vec<HashSet<String>>) {
    match tokens[i].kind {
        TokenKind::LBrace => scopes.push(params(&tokens[i + 1..])),
        TokenKind::RBrace if scopes.len() > 1 => drop(scopes.pop()),
        TokenKind::Assign if scopes.len() > 1 && i > 0 => {
            if let Some((None, key)) = name(&tokens[i - 1]) {
                scopes.last_mut().map(|s| s.insert(key));
            }
        }
        _ => {}
    }
}

/// The parameters of the lambda whose body follows (names before `->`).
fn params(body: &[Token]) -> HashSet<String> {
    let mut depth = 0;
    for (i, t) in body.iter().enumerate() {
        match t.kind {
            TokenKind::LBrace | TokenKind::LParen | TokenKind::LBracket => depth += 1,
            TokenKind::RBrace if depth == 0 => break,
            TokenKind::RBrace | TokenKind::RParen | TokenKind::RBracket => depth -= 1,
            TokenKind::Arrow if depth == 0 => {
                return body[..i]
                    .iter()
                    .filter_map(|t| name(t).map(|(_, k)| k))
                    .collect();
            }
            _ => {}
        }
    }
    HashSet::new()
}
