//! Reshape, take and drop.

use xetal_array::{Array, ArrayError, size};

use crate::cells::cell_len;

/// `shape` filled with `items`, reused cyclically (B4); no items for a
/// non-empty shape is an error (B10).
pub fn reshape<T: Clone>(shape: Vec<usize>, items: &[T]) -> Result<Array<T>, ArrayError> {
    let n = size(&shape)?;
    if n > 0 && items.is_empty() {
        return Err(ArrayError::Empty);
    }
    Array::new(shape, items.iter().cycle().take(n).cloned().collect())
}

/// The first `n` major cells (the last `-n` when negative); missing
/// cells are padded with `fill`, and without one that is an error (B10).
pub fn take<T: Clone>(n: i64, a: &Array<T>, fill: Option<T>) -> Result<Array<T>, ArrayError> {
    let (len, cell) = (a.shape()[0], cell_len(a));
    let want = usize::try_from(n.unsigned_abs()).map_err(|_| ArrayError::TooLarge)?;
    let mut shape = a.shape().to_vec();
    shape[0] = want;
    let pad = size(&shape)? - want.min(len) * cell;
    let kept = if n >= 0 {
        &a.data()[..want.min(len) * cell]
    } else {
        &a.data()[(len - want.min(len)) * cell..]
    };
    if pad == 0 {
        return Array::new(shape, kept.to_vec());
    }
    let padding = vec![fill.ok_or(ArrayError::Empty)?; pad];
    let data = if n >= 0 {
        [kept, &padding].concat()
    } else {
        [&padding, kept].concat()
    };
    Array::new(shape, data)
}

/// All but the first `n` major cells (the last `-n` when negative).
pub fn drop<T: Clone>(n: i64, a: &Array<T>) -> Array<T> {
    let (len, cell) = (a.shape()[0], cell_len(a));
    let k = usize::try_from(n.unsigned_abs()).map_or(len, |k| k.min(len));
    let mut shape = a.shape().to_vec();
    shape[0] = len - k;
    let kept = if n >= 0 {
        &a.data()[k * cell..]
    } else {
        &a.data()[..(len - k) * cell]
    };
    Array::new(shape, kept.to_vec()).unwrap_or_else(|_| unreachable!("whole cells"))
}
