//! A runtime value as a display grid (`xetal-grid`): element type,
//! shape and formatted items.

use xetal_grid::Grid;

use crate::{Value, as_array};

/// The type name of a scalar item.
fn kind(v: &Value<'_>) -> &'static str {
    match v {
        Value::Int(_) => "Int",
        Value::Float(_) => "Float",
        Value::Bool(_) => "Bool",
        Value::Char(_) => "Char",
        Value::Unit => "Unit",
        Value::Array(_) => "Array",
        Value::Boxed(_) => "Box",
        Value::Closure(_) | Value::Prim(_) => "function",
    }
}

/// `v` for display.
pub fn grid(v: &Value<'_>) -> Grid {
    let a = as_array(v);
    Grid {
        kind: a.data().first().map_or("", kind).to_string(),
        shape: a.shape().to_vec(),
        items: a.data().iter().map(ToString::to_string).collect(),
    }
}
