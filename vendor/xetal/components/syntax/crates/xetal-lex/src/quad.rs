//! System names (QD1-QD3): `[]` touching a name is one token, the name
//! in the system namespace (written `[]`, drawn as the quad). System
//! names are uppercase, as in APL, so they stand out: `[]N_GET`.

use xetal_base::Span;
use xetal_token::{ErrorKind, LexError, SYSTEM, TokenKind};

use crate::cursor::Cursor;
use crate::name::lex_name;

pub(crate) fn lex_quad(cur: &mut Cursor) -> Result<TokenKind, LexError> {
    let start = cur.pos;
    cur.pos += 2;
    let bad = |end: usize| {
        let message = "a system name is `[]` touching an uppercase name, as in APL: `[]N_GET`";
        LexError::new(ErrorKind::BadName, Span::new(start, end), message)
    };
    if !cur.peek().is_some_and(|b| b.is_ascii_alphabetic()) {
        return Err(bad(cur.pos));
    }
    let mut kind = lex_name(cur)?;
    let upper = |s: &str| !s.chars().any(|c| c.is_ascii_lowercase());
    match &mut kind {
        TokenKind::Func(f) if f.ns.is_none() && upper(&f.stem) => f.ns = Some(SYSTEM.into()),
        TokenKind::Var(v) if v.ns.is_none() && upper(&v.name) => v.ns = Some(SYSTEM.into()),
        _ => return Err(bad(cur.pos)),
    }
    Ok(kind)
}
