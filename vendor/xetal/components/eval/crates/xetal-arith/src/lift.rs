//! Scalar extension (T7): a scalar function applies item by item, and a
//! scalar argument extends to every item of an array argument; two
//! array arguments must have the same shape.

use std::rc::Rc;

use xetal_array::zip;
use xetal_base::{Diagnostic, Span};
use xetal_value::Value;

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// A monadic scalar function over a value.
pub fn lift1<'a>(a: &Value<'a>, f: impl Fn(&Value<'a>) -> Out<'a>) -> Out<'a> {
    match a {
        Value::Array(x) => Ok(Value::Array(Rc::new(x.map(f)?))),
        v => f(v),
    }
}

/// A dyadic scalar function over two values.
pub fn lift2<'a>(
    a: &Value<'a>,
    b: &Value<'a>,
    span: Span,
    f: impl Fn(&Value<'a>, &Value<'a>) -> Out<'a>,
) -> Out<'a> {
    let array = match (a, b) {
        (Value::Array(x), Value::Array(y)) => {
            zip(x, y, &f).map_err(|d: Diagnostic| match d.span {
                Some(_) => d,
                None => d.with_span(span),
            })?
        }
        (Value::Array(x), s) => x.map(|v| f(v, s))?,
        (s, Value::Array(y)) => y.map(|v| f(s, v))?,
        (s, t) => return f(s, t),
    };
    Ok(Value::Array(Rc::new(array)))
}
