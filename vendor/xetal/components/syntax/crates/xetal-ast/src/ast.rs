//! The surface AST. Every node carries the span of its source text.

use xetal_base::Span;
use xetal_lex::{FuncName, Number, Side, Symbol, Var};

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Bind {
        target: Target,
        value: Expr,
        span: Span,
    },
    Guard {
        cond: Expr,
        result: Expr,
        span: Span,
    },
    Expr(Expr),
}

/// A name that can be bound: a variable or a function name.
#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    Var(Var),
    Func(FuncName),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: Span,
    /// Nesting depth of this tree (1 for a leaf), bounded by `MAX_DEPTH`.
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Num(Number),
    /// Adjacent numbers (each possibly with an exponent).
    Strand(Vec<Expr>),
    Var(Var),
    Arg(Side),
    Str(String),
    Unit,
    Pow {
        base: Box<Expr>,
        exp: Number,
    },
    /// A function passed as a value (`'f_`).
    Quote(Box<Fun>),
    Monadic {
        f: Box<Fun>,
        arg: Box<Expr>,
    },
    Dyadic {
        left: Box<Expr>,
        f: Box<Fun>,
        right: Box<Expr>,
    },
    /// A function standing alone as a value (a lambda, a train, a name).
    Fn(Box<Fun>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fun {
    pub kind: FunKind,
    pub span: Span,
    /// Nesting depth of this tree (1 for a leaf), bounded by `MAX_DEPTH`.
    pub depth: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FunKind {
    Name(FuncName),
    Sym(Symbol),
    /// `_l_` / `_r_`: apply the argument's value.
    Arg(Side),
    /// `(expr)_`: apply a computed function value.
    Apply(Box<Expr>),
    /// A quoted operand bound to the function on its right (F8).
    Operand {
        operand: Box<Fun>,
        f: Box<Fun>,
    },
    Lambda(Lambda),
    Train(Vec<Fun>),
    /// `f_^3`: a function name applied `count` times (D-7).
    Power {
        f: Box<Fun>,
        count: i64,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lambda {
    pub params: Params,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Params {
    /// Shorthand using `_r` only.
    Right,
    /// Shorthand using `_l` and `_r`.
    LeftRight,
    /// `{ @ -> ... }`.
    Niladic,
    Named(Vec<Param>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: Target,
    pub lazy: bool,
    pub span: Span,
}

/// The deepest tree the parser builds; deeper input is `too-deep`, so
/// every later stage (printers, desugaring, evaluation, dropping the
/// tree) recurses a bounded number of levels.
pub const MAX_DEPTH: u32 = 256;

impl Expr {
    pub fn new(kind: ExprKind, span: Span) -> Self {
        let children = match &kind {
            ExprKind::Strand(items) => items.iter().map(|x| x.depth).max().unwrap_or(0),
            ExprKind::Pow { base, .. } => base.depth,
            ExprKind::Quote(f) | ExprKind::Fn(f) => f.depth,
            ExprKind::Monadic { f, arg } => f.depth.max(arg.depth),
            ExprKind::Dyadic { left, f, right } => left.depth.max(f.depth).max(right.depth),
            _ => 0,
        };
        Expr {
            kind,
            span,
            depth: children + 1,
        }
    }
}

impl Fun {
    pub fn new(kind: FunKind, span: Span) -> Self {
        let children = match &kind {
            FunKind::Apply(e) => e.depth,
            FunKind::Operand { operand, f } => operand.depth.max(f.depth),
            FunKind::Power { f, .. } => f.depth,
            FunKind::Lambda(l) => l
                .body
                .iter()
                .enumerate()
                .map(|(i, s)| i as u32 + stmt_depth(s))
                .max()
                .unwrap_or(0),
            FunKind::Train(fs) => fs.iter().map(|f| f.depth).max().unwrap_or(0),
            _ => 0,
        };
        Fun {
            kind,
            span,
            depth: children + 1,
        }
    }
}

fn stmt_depth(s: &Stmt) -> u32 {
    match s {
        Stmt::Bind { value, .. } => value.depth + 1,
        Stmt::Guard { cond, result, .. } => cond.depth.max(result.depth) + 1,
        Stmt::Expr(e) => e.depth,
    }
}

/// `too-deep` when a tree exceeds `MAX_DEPTH`.
pub fn check_depth(depth: u32, span: Span) -> Result<(), xetal_base::Diagnostic> {
    if depth > MAX_DEPTH {
        return Err(xetal_base::Diagnostic::new(
            "too-deep",
            format!("this expression nests more than {MAX_DEPTH} levels deep"),
        )
        .with_span(span));
    }
    Ok(())
}
