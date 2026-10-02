//! Literals: numbers `[-]digits[.digits]` and strings `"..."` (ST1,
//! ST2); exponents are in `exponent`.

use xetal_base::Span;

use crate::cursor::Cursor;
use xetal_token::{ErrorKind, LexError};
use xetal_token::{Number, Symbol, TokenKind};

/// Lex a number whose text starts at `start` (a `-` already consumed, if
/// any); the cursor is at the first digit.
pub(crate) fn lex_number(cur: &mut Cursor, start: usize) -> Result<TokenKind, LexError> {
    cur.eat_while(|b| b.is_ascii_digit());
    let mut float = false;
    if cur.peek() == Some(b'.') {
        if !cur.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
            return Err(bad(
                Span::new(start, cur.pos + 1),
                "a `.` must be followed by digits",
            ));
        }
        cur.pos += 1;
        cur.eat_while(|b| b.is_ascii_digit());
        float = true;
    }
    if let Some(b) = cur.peek()
        && (b.is_ascii_alphanumeric() || b"_.@".contains(&b))
    {
        return Err(bad(
            Span::new(cur.pos, cur.pos + 1),
            "a number must be followed by a space, symbol or bracket",
        ));
    }
    value(cur.text(start), Span::new(start, cur.pos), float)
}

fn value(text: &str, span: Span, float: bool) -> Result<TokenKind, LexError> {
    let number = if float {
        text.parse::<f64>().ok().map(Number::Float)
    } else {
        text.parse::<i64>().ok().map(Number::Int)
    };
    number.map(TokenKind::Num).ok_or_else(|| {
        LexError::new(
            ErrorKind::NumberOutOfRange,
            span,
            "integer literal does not fit in 64 bits",
        )
    })
}

fn bad(span: Span, message: &str) -> LexError {
    LexError::new(ErrorKind::BadNumber, span, message)
}

/// A single-line ASCII string with the escapes `\"` `\\` `\n` `\t`.
pub(crate) fn lex_string(cur: &mut Cursor) -> Result<TokenKind, LexError> {
    let start = cur.pos;
    cur.pos += 1;
    let mut text = String::new();
    loop {
        let c = match cur.peek() {
            None | Some(b'\n') => return Err(unterminated(start, cur.pos)),
            Some(b'"') => {
                cur.pos += 1;
                return Ok(TokenKind::Str(text));
            }
            Some(b'\\') => match cur.peek_at(1) {
                Some(b'"') => '"',
                Some(b'\\') => '\\',
                Some(b'n') => '\n',
                Some(b't') => '\t',
                None | Some(b'\n') => return Err(unterminated(start, cur.pos)),
                Some(_) => {
                    return Err(LexError::new(
                        ErrorKind::BadString,
                        Span::new(cur.pos, cur.pos + 2),
                        "unknown escape: use \\\" \\\\ \\n or \\t",
                    ));
                }
            },
            // Unicode is allowed in a string (and a comment), nowhere else.
            Some(b) if !b.is_ascii() => {
                let span = cur.char_span();
                cur.pos = span.end;
                text.push_str(cur.text(span.start));
                continue;
            }
            Some(b) => char::from(b),
        };
        cur.pos += if matches!(cur.peek(), Some(b'\\')) {
            2
        } else {
            1
        };
        text.push(c);
    }
}

fn unterminated(start: usize, pos: usize) -> LexError {
    LexError::new(
        ErrorKind::BadString,
        Span::new(start, pos),
        "unterminated string (strings are single-line; use \\n)",
    )
}

/// `->`, a negative literal (only at a token boundary), or subtract.
pub(crate) fn minus(cur: &mut Cursor) -> Result<TokenKind, LexError> {
    if cur.peek_at(1) == Some(b'>') {
        cur.pos += 2;
        return Ok(TokenKind::Arrow);
    }
    if !cur.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
        cur.pos += 1;
        return Ok(TokenKind::Sym(Symbol::Minus));
    }
    if !cur.prev().is_none_or(|b| b" \t\r\n({[;".contains(&b)) {
        return Err(LexError::new(
            ErrorKind::AmbiguousMinus,
            Span::new(cur.pos, cur.pos + 1),
            "`-` touching the previous token and a digit is ambiguous: \
             write `3 - 1` to subtract or `3 -1` for a negative literal",
        ));
    }
    let start = cur.pos;
    cur.pos += 1;
    lex_number(cur, start)
}
