//! Rotate along listed axes (A4): one amount turns every listed axis;
//! a list of amounts gives every combination, one leading result axis
//! per listed axis.

use xetal_array::Array;
use xetal_base::Diagnostic;
use xetal_rotate::rotate;
use xetal_value::{Value, as_array, to_value};

use crate::apply::{axis_error, checked};
use crate::move_axis;

/// `n o_-_axes x`.
pub(crate) fn rotate_on<'a>(
    axes: &[u8],
    n: &Value<'a>,
    x: &Value<'a>,
) -> Result<Value<'a>, Diagnostic> {
    let (amounts, xs) = (as_array(n), as_array(x));
    let axes = checked(axes, xs.rank())?;
    let counts = amounts.map(|v| match v {
        Value::Int(i) => Ok(*i),
        other => Err(axis_error(&format!(
            "a rotation amount must be a whole number, got {other}"
        ))),
    })?;
    if counts.rank() > 1 {
        return Err(Diagnostic::new(
            "rank",
            "rotation amounts must be one number or a list",
        ));
    }
    let lists: Vec<i64> = counts.data().to_vec();
    let combos = match counts.rank() {
        0 => vec![vec![lists[0]; axes.len()]],
        _ => combinations(&lists, axes.len()),
    };
    let mut data = Vec::with_capacity(combos.len() * xs.data().len());
    for combo in &combos {
        let turned = axes
            .iter()
            .zip(combo)
            .fold(xs.clone(), |a, (k, n)| along(&a, *k, *n));
        data.extend_from_slice(turned.data());
    }
    let lead = match counts.rank() {
        0 => Vec::new(),
        _ => vec![lists.len(); axes.len()],
    };
    Ok(to_value(Array::new(
        [&lead[..], xs.shape()].concat(),
        data,
    )?))
}

/// Rotate `a` by `n` along 1-origin axis `k`.
fn along<T: Clone>(a: &Array<T>, k: usize, n: i64) -> Array<T> {
    if a.rank() == 0 {
        return a.clone();
    }
    move_axis(&rotate(n, &move_axis(a, k - 1, 0)), 0, k - 1)
}

/// Every tuple of `len` items from `xs`, the last varying fastest.
fn combinations(xs: &[i64], len: usize) -> Vec<Vec<i64>> {
    (0..len).fold(vec![Vec::new()], |acc, _| {
        acc.iter()
            .flat_map(|t| xs.iter().map(move |x| [&t[..], &[*x]].concat()))
            .collect()
    })
}
