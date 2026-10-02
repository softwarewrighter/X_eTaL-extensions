//! The main scanning loop: whitespace and comments, punctuation, the
//! negative-literal rule, symbols and markers; names and literals are
//! delegated.

use xetal_base::Span;

use crate::cursor::Cursor;
use crate::{exponent, literal, name, quad};
use xetal_token::{ErrorKind, LexError};
use xetal_token::{Side, Symbol, Token, TokenKind};

/// Lex `src` into tokens. Spaces, tabs, carriage returns and `#`
/// comments separate tokens; each `\n` is a `Newline` token.
pub fn lex(src: &str) -> Result<Vec<Token>, LexError> {
    let mut cur = Cursor::new(src);
    let mut tokens: Vec<Token> = Vec::new();
    loop {
        cur.eat_while(|b| matches!(b, b' ' | b'\t' | b'\r'));
        if cur.peek() == Some(b'#') {
            cur.eat_while(|b| b != b'\n');
            continue;
        }
        let start = cur.pos;
        let Some(byte) = cur.peek() else {
            return Ok(tokens);
        };
        let touching = tokens
            .last()
            .filter(|t| t.span.end == start)
            .map(|t| &t.kind);
        let kind = next_token(&mut cur, byte, touching)?;
        tokens.push(Token {
            kind,
            span: Span::new(start, cur.pos),
        });
    }
}

/// `touching` is the previous token when no whitespace separates it.
fn next_token(
    cur: &mut Cursor,
    byte: u8,
    touching: Option<&TokenKind>,
) -> Result<TokenKind, LexError> {
    if byte == b'[' && cur.peek_at(1) == Some(b']') {
        return quad::lex_quad(cur);
    }
    if let Some(kind) = punct(byte) {
        cur.pos += 1;
        return Ok(kind);
    }
    match (byte, touching) {
        (b'^', Some(prev)) => exponent::lex_exponent(cur, prev),
        (b'0'..=b'9', _) => literal::lex_number(cur, cur.pos),
        (b'"', _) => literal::lex_string(cur),
        (b'-', _) => literal::minus(cur),
        (b'_', Some(TokenKind::RParen)) => lambda_arg(cur, true),
        (b'_', _) => lambda_arg(cur, false),
        (b'\'' | b'~' | b'?' | b':' | b'!', _) => marker(cur, byte),
        (b'+' | b'*' | b'/' | b'^' | b'=' | b'&' | b'|' | b'<' | b'>', _) => Ok(symbol(cur, byte)),
        (b, _) if b.is_ascii_alphabetic() => name::lex_name(cur),
        _ => Err(unexpected(cur)),
    }
}

fn punct(byte: u8) -> Option<TokenKind> {
    Some(match byte {
        b'\n' => TokenKind::Newline,
        b';' => TokenKind::Semi,
        b'@' => TokenKind::Unit,
        b'(' => TokenKind::LParen,
        b')' => TokenKind::RParen,
        b'{' => TokenKind::LBrace,
        b'}' => TokenKind::RBrace,
        b'[' => TokenKind::LBracket,
        b']' => TokenKind::RBracket,
        _ => return None,
    })
}

/// `_l`, `_r`, the applied forms `_l_`, `_r_`, and (after `)`) a lone
/// `_` that applies the parenthesized value.
fn lambda_arg(cur: &mut Cursor, after_paren: bool) -> Result<TokenKind, LexError> {
    let start = cur.pos;
    let next = cur.peek_at(1);
    if after_paren && !next.is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@') {
        cur.pos += 1;
        return Ok(TokenKind::Apply);
    }
    let side = if cur.peek_at(1) == Some(b'l') {
        Side::Left
    } else {
        Side::Right
    };
    let named = matches!(cur.peek_at(1), Some(b'l' | b'r'));
    cur.pos += if named { 2 } else { 1 };
    let applied = named && cur.peek() == Some(b'_');
    if applied {
        cur.pos += 1;
    }
    if !named
        || cur
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@')
    {
        cur.eat_while(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'@');
        return Err(LexError::new(
            ErrorKind::BadLambdaArg,
            Span::new(start, cur.pos),
            "lambda arguments are `_l` and `_r` (and `_l_` / `_r_` to apply them)",
        ));
    }
    Ok(TokenKind::LamArg { side, applied })
}

/// Quote, lazy marker, guard, `:=` and `!=` (each with its spacing rule).
fn marker(cur: &mut Cursor, byte: u8) -> Result<TokenKind, LexError> {
    let next = cur.peek_at(1);
    let (kind, len, error) = match byte {
        b'\'' if next.is_some_and(|b| b.is_ascii_alphabetic() || b"{[+-*/^=!<>&|".contains(&b)) => {
            (TokenKind::Quote, 1, None)
        }
        b'\'' => (
            TokenKind::Quote,
            1,
            Some((
                ErrorKind::BadQuote,
                "a quote must touch a function name, `{`, `[` or a symbol",
            )),
        ),
        b'~' if next.is_some_and(|b| b.is_ascii_alphabetic()) => (TokenKind::Lazy, 1, None),
        b'~' => (
            TokenKind::Lazy,
            1,
            Some((
                ErrorKind::BadLazy,
                "`~` marks a lazy parameter and must touch its name",
            )),
        ),
        b'?' if cur.prev().is_none_or(|b| b" \t\r\n".contains(&b)) => (TokenKind::Guard, 1, None),
        b'?' => (
            TokenKind::Guard,
            1,
            Some((ErrorKind::BadGuard, "the guard `?` needs a space before it")),
        ),
        b':' if next == Some(b'=') => (TokenKind::Assign, 2, None),
        b'!' if next == Some(b'=') => (TokenKind::Sym(Symbol::Ne), 2, None),
        _ => (
            TokenKind::Unit,
            1,
            Some((ErrorKind::UnexpectedChar, "unexpected character")),
        ),
    };
    if let Some((kind, message)) = error {
        return Err(LexError::at(kind, cur.pos, message));
    }
    cur.pos += len;
    Ok(kind)
}

fn symbol(cur: &mut Cursor, byte: u8) -> TokenKind {
    let equals = cur.peek_at(1) == Some(b'=');
    let (sym, len) = match byte {
        b'+' => (Symbol::Plus, 1),
        b'*' => (Symbol::Times, 1),
        b'/' => (Symbol::Divide, 1),
        b'^' => (Symbol::Power, 1),
        b'=' => (Symbol::Eq, 1),
        b'&' => (Symbol::And, 1),
        b'|' => (Symbol::Or, 1),
        b'<' if equals => (Symbol::Le, 2),
        b'<' => (Symbol::Lt, 1),
        b'>' if equals => (Symbol::Ge, 2),
        _ => (Symbol::Gt, 1),
    };
    cur.pos += len;
    TokenKind::Sym(sym)
}

fn unexpected(cur: &Cursor) -> LexError {
    let span = cur.char_span();
    if cur.peek().is_some_and(|b| !b.is_ascii()) {
        LexError::new(
            ErrorKind::NonAscii,
            span,
            "source is ASCII; Unicode appears only in rendered output",
        )
    } else {
        LexError::new(ErrorKind::UnexpectedChar, span, "unexpected character")
    }
}
