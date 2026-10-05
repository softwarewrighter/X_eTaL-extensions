//! Which system built-in a name is.

use xetal_base::{Diagnostic, Span};
use xetal_value::Value;

use crate::draw::{grid, path, show};
use crate::files::{get, put, read};
use crate::text::{format, numbers};

/// Call the system built-in `name` on its arguments, if it is one.
pub fn call<'a>(
    name: &str,
    args: &[Value<'a>],
    span: Span,
) -> Option<Result<Value<'a>, Diagnostic>> {
    let at = |d: Diagnostic| d.with_span(span);
    if let Some(result) = crate::hooks::hook(name, args) {
        return Some(result.map_err(at));
    }
    Some(match (name, args) {
        ("f_ormat", [v]) => Ok(format(v)),
        ("n_umbers", [t]) => numbers(t).map_err(at),
        ("[]N_PUT", [t, path]) => put(t, path).map_err(at),
        ("[]N_GET", [path]) => get(path).map_err(at),
        ("[]R_EAD", [_]) => read().map_err(at),
        ("[]P_ANIC", [t]) => crate::text::chars(t)
            .and_then(|message| Err(Diagnostic::new("panic", message)))
            .map_err(at),
        ("[]E_RR", [t]) => crate::text::chars(t).map_err(at).map(|line| {
            xetal_tty::error(&line);
            t.clone()
        }),
        ("[]K_EY", [_]) => xetal_tty::key()
            .ok()
            .and_then(|k| xetal_value::key_named(&k))
            .map(|i| Value::Tag("Key", i))
            .ok_or_else(|| at(Diagnostic::new("io", "[]K_EY: no key"))),
        ("[]T_E", [_]) => Ok(xetal_value::to_value(xetal_array::Array::vector(
            xetal_tty::facts().iter().map(|i| Value::Int(*i)).collect(),
        ))),
        ("[]G_RID", [a]) => grid(a).map_err(at),
        ("[]P_ATH", [xy]) => path(xy).map_err(at),
        ("[]S_HOW", [svg]) => show(svg).map_err(at),
        _ => return None,
    })
}
