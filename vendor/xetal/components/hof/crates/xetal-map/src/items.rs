//! Results of item-by-item calls: single values only (A7 later), and
//! whether an operand still takes two arguments.

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_core::Kind;
use xetal_value::{Value, to_value};

/// The results as one value; each must be a single, non-function value.
pub(crate) fn finish<'a>(name: &str, results: Array<Value<'a>>) -> Result<Value<'a>, Diagnostic> {
    match results.data().iter().find(|v| !single(v)) {
        Some(bad) => Err(Diagnostic::new(
            "not-a-scalar",
            format!(
                "{name} needs a single value from each call (nested arrays come later), got {bad}"
            ),
        )),
        None => Ok(to_value(results)),
    }
}

fn single(v: &Value<'_>) -> bool {
    !matches!(v, Value::Array(_) | Value::Closure(_) | Value::Prim(_))
}

pub(crate) fn is_function(v: &Value<'_>) -> bool {
    matches!(v, Value::Closure(_) | Value::Prim(_))
}

/// `f` visibly takes two more arguments: a built-in short of two, or a
/// lambda whose body is a lambda. Used only when there is no item to
/// apply it to (an empty array under dyadic each).
pub(crate) fn takes_two(f: &Value<'_>) -> bool {
    match f {
        Value::Prim(p) => p.arity >= p.args.len() + 2,
        Value::Closure(c) => matches!(c.body.kind, Kind::Lam { .. }),
        _ => false,
    }
}
