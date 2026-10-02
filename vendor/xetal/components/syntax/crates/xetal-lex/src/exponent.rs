//! A `^` touching the token before it: a literal exponent on a value
//! (`x^2`, D-1 to D-5) or a power on a function name (`f_^3`, D-7).

use xetal_base::Span;
use xetal_token::{ErrorKind, LexError, Number, TokenKind};

use crate::cursor::Cursor;
use crate::literal::lex_number;

const ON_SYMBOL: &str =
    "a power goes on a function name (`f_^3`); for a symbol write `n '+ p_ower x`";
const COMPUTED: &str =
    "a power count must be a number literal; for a computed count write `n 'f_ p_ower x`";

/// The exponent or power at `^`, given the token it touches.
pub(crate) fn lex_exponent(cur: &mut Cursor, prev: &TokenKind) -> Result<TokenKind, LexError> {
    let caret = cur.pos;
    match prev {
        TokenKind::Func(_) => return function_power(cur),
        TokenKind::Sym(_) => return Err(LexError::at(ErrorKind::BadPower, caret, ON_SYMBOL)),
        TokenKind::Var(_)
        | TokenKind::Num(_)
        | TokenKind::RParen
        | TokenKind::LamArg { applied: false, .. } => {}
        _ => {
            return Err(LexError::at(
                ErrorKind::BadExponent,
                caret,
                "an exponent must touch a number, variable, lambda argument or `)`",
            ));
        }
    }
    cur.pos += 1;
    let start = cur.pos;
    if cur.peek() == Some(b'-') && cur.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
        cur.pos += 1;
    }
    match cur.peek() {
        Some(b'0'..=b'9') => match lex_number(cur, start)? {
            TokenKind::Num(n) => Ok(TokenKind::Exp(n)),
            other => Ok(other),
        },
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => Err(LexError::at(
            ErrorKind::BadExponent,
            cur.pos,
            "an exponent must be a number literal; for a computed power write `x ^ n`",
        )),
        _ => Err(LexError::at(
            ErrorKind::BadExponent,
            caret,
            "`^` touching a value must be followed by a number literal",
        )),
    }
}

/// `f_^n`: a whole number n, 0 or more; `^-1` (the inverse) is reserved.
fn function_power(cur: &mut Cursor) -> Result<TokenKind, LexError> {
    let caret = cur.pos;
    cur.pos += 1;
    let start = cur.pos;
    if cur.peek() == Some(b'-') && cur.peek_at(1).is_some_and(|b| b.is_ascii_digit()) {
        cur.pos += 1;
    }
    let count = match cur.peek() {
        Some(b'0'..=b'9') => lex_number(cur, start)?,
        Some(b) if b.is_ascii_alphabetic() || b == b'_' => {
            return Err(LexError::at(ErrorKind::BadPower, cur.pos, COMPUTED));
        }
        _ => {
            let message = "`^` touching a function name must be followed by a count, as in `f_^3`";
            return Err(LexError::at(ErrorKind::BadPower, caret, message));
        }
    };
    let span = Span::new(caret, cur.pos);
    match count {
        TokenKind::Num(Number::Int(n)) if n >= 0 => Ok(TokenKind::Exp(Number::Int(n))),
        TokenKind::Num(Number::Int(-1)) => Err(LexError::new(
            ErrorKind::ReservedSuperscript,
            span,
            "`^-1`, the inverse of a function, is reserved",
        )),
        _ => Err(LexError::new(
            ErrorKind::BadPower,
            span,
            "a power count is a whole number, 0 or more",
        )),
    }
}
