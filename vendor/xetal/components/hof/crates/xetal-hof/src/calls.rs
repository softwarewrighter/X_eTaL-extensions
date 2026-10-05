//! Dispatch: the higher-order built-ins on runtime values, each a
//! kernel (D50) the evaluator runs call by call.

use xetal_base::{Diagnostic, Span};
use xetal_kernel::{Direct, Kernel, apply, then};
use xetal_value::Value;

use crate::fold::{reduce, scan};
use crate::power::power;
use xetal_axes::on_axes;
use xetal_map::{each, inner, map, table, zip};
use xetal_value::as_array;

type Out<'a> = Result<Kernel<'a, Value<'a>>, Diagnostic>;

/// The higher-order built-ins' names.
const NAMES: [&str; 11] = [
    "r_/", "s_\\", "e_ach", "#each", "m_ap", "t_able", "i_nner", "c_ompose", "#axes", "s_wap",
    "p_ower",
];

/// Whether `name` is a higher-order built-in (one that calls functions).
pub fn higher(name: &str) -> bool {
    NAMES.contains(&name)
}

/// The kernel of the higher-order built-in `name` on its arguments, if
/// it is one; an error without a place is given `span`. Built-in
/// operands the runner calls at once (`direct`) need no kernel calls.
pub fn call<'a>(
    name: &str,
    args: &[Value<'a>],
    span: Span,
    direct: &mut dyn Direct<'a>,
) -> Option<Out<'a>> {
    let result = match (name, args) {
        ("r_/", [f, x]) => reduce(f, x, direct),
        ("s_\\", [f, x]) => scan(f, x),
        ("e_ach", [f, x]) => each(f, x, direct),
        ("#each", [fs, y]) => zip(fs, y),
        ("m_ap", [f, x]) => map(f, x),
        ("t_able", [f, x, y]) => table(f, x, y, direct),
        ("i_nner", [g, f, x, y]) => inner(g, f, x, y, direct),
        ("c_ompose", [g, f, x]) => {
            let f = f.clone();
            Ok(then(apply(g.clone(), vec![x.clone()]), move |gx| {
                Ok(apply(f, vec![gx]))
            }))
        }
        ("#axes", [spec, f, rest @ ..]) => on_axes(&digits(spec), f, rest),
        ("s_wap", [f, x, y]) => Ok(apply(f.clone(), vec![y.clone(), x.clone()])),
        ("p_ower", [f, n, x]) => power(f, n, x),
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
