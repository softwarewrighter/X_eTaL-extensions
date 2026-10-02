//! Lexer: raw ASCII source to tokens with byte spans, following
//! docs/lang-choices.md. An underlined letter (`_` directly after it)
//! makes a name a function name; a leading `ns:` names its namespace.

mod cursor;
mod exponent;
mod literal;
mod name;
mod quad;
mod scan;

pub use scan::lex;
pub use xetal_token::{
    ErrorKind, FuncName, LexError, Number, SYSTEM, Side, Symbol, Token, TokenKind, Var, ns_text,
};
