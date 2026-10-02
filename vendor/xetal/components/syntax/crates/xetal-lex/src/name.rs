//! Names: `[ns:]stem`, where a `_` directly after a letter underlines it
//! and makes the name a function name (N1-N5, D-8, D-9).

use xetal_base::Span;

use crate::cursor::Cursor;
use xetal_token::{ErrorKind, LexError};
use xetal_token::{FuncName, TokenKind, Var};

/// Characters that may end a function name (N3).
const MARKS: &[u8] = b"|-/\\+*<>~!?%$&";

pub(crate) fn lex_name(cur: &mut Cursor) -> Result<TokenKind, LexError> {
    let ns = namespace(cur)?;
    let (stem, underline) = stem(cur)?;
    let kind = match underline {
        Some(underline) => function(cur, ns, stem, underline)?,
        None => variable(cur, ns, stem)?,
    };
    follow(cur, &kind)?;
    Ok(kind)
}

/// `letters:` directly followed by a letter is a namespace prefix.
fn namespace(cur: &mut Cursor) -> Result<Option<String>, LexError> {
    let mut i = 0;
    while cur.peek_at(i).is_some_and(|b| b.is_ascii_alphabetic()) {
        i += 1;
    }
    if cur.peek_at(i) != Some(b':') || cur.peek_at(i + 1) == Some(b'=') {
        return Ok(None);
    }
    let start = cur.pos;
    if !cur.peek_at(i + 1).is_some_and(|b| b.is_ascii_alphabetic()) {
        return Err(LexError::new(
            ErrorKind::BadNamespace,
            Span::new(start, start + i + 1),
            "a namespace prefix must be followed by a name (aliases are written as strings)",
        ));
    }
    let ns = cur.eat_while(|b| b.is_ascii_alphabetic()).to_string();
    cur.pos += 1;
    Ok(Some(ns))
}

/// Letters and digits with at most one `_`, which must follow a letter.
fn stem(cur: &mut Cursor) -> Result<(String, Option<usize>), LexError> {
    let mut stem = String::new();
    let mut underline = None;
    while let Some(b) = cur.peek() {
        if b.is_ascii_alphanumeric() {
            stem.push(char::from(b));
        } else if b == b'_' && underline.is_none() {
            if !cur.prev().is_some_and(|p| p.is_ascii_alphabetic()) {
                return Err(LexError::at(
                    ErrorKind::BadName,
                    cur.pos,
                    "an underline must directly follow a letter",
                ));
            }
            underline = Some(stem.len() - 1);
        } else {
            break;
        }
        cur.pos += 1;
    }
    Ok((stem, underline))
}

fn function(
    cur: &mut Cursor,
    ns: Option<String>,
    stem: String,
    underline: usize,
) -> Result<TokenKind, LexError> {
    let mut mark = None;
    if let Some(b) = cur.peek().filter(|b| MARKS.contains(b)) {
        if b == b'!' && cur.peek_at(1) == Some(b'=') {
            return Err(LexError::bang_equals(cur.pos));
        }
        mark = Some(char::from(b));
        cur.pos += 1;
    }
    let axes = if cur.peek() == Some(b'_') {
        axes(cur)?
    } else {
        Vec::new()
    };
    Ok(TokenKind::Func(FuncName {
        ns,
        stem,
        underline,
        mark,
        axes,
    }))
}

/// `_digits` after a function name: one digit per axis, 1-9, no repeats.
fn axes(cur: &mut Cursor) -> Result<Vec<u8>, LexError> {
    let underscore = cur.pos;
    cur.pos += 1;
    let mut axes = Vec::new();
    while let Some(d @ b'0'..=b'9') = cur.peek() {
        let axis = d - b'0';
        if axis == 0 || axes.contains(&axis) {
            return Err(LexError::at(
                ErrorKind::BadAxis,
                cur.pos,
                "axes are the digits 1-9, each at most once",
            ));
        }
        axes.push(axis);
        cur.pos += 1;
    }
    if axes.is_empty() {
        return Err(LexError::at(
            ErrorKind::BadName,
            underscore,
            "a function name has exactly one underline; a later `_` starts an axis subscript",
        ));
    }
    Ok(axes)
}

fn variable(cur: &mut Cursor, ns: Option<String>, name: String) -> Result<TokenKind, LexError> {
    let mut mutable = false;
    if cur.peek() == Some(b'!') {
        if cur.peek_at(1) == Some(b'=') {
            return Err(LexError::bang_equals(cur.pos));
        }
        mutable = true;
        cur.pos += 1;
    }
    Ok(TokenKind::Var(Var { ns, name, mutable }))
}

/// What may not touch the end of a name.
fn follow(cur: &Cursor, kind: &TokenKind) -> Result<(), LexError> {
    let func = matches!(kind, TokenKind::Func(_));
    let marked = matches!(kind, TokenKind::Func(f) if f.mark.is_some() && f.axes.is_empty());
    match cur.peek() {
        Some(b'"') => Err(LexError::at(
            ErrorKind::BadString,
            cur.pos,
            "a name touching `\"` is reserved (raw strings)",
        )),
        Some(b'@') if func => Err(LexError::at(
            ErrorKind::NoNiladicSugar,
            cur.pos,
            "there is no `_@` sugar: write `n_ow @`",
        )),
        Some(b'_') => Err(LexError::at(
            ErrorKind::BadName,
            cur.pos,
            "a function name has exactly one underline",
        )),
        Some(b) if marked && (b.is_ascii_alphanumeric() || MARKS.contains(&b)) => {
            Err(LexError::at(
                ErrorKind::BadMark,
                cur.pos,
                "a function name ends after one trailing mark",
            ))
        }
        Some(b) if func && b.is_ascii_alphanumeric() => Err(LexError::at(
            ErrorKind::BadName,
            cur.pos,
            "unexpected character after an axis subscript",
        )),
        _ => Ok(()),
    }
}
