//! Item-wise operations: map, and zip over arrays of the same shape.

use crate::{Array, ArrayError};

impl<T> Array<T> {
    /// Apply `f` to every item; the shape is kept.
    pub fn map<U, E>(&self, f: impl FnMut(&T) -> Result<U, E>) -> Result<Array<U>, E> {
        let data = self.data.iter().map(f).collect::<Result<_, _>>()?;
        Ok(Array {
            shape: self.shape.clone(),
            data,
        })
    }
}

/// Combine two arrays of the same shape item by item.
pub fn zip<T, U, E: From<ArrayError>>(
    a: &Array<T>,
    b: &Array<T>,
    mut f: impl FnMut(&T, &T) -> Result<U, E>,
) -> Result<Array<U>, E> {
    if a.shape != b.shape {
        return Err(ArrayError::Shape {
            left: a.shape.clone(),
            right: b.shape.clone(),
        }
        .into());
    }
    let data = a
        .data
        .iter()
        .zip(&b.data)
        .map(|(x, y)| f(x, y))
        .collect::<Result<_, _>>()?;
    Ok(Array {
        shape: a.shape.clone(),
        data,
    })
}
