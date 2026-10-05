//! The two kinds of number decode folds: Ints, with overflow checked,
//! and Floats (B18).

use xetal_base::Diagnostic;
use xetal_value::Value;

use crate::number::{Number, overflow};

impl Number for i64 {
    const ZERO: i64 = 0;
    const WHAT: &'static str = "integers";
    fn read(v: &Value<'_>) -> Option<i64> {
        match v {
            Value::Int(i) => Some(*i),
            Value::Bool(b) => Some(i64::from(*b)),
            _ => None,
        }
    }
    fn value<'a>(self) -> Value<'a> {
        Value::Int(self)
    }
    fn horner(n: i64, r: i64, d: i64) -> Result<i64, Diagnostic> {
        n.checked_mul(r)
            .and_then(|n| n.checked_add(d))
            .ok_or_else(overflow)
    }
}

impl Number for f64 {
    const ZERO: f64 = 0.0;
    const WHAT: &'static str = "numbers";
    fn read(v: &Value<'_>) -> Option<f64> {
        match v {
            Value::Float(x) => Some(*x),
            _ => None,
        }
    }
    fn value<'a>(self) -> Value<'a> {
        Value::Float(self)
    }
    fn horner(n: f64, r: f64, d: f64) -> Result<f64, Diagnostic> {
        Ok(n.mul_add(r, d))
    }
}
