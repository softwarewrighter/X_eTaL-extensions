//! Numbers as the arithmetic sees them: Bool is 1 / 0 in arithmetic
//! and 1 / 0 is a Bool in a condition (T1).

use xetal_base::{Diagnostic, Span};
use xetal_value::Value;

pub(crate) fn err(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}

#[derive(Clone, Copy)]
pub enum Num {
    I(i64),
    F(f64),
}

impl Num {
    pub fn f(self) -> f64 {
        match self {
            Num::I(i) => i as f64,
            Num::F(x) => x,
        }
    }
}

/// A number, with Bool as 1 / 0 (T1).
pub fn num(v: &Value, span: Span) -> Result<Num, Diagnostic> {
    match v {
        Value::Int(i) => Ok(Num::I(*i)),
        Value::Float(x) => Ok(Num::F(*x)),
        Value::Bool(b) => Ok(Num::I(i64::from(*b))),
        other => Err(err(
            "not-a-number",
            span,
            format!("expected a number, got {other}"),
        )),
    }
}

/// A Bool: true / false, or the Ints 1 / 0; anything else is an error
/// (T1), and so is an array, since a condition is one value (T7).
pub fn truth(v: &Value, span: Span) -> Result<bool, Diagnostic> {
    match v {
        Value::Bool(b) => Ok(*b),
        Value::Int(1) => Ok(true),
        Value::Int(0) => Ok(false),
        Value::Array(_) => Err(err(
            "not-a-scalar",
            span,
            format!("a condition must be a single value, got {v}"),
        )),
        other => Err(err(
            "not-a-bool",
            span,
            format!("expected a Bool (or 1 / 0), got {other}"),
        )),
    }
}

impl<'a> From<Num> for Value<'a> {
    fn from(n: Num) -> Self {
        match n {
            Num::I(i) => Value::Int(i),
            Num::F(x) => Value::Float(x),
        }
    }
}
