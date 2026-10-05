//! Which names are local while walking a source's tokens: a lambda's
//! parameters (`{ a b -> ...}`) and the names its body binds (`p :=
//! ...` inside braces) hide everything outside; a top-level `name :=`
//! defines rather than uses.

use std::collections::HashSet;

use xetal_lex::{Token, TokenKind};

use crate::name::{Name, name};

/// One frame per open bracket; a brace's frame holds its local names.
#[derive(Default)]
pub(crate) struct Scopes {
    open: Vec<Option<HashSet<String>>>,
}

impl Scopes {
    /// Token `i` of `tokens`: the name it uses, unless it is a local or
    /// is being bound.
    pub(crate) fn visit(&mut self, tokens: &[Token], i: usize) -> Option<Name> {
        match tokens[i].kind {
            TokenKind::LBrace => self.open.push(Some(params(&tokens[i + 1..]))),
            TokenKind::LParen | TokenKind::LBracket => self.open.push(None),
            TokenKind::RParen | TokenKind::RBrace | TokenKind::RBracket => drop(self.open.pop()),
            _ => {}
        }
        let name = name(&tokens[i].kind)?;
        if binds(tokens, i) {
            if let Some(Some(frame)) = self.open.last_mut() {
                frame.insert(name.key);
            }
            return None;
        }
        let local = name.ns.is_none() && self.open.iter().flatten().any(|f| f.contains(&name.key));
        (!local).then_some(name)
    }
}

/// A lambda's parameters: the names after its `{` up to `->`; none
/// when what follows the brace is not a parameter list.
fn params(after: &[Token]) -> HashSet<String> {
    let mut names = HashSet::new();
    for t in after {
        match &t.kind {
            TokenKind::Arrow => return names,
            TokenKind::Unit | TokenKind::Lazy | TokenKind::Newline => {}
            TokenKind::Var(v) if v.ns.is_none() => drop(names.insert(v.name.clone())),
            TokenKind::Func(f) if f.ns.is_none() => drop(names.insert(f.spelled())),
            _ => break,
        }
    }
    HashSet::new()
}

/// Token `i` is bound: it starts a statement and `:=` follows it.
fn binds(tokens: &[Token], i: usize) -> bool {
    let next = tokens.get(i + 1).map(|t| &t.kind);
    let prev = i.checked_sub(1).map(|p| &tokens[p].kind);
    matches!(next, Some(TokenKind::Assign))
        && matches!(
            prev,
            None | Some(
                TokenKind::Newline | TokenKind::Semi | TokenKind::LBrace | TokenKind::Arrow
            )
        )
}
