//! The Core IR: the small calculus every surface form desugars into.
//! Every node carries a NodeId and the span of the surface construct it
//! came from (for traces and the explainer).

use xetal_base::{NodeId, Span};
use xetal_lex::Number;

/// A lowered program: top-level items in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    /// Notes for errors at particular spans (see [`Program::annotate`]).
    pub notes: Vec<crate::SpanNote>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// A module-level definition (`u:f_ := ...`): defined once per file,
    /// late-bound so definitions may refer to each other.
    Def { name: String, value: Expr },
    /// A top-level variable binding; later items see it (lexical).
    Let {
        name: String,
        rec: bool,
        value: Expr,
    },
    /// Reassigning a mutable `!` variable in place (M2).
    Set { name: String, value: Expr },
    /// An expression whose value is printed.
    Eval(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub id: NodeId,
    pub span: Span,
    pub kind: Kind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Lit(Number),
    Str(String),
    Unit,
    /// A strand: an array of the element values.
    Array(Vec<Expr>),
    /// A local or top-level variable (lexical).
    Var(String),
    /// A namespace-qualified name (`u:s_quare`, `m:pi`).
    Global(String),
    /// A built-in function by its spelling (`+`, `r_/`, `o_-`).
    Prim(String),
    /// A function specialized to axes (A6): `f_12`. `arity` is how many
    /// arguments f takes, the last being the data: filled in from f's
    /// type by elaboration, else found at run time from f's value.
    Axes {
        axes: Vec<u8>,
        arity: Option<usize>,
        f: Box<Expr>,
    },
    Lam {
        param: Param,
        lazy: bool,
        body: Box<Expr>,
    },
    /// Monadic application `f x`.
    App(Box<Expr>, Box<Expr>),
    /// Dyadic application `x f y`: means `App(App(f, x), y)`, evaluated
    /// function first, then the right argument, then the left (E4).
    App2 {
        f: Box<Expr>,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Let {
        name: String,
        rec: bool,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    Set {
        name: String,
        value: Box<Expr>,
        body: Box<Expr>,
    },
    /// A guard: `cond ? then`, otherwise `other`.
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        other: Box<Expr>,
    },
    /// No guard matched and no statement followed (G2): a runtime error.
    NoMatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Param {
    Name(String),
    /// `{ @ -> ... }`: the argument must be Unit.
    Unit,
}
