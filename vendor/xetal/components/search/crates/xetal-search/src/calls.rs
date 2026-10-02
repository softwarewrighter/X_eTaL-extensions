//! Dispatch: the search and order built-ins on runtime values.

use xetal_base::{Diagnostic, Span};
use xetal_value::Value;

use crate::find::{index_of, matches, member, unique};
use crate::order::{grade, sort, where_ones};

/// Call the search or order built-in `name`, if it is one.
pub fn call<'a>(
    name: &str,
    args: &[Value<'a>],
    span: Span,
) -> Option<Result<Value<'a>, Diagnostic>> {
    let result = match (name, args) {
        ("i_ndexOf", [a, b]) => index_of(a, b),
        ("m_ember?", [a, b]) => member(a, b),
        ("m_atch", [a, b]) => matches(a, b),
        ("u_nique", [a]) => unique(a),
        ("s_ort", [a]) => sort(a),
        ("g_rade", [a]) => grade(a),
        ("w_here", [a]) => where_ones(a, span),
        _ => return None,
    };
    Some(result.map_err(|d| match d.span {
        Some(_) => d,
        None => d.with_span(span),
    }))
}
