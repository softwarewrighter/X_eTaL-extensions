//! Value helpers for the structural built-ins: integers and fills.

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_value::Value;

pub(crate) use xetal_value::{as_array, as_vector, to_value};

/// Integers (Bool counts as 1 / 0, T1), keeping the shape.
pub(crate) fn ints(v: &Value<'_>) -> Result<Array<i64>, Diagnostic> {
    as_array(v).map(|x| match x {
        Value::Int(i) => Ok(*i),
        Value::Bool(b) => Ok(i64::from(*b)),
        other => Err(Diagnostic::new(
            "not-an-integer",
            format!("expected integers, got {other}"),
        )),
    })
}

/// The fill for padding (B10), taken from the items: 0, 0.0 or a space.
pub(crate) fn fill<'a>(a: &Array<Value<'a>>) -> Option<Value<'a>> {
    Some(match a.data().first()? {
        Value::Int(_) => Value::Int(0),
        Value::Bool(_) => Value::Bool(false),
        Value::Float(_) => Value::Float(0.0),
        Value::Char(_) => Value::Char(' '),
        _ => return None,
    })
}

/// The counts of `r_eplicate` (B11) or keys of `p_artition` (B14), one
/// per each of `cells` major cells: a scalar extends, a vector must
/// match, none may be negative. `of` names the built-in and its unit.
pub(crate) fn counts(
    c: &Value<'_>,
    cells: usize,
    of: (&str, &str),
) -> Result<Vec<usize>, Diagnostic> {
    let (name, unit) = of;
    let c = ints(c)?;
    let each = c
        .data()
        .iter()
        .map(|k| {
            usize::try_from(*k)
                .map_err(|_| Diagnostic::new("domain", format!("a {unit} cannot be {k}")))
        })
        .collect::<Result<Vec<usize>, Diagnostic>>()?;
    match (c.rank(), each.as_slice()) {
        (0, [k]) => Ok(vec![*k; cells]),
        (1, _) if each.len() == cells => Ok(each),
        (1, _) => Err(Diagnostic::new(
            "length-mismatch",
            format!("{} {unit}s for {cells} cells", each.len()),
        )),
        _ => {
            let dims: Vec<String> = c.shape().iter().map(ToString::to_string).collect();
            let message = format!(
                "{name} needs a scalar or a vector of {unit}s, got shape {}",
                dims.join(" ")
            );
            Err(Diagnostic::new("rank", message))
        }
    }
}

/// `d_isclose`: what one box holds (A7); an array of boxes is not
/// opened into one array.
pub(crate) fn disclose<'a>(x: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    match x {
        Value::Boxed(inner) => Ok((**inner).clone()),
        other => {
            let dims: Vec<String> = as_array(other)
                .shape()
                .iter()
                .map(ToString::to_string)
                .collect();
            let message = format!("d_isclose opens one box, got shape {}", dims.join(" "));
            Err(Diagnostic::new("rank", message))
        }
    }
}

/// Each array boxed, as a vector of boxes (a nested vector).
pub(crate) fn boxes<'a>(pieces: Vec<Array<Value<'a>>>) -> Value<'a> {
    let items = pieces
        .into_iter()
        .map(|p| Value::Boxed(std::rc::Rc::new(to_value(p))))
        .collect();
    Value::Array(std::rc::Rc::new(Array::vector(items)))
}
