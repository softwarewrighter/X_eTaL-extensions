//! Tokens and lexer errors: what the lexer (`xetal-lex`, which
//! re-exports them) produces from raw ASCII source.

mod error;
mod token;

pub use error::{ErrorKind, LexError};
pub use token::{FuncName, Number, SYSTEM, Side, Symbol, Token, TokenKind, Var, ns_text};
