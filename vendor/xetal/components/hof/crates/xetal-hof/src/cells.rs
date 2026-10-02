//! Joining major cells back into one value.

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_value::{Value, as_array, to_value};

/// Cells of one `shape` joined along a new leading axis.
pub fn join<'a>(cells: &[Value<'a>], shape: &[usize]) -> Result<Value<'a>, Diagnostic> {
    let mut data = Vec::with_capacity(cells.len() * shape.iter().product::<usize>());
    for c in cells {
        let a = as_array(c);
        if a.shape() != shape {
            return Err(Diagnostic::new(
                "shape-mismatch",
                format!(
                    "each result must be {}, got {}",
                    named(shape),
                    named(a.shape())
                ),
            ));
        }
        data.extend_from_slice(a.data());
    }
    let full = [&[cells.len()], shape].concat();
    Ok(to_value(Array::new(full, data)?))
}

/// `a scalar` or `shape 2 3`.
fn named(shape: &[usize]) -> String {
    match shape {
        [] => "a scalar".into(),
        _ => {
            let dims: Vec<String> = shape.iter().map(ToString::to_string).collect();
            format!("shape {}", dims.join(" "))
        }
    }
}
