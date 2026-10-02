//! The evaluator as the callback higher-order built-ins use: operands
//! run by the ordinary application rules (`components/hof`, B6); and
//! functions under an axis subscript (A6), which wait for their
//! arguments as a built-in value before `xetal-axes` applies them.

use std::rc::Rc;
use xetal_base::{Diagnostic, Span};

use xetal_array::Array;
use xetal_core::Kind;
use xetal_value::{Caller, Prim, Slot, Value};

use crate::machine::Machine;
use crate::run::err;

impl<'a> Caller<'a> for Machine<'a, '_> {
    fn call(&mut self, f: &Value<'a>, x: Value<'a>, span: Span) -> Result<Value<'a>, Diagnostic> {
        self.apply(f.clone(), Slot::Value(x), span)
    }
}

impl<'a> Machine<'a, '_> {
    /// `f_axes`: a built-in value `#axes` holding the axes and f, which
    /// takes f's arguments (`arity` from its type, else what f shows).
    pub(crate) fn axes(
        &mut self,
        axes: &[u8],
        arity: Option<usize>,
        f: Value<'a>,
        span: Span,
    ) -> Result<Value<'a>, Diagnostic> {
        let n = arity.unwrap_or_else(|| visible_arity(&f));
        if n == 0 {
            return Err(err(
                "not-a-function",
                span,
                format!("{f} is not a function"),
            ));
        }
        let digits = axes.iter().map(|d| Value::Int(i64::from(*d))).collect();
        Ok(Value::Prim(Rc::new(Prim {
            name: "#axes",
            arity: 2 + n,
            args: vec![Value::Array(Rc::new(Array::vector(digits))), f],
        })))
    }
}

/// The arguments a function value visibly takes: what a built-in still
/// lacks, or a lambda's parameters written together.
fn visible_arity(f: &Value<'_>) -> usize {
    match f {
        Value::Prim(p) => p.arity.saturating_sub(p.args.len()),
        Value::Closure(c) => {
            let (mut n, mut body) = (1, c.body);
            while let Kind::Lam { body: inner, .. } = &body.kind {
                (n, body) = (n + 1, inner);
            }
            n
        }
        _ => 0,
    }
}
