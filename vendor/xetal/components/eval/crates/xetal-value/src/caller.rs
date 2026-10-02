//! How a built-in applies a function value: the evaluator implements
//! [`Caller`], so higher-order built-ins run operands (symbols, named
//! built-ins, lambdas, user functions) by the ordinary rules.

use xetal_base::{Diagnostic, Span};

use crate::Value;

pub trait Caller<'a> {
    /// `f x`.
    fn call(&mut self, f: &Value<'a>, x: Value<'a>, span: Span) -> Result<Value<'a>, Diagnostic>;

    /// `x f y`: `f` applied to `x`, then the result to `y`.
    fn call2(
        &mut self,
        f: &Value<'a>,
        x: Value<'a>,
        y: Value<'a>,
        span: Span,
    ) -> Result<Value<'a>, Diagnostic> {
        let partial = self.call(f, x, span)?;
        self.call(&partial, y, span)
    }
}
