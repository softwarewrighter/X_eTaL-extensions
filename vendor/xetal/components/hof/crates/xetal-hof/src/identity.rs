//! Operand facts: the identity of reducing an empty axis, and which
//! operands let a scan accumulate in one pass with identical results.

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_value::{Value, as_array, to_value};

/// The identity of `f`, shaped like one major cell (B6): `+ -` 0,
/// `* /` 1 (a Float for `/`), `& = ` true, `| !=` false.
pub(crate) fn identity<'a>(f: &Value<'a>, shape: &[usize]) -> Result<Value<'a>, Diagnostic> {
    let unit = match name(f) {
        Some("+" | "-") => Value::Int(0),
        Some("*") => Value::Int(1),
        Some("/") => Value::Float(1.0),
        Some("&" | "=") => Value::Bool(true),
        Some("|" | "!=") => Value::Bool(false),
        Some(other) => return Err(none(other)),
        None => return Err(none("a function value")),
    };
    let n = shape.iter().product();
    Ok(to_value(Array::new(shape.to_vec(), vec![unit; n])?))
}

fn none(what: &str) -> Diagnostic {
    Diagnostic::new(
        "no-identity",
        format!("{what} has no identity, so it cannot reduce an empty array"),
    )
}

/// A built-in with no arguments yet, by name.
fn name<'a>(f: &'a Value<'_>) -> Option<&'a str> {
    match f {
        Value::Prim(p) if p.args.is_empty() => Some(p.name),
        _ => None,
    }
}

/// Exactly associative on the items of `x`, so a running fold from the
/// left gives the same prefix reductions as folding each prefix from
/// the right: `m_ax m_in & |` always; Int `+` and `*` when no grouping
/// can overflow (Float rounding depends on the grouping, so never).
pub(crate) fn associative(f: &Value<'_>, x: &Value<'_>) -> bool {
    match name(f) {
        Some("m_ax" | "m_in" | "&" | "|") => true,
        Some(op @ ("+" | "*")) => bounded(op, x),
        _ => false,
    }
}

/// Every item is an Int (or Bool) and the sum (`+`) or product (`*`) of
/// their magnitudes (at least 1 each for `*`) fits in an Int.
fn bounded(op: &str, x: &Value<'_>) -> bool {
    let mut total: i64 = i64::from(op == "*");
    for item in as_array(x).data() {
        let m = match item {
            Value::Int(i) => i.checked_abs(),
            Value::Bool(b) => Some(i64::from(*b)),
            _ => None,
        };
        let next = m.and_then(|m| match op {
            "+" => total.checked_add(m),
            _ => total.checked_mul(m.max(1)),
        });
        match next {
            Some(t) => total = t,
            None => return false,
        }
    }
    true
}
