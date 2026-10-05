//! Transpose's own axis rule (B17): `o_\_jk x` swaps axes j and k.
//! One digit or three are an error; the axes must exist and differ.

use xetal_base::Diagnostic;
use xetal_transpose::swap_axes;
use xetal_value::{Value, as_array, to_value};

use crate::apply::{axis_error, checked};

/// `o_\_axes x`.
pub(crate) fn transpose_on<'a>(axes: &[u8], x: &Value<'a>) -> Result<Value<'a>, Diagnostic> {
    if axes.len() != 2 {
        let n = axes.len();
        let plural = if n == 1 { "axis" } else { "axes" };
        return Err(axis_error(&format!(
            "o_\\ swaps two axes: write o_\\_jk, got {n} {plural}"
        )));
    }
    let xs = as_array(x);
    let ks = checked(axes, xs.rank())?;
    Ok(match x {
        Value::Array(a) => to_value(swap_axes(a, ks[0] - 1, ks[1] - 1)),
        scalar => scalar.clone(),
    })
}
