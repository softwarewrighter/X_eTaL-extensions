//! `o_\` and `t_ranspose` on runtime values. A scalar has no axes and
//! is returned as it is.

use xetal_base::{Diagnostic, Span};
use xetal_value::{Value, as_array, to_value};

use crate::{permutation, permute, reverse_axes};

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// Call transpose or permute, if `name` is one of them.
pub fn call<'a>(name: &str, args: &[Value<'a>], span: Span) -> Option<Out<'a>> {
    let result = match (name, args) {
        ("o_\\", [x]) => Ok(match x {
            Value::Array(a) => to_value(reverse_axes(a)),
            scalar => scalar.clone(),
        }),
        ("t_ranspose", [p, x]) => permuted(p, x),
        _ => return None,
    };
    Some(result.map_err(|d| match d.span {
        Some(_) => d,
        None => d.with_span(span),
    }))
}

/// `p t_ranspose x`: axis i of x becomes axis p[i].
fn permuted<'a>(p: &Value<'a>, x: &Value<'a>) -> Out<'a> {
    let axes = as_array(p)
        .data()
        .iter()
        .map(|v| match v {
            Value::Int(i) => Ok(*i),
            other => Err(Diagnostic::new(
                "not-an-integer",
                format!("an axis must be a whole number, got {other}"),
            )),
        })
        .collect::<Result<Vec<i64>, _>>()?;
    let xs = as_array(x);
    let to = permutation(&axes, xs.rank())?;
    Ok(match x {
        Value::Array(a) => to_value(permute(a, &to)),
        scalar => scalar.clone(),
    })
}
