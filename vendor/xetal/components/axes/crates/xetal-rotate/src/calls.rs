//! `o_-` and `r_ev` on runtime values. A scalar has no axis to turn and
//! is returned as it is.

use xetal_array::Array;
use xetal_base::{Diagnostic, Span};
use xetal_value::{Value, as_array, to_value};

use crate::{reverse, rotate};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// Call rotate or reverse, if `name` is one of them.
pub fn call<'a>(name: &str, args: &[Value<'a>], span: Span) -> Option<Out<'a>> {
    let result = match (name, args) {
        ("r_ev", [x]) => Ok(on_array(x, reverse)),
        ("o_-", [n, x]) => rotations(n, x),
        _ => return None,
    };
    Some(result.map_err(|d| match d.span {
        Some(_) => d,
        None => d.with_span(span),
    }))
}

fn on_array<'a>(x: &Value<'a>, f: impl Fn(&Array<Value<'a>>) -> Array<Value<'a>>) -> Value<'a> {
    match x {
        Value::Array(a) => to_value(f(a)),
        scalar => scalar.clone(),
    }
}

/// One amount rotates once; a list of amounts gives every rotation,
/// stacked along a new leading axis (A4).
fn rotations<'a>(n: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let amounts = as_array(n);
    let counts = amounts
        .data()
        .iter()
        .map(|v| match v {
            Value::Int(i) => Ok(*i),
            other => Err(Diagnostic::new(
                "not-an-integer",
                format!("a rotation amount must be a whole number, got {other}"),
            )),
        })
        .collect::<Result<Vec<i64>, _>>()?;
    match amounts.rank() {
        0 => Ok(on_array(x, |a| rotate(counts[0], a))),
        1 => {
            let xs = as_array(x);
            let data = counts
                .iter()
                .flat_map(|k| as_array(&on_array(x, |a| rotate(*k, a))).data().to_vec())
                .collect();
            Ok(to_value(Array::new(
                [&[counts.len()], xs.shape()].concat(),
                data,
            )?))
        }
        _ => Err(Diagnostic::new(
            "rank",
            "rotation amounts must be one number or a list",
        )),
    }
}
