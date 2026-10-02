//! Token types and the stable one-line dump format used by `xetal lex`.

use std::fmt;

use xetal_base::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Var(Var),
    Func(FuncName),
    /// `_l` / `_r`; `applied` for `_l_` / `_r_` (apply the value, F5).
    LamArg {
        side: Side,
        applied: bool,
    },
    Num(Number),
    /// A literal exponent touching the value before it (`x^2`).
    Exp(Number),
    Sym(Symbol),
    Str(String),
    Assign,
    Arrow,
    Guard,
    Quote,
    /// `_` touching a closing parenthesis: apply the value (F5).
    Apply,
    Lazy,
    Unit,
    Semi,
    Newline,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
}

/// A variable: letters and digits, optional namespace, optional `!`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub ns: Option<String>,
    pub name: String,
    pub mutable: bool,
}

/// A function name: `[ns:]stem` with exactly one underlined letter,
/// an optional trailing mark and optional axis digits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncName {
    pub ns: Option<String>,
    /// Letters and digits, without the `_`.
    pub stem: String,
    /// Byte index in `stem` of the underlined letter.
    pub underline: usize,
    pub mark: Option<char>,
    /// Axis subscript digits; empty when there is none.
    pub axes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    And,
    Or,
}

impl Symbol {
    pub fn text(self) -> &'static str {
        match self {
            Symbol::Plus => "+",
            Symbol::Minus => "-",
            Symbol::Times => "*",
            Symbol::Divide => "/",
            Symbol::Power => "^",
            Symbol::Eq => "=",
            Symbol::Ne => "!=",
            Symbol::Lt => "<",
            Symbol::Gt => ">",
            Symbol::Le => "<=",
            Symbol::Ge => ">=",
            Symbol::And => "&",
            Symbol::Or => "|",
        }
    }
}

impl FuncName {
    /// The name as written, without namespace or axes (`r_/`, `self_`).
    pub fn spelled(&self) -> String {
        let (before, after) = self.stem.split_at(self.underline + 1);
        let mut out = format!("{before}_{after}");
        out.extend(self.mark);
        out
    }

    /// A function name ending in `<` is a macro (MC2).
    pub fn is_macro(&self) -> bool {
        self.mark == Some('<')
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{} {}", self.span.start, self.span.end, self.kind)
    }
}

fn number_text(n: &Number) -> String {
    match n {
        Number::Int(i) => i.to_string(),
        Number::Float(x) => format!("{x:?}"),
    }
}

/// The system namespace of quad names (`[]N_GET`, QD1-QD3): written
/// as `[]` touching the name, with no colon.
pub const SYSTEM: &str = "[]";

/// A name's namespace prefix as written: `u:`, `[]`, or nothing.
pub fn ns_text(ns: &Option<String>) -> String {
    match ns.as_deref() {
        None => String::new(),
        Some(SYSTEM) => SYSTEM.to_string(),
        Some(ns) => format!("{ns}:"),
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let fixed = match self {
            TokenKind::Var(v) => {
                let bang = if v.mutable { "!" } else { "" };
                return write!(f, "Var({}{}{bang})", ns_text(&v.ns), v.name);
            }
            TokenKind::Func(n) => {
                write!(f, "Func({}{}", ns_text(&n.ns), n.spelled())?;
                if !n.axes.is_empty() {
                    let axes: Vec<String> = n.axes.iter().map(u8::to_string).collect();
                    write!(f, ", axes=[{}]", axes.join(","))?;
                }
                return write!(f, ")");
            }
            TokenKind::LamArg { side, applied } => {
                let s = if *side == Side::Left { "l" } else { "r" };
                let a = if *applied { ", applied" } else { "" };
                return write!(f, "LamArg({s}{a})");
            }
            TokenKind::Num(n) => return write!(f, "Num({})", number_text(n)),
            TokenKind::Exp(n) => return write!(f, "Exp({})", number_text(n)),
            TokenKind::Sym(s) => return write!(f, "Sym({})", s.text()),
            TokenKind::Str(s) => return write!(f, "Str({s:?})"),
            TokenKind::Assign => "Assign",
            TokenKind::Arrow => "Arrow",
            TokenKind::Guard => "Guard",
            TokenKind::Quote => "Quote",
            TokenKind::Apply => "Apply",
            TokenKind::Lazy => "Lazy",
            TokenKind::Unit => "Unit",
            TokenKind::Semi => "Semi",
            TokenKind::Newline => "Newline",
            TokenKind::LParen => "LParen",
            TokenKind::RParen => "RParen",
            TokenKind::LBrace => "LBrace",
            TokenKind::RBrace => "RBrace",
            TokenKind::LBracket => "LBracket",
            TokenKind::RBracket => "RBracket",
        };
        f.write_str(fixed)
    }
}
