//! A value as APL2's DISPLAY draws it (A7): what `xetal_grid::display`
//! needs, built from the value. Nested arrays print this way.

use xetal_array::layout;
use xetal_grid::{Body, Shown};

use crate::{Value, as_array};

/// Whether `v` holds a box anywhere at its top level.
pub fn nested(v: &Value<'_>) -> bool {
    match v {
        Value::Boxed(_) => true,
        Value::Array(a) => a.data().iter().any(|x| matches!(x, Value::Boxed(_))),
        _ => false,
    }
}

/// The picture of a value printed on its own.
pub fn shown(v: &Value<'_>) -> Shown {
    match v {
        Value::Boxed(x) => scalar_frame(x),
        Value::Array(_) => array_frame(v),
        other => Shown::Atom(other.to_string()),
    }
}

/// An item of a nested array: a box holding an array is that array's
/// frame, a box holding a scalar a frame of its own.
fn item(v: &Value<'_>) -> Shown {
    match v {
        Value::Boxed(x) if matches!(**x, Value::Array(_)) => array_frame(x),
        Value::Boxed(x) => scalar_frame(x),
        other => Shown::Atom(other.to_string()),
    }
}

fn scalar_frame(x: &Value<'_>) -> Shown {
    let inner = match x {
        Value::Array(_) => array_frame(x),
        other => item(other),
    };
    let body = match inner {
        Shown::Atom(text) => Body::Text(vec![text]),
        frame => Body::Items(vec![frame]),
    };
    let mark = mark(std::slice::from_ref(x));
    Shown::Frame {
        shape: Vec::new(),
        mark,
        body,
    }
}

fn array_frame(v: &Value<'_>) -> Shown {
    let a = as_array(v);
    let body = if nested(v) {
        Body::Items(a.data().iter().map(item).collect())
    } else {
        let cells: Vec<String> = a.data().iter().map(ToString::to_string).collect();
        let chars = a.data().iter().all(|x| matches!(x, Value::Char(_)));
        let sep = if chars && !cells.is_empty() { "" } else { " " };
        Body::Text(
            layout(a.shape(), &cells, sep)
                .lines()
                .map(String::from)
                .collect(),
        )
    };
    let mark = mark(a.data());
    Shown::Frame {
        shape: a.shape().to_vec(),
        mark,
        body,
    }
}

/// The bottom mark: boxes, characters or numbers.
fn mark(items: &[Value<'_>]) -> char {
    match items {
        _ if items
            .iter()
            .any(|x| matches!(x, Value::Boxed(_) | Value::Array(_))) =>
        {
            '∊'
        }
        [_, ..] if items.iter().all(|x| matches!(x, Value::Char(_))) => '─',
        _ => '~',
    }
}
