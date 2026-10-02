//! Dispatch: the higher-order built-ins on runtime values.

use xetal_base::{Diagnostic, Span};
use xetal_value::{Caller, Value};

use crate::fold::{reduce, scan};
use crate::power::power;
use xetal_axes::on_axes;
use xetal_map::{each, inner, map, table, zip};
use xetal_value::as_array;

type Out<'a> = Result<Value<'a>, Diagnostic>;

/// Call the higher-order built-in `name` with its arguments, applying
/// operands through `c`, if it is one.
pub fn call<'a>(
    name: &str,
    args: &[Value<'a>],
    span: Span,
    c: &mut dyn Caller<'a>,
) -> Option<Out<'a>> {
    let result = match (name, args) {
        ("r_/", [f, x]) => reduce(f, x, span, c),
        ("s_\\", [f, x]) => scan(f, x, span, c),
        ("e_ach", [f, x]) => each(f, x, span, c),
        ("#each", [fs, y]) => zip(fs, y, span, c),
        ("m_ap", [f, x]) => map(f, x, span, c),
        ("t_able", [f, x, y]) => table(f, x, y, span, c),
        ("i_nner", [g, f, x, y]) => inner(g, f, x, y, span, c),
        ("c_ompose", [g, f, x]) => c
            .call(g, x.clone(), span)
            .and_then(|gx| c.call(f, gx, span)),
        ("#axes", [spec, f, rest @ ..]) => on_axes(&digits(spec), f, rest, span, c),
        ("s_wap", [f, x, y]) => c.call2(f, y.clone(), x.clone(), span),
        ("p_ower", [f, n, x]) => power(f, n, x, span, c),
        _ => return None,
    };
    Some(result.map_err(|d| match d.span {
        Some(_) => d,
        None => d.with_span(span),
    }))
}

/// The axis digits held by an `#axes` value.
fn digits(spec: &Value<'_>) -> Vec<u8> {
    let digit = |v: &Value<'_>| match v {
        Value::Int(d) => u8::try_from(*d).unwrap_or(0),
        _ => 0,
    };
    as_array(spec).data().iter().map(digit).collect()
}
