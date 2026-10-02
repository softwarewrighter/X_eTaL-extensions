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
    Some(match (name, args) {
        ("f_ormat", [v]) => Ok(format(v)),
        ("n_umbers", [t]) => numbers(t).map_err(at),
        ("[]N_PUT", [t, path]) => put(t, path).map_err(at),
        ("[]N_GET", [path]) => get(path).map_err(at),
        ("[]R_EAD", [_]) => read().map_err(at),
        ("[]G_RID", [a]) => grid(a).map_err(at),
        ("[]P_ATH", [xy]) => path(xy).map_err(at),
        ("[]S_HOW", [svg]) => show(svg).map_err(at),
        _ => return None,
    })
}
