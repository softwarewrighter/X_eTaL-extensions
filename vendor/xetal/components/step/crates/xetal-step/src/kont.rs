//! The machine's state as data: what to do next ([`Control`]) and the
//! stack of pending work, each frame waiting for a value ([`Kont`]).

use std::cell::RefCell;
use std::rc::Rc;

use xetal_base::{Diagnostic, Span};
use xetal_core::Expr;
use xetal_value::{Env, Prim, Slot, Value};

/// Evaluate an expression, or give a value to the frame on top.
pub(crate) enum Control<'a> {
    Eval(&'a Expr, Env<'a>),
    Return(Value<'a>),
}

/// A pending piece of work, waiting for the value of what runs above it.
pub(crate) enum Kont<'a> {
    /// The program's items from `next` on.
    Items { next: usize },
    /// A definition's value, to store under `name`.
    Def { name: &'a str },
    /// A top-level expression's value, to print (or keep).
    Show { span: Span },
    /// A binding's value; then `body`, or at the top level the next item.
    Bind {
        name: &'a str,
        env: Env<'a>,
        inner: Option<Env<'a>>,
        body: Option<&'a Expr>,
    },
    /// A `!` variable's new value; then `body`, or the next item.
    Assign {
        slot: Rc<RefCell<Slot<'a>>>,
        env: Env<'a>,
        body: Option<&'a Expr>,
    },
    /// A guard's condition; then one branch.
    Choose {
        then: &'a Expr,
        other: &'a Expr,
        env: Env<'a>,
        span: Span,
    },
    /// An array's items, right to left: `left` still to evaluate.
    Array {
        items: &'a [Expr],
        left: usize,
        done: Vec<Value<'a>>,
        env: Env<'a>,
    },
    /// The function under an axis subscript.
    Axes {
        axes: &'a [u8],
        arity: Option<usize>,
        span: Span,
    },
    /// `f x`: the function's value, then its argument.
    Arg {
        x: &'a Expr,
        env: Env<'a>,
        span: Span,
    },
    /// `f x`: the argument's value, for `f`.
    Call { f: Value<'a>, span: Span },
    /// `x f y`: the function's value, then y, then x.
    Pair {
        l: &'a Expr,
        r: &'a Expr,
        env: Env<'a>,
        span: Span,
    },
    /// `x f y`: y's value; x next.
    PairRight {
        f: Value<'a>,
        l: &'a Expr,
        env: Env<'a>,
        span: Span,
    },
    /// `x f y`: x's value; then f x, then that applied to y.
    PairLeft {
        f: Value<'a>,
        right: Slot<'a>,
        span: Span,
    },
    /// A function's value, to apply to `arg`.
    ApplyTo { arg: Slot<'a>, span: Span },
    /// A lazy argument's value, stored in its slot (E1).
    Force {
        slot: Rc<RefCell<Slot<'a>>>,
        expr: &'a Expr,
        env: Env<'a>,
    },
    /// A built-in's last argument, forced.
    PrimArg { p: Rc<Prim<'a>>, span: Span },
    /// A niladic function's argument, which must be `@`.
    UnitBody {
        body: &'a Expr,
        env: Env<'a>,
        span: Span,
    },
    /// A function call in progress (the depth a runaway recursion grows).
    Called,
    /// A higher-order built-in waiting for the call it asked for.
    Kernel {
        kernel: xetal_kernel::Kernel<'a, Value<'a>>,
        span: Span,
    },
}

pub(crate) fn err(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}
