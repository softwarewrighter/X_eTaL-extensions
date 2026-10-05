//! Moving the axes of an array: every element keeps its value, and its
//! index is permuted.

use xetal_array::Array;

/// `a` with axis i moved to position `to[i]` (0-origin; `to` is a
/// permutation of 0..rank, checked by the caller).
pub fn permute<T: Clone>(a: &Array<T>, to: &[usize]) -> Array<T> {
    let rank = a.rank();
    let mut shape = vec![0; rank];
    for (i, &p) in to.iter().enumerate() {
        shape[p] = a.shape()[i];
    }
    let strides = strides(a.shape());
    let mut index = vec![0; rank];
    let mut data = Vec::with_capacity(a.data().len());
    for _ in 0..a.data().len() {
        let at: usize = (0..rank).map(|i| index[to[i]] * strides[i]).sum();
        data.push(a.data()[at].clone());
        step(&mut index, &shape);
    }
    Array::new(shape, data).unwrap_or_else(|_| a.clone())
}

/// The axes in reverse order (APL's monadic transpose).
pub fn reverse_axes<T: Clone>(a: &Array<T>) -> Array<T> {
    let rank = a.rank();
    let to: Vec<usize> = (0..rank).map(|i| rank - 1 - i).collect();
    permute(a, &to)
}

/// Axes `j` and `k` (0-origin, below the rank) exchanged.
pub fn swap_axes<T: Clone>(a: &Array<T>, j: usize, k: usize) -> Array<T> {
    let mut to: Vec<usize> = (0..a.rank()).collect();
    to.swap(j, k);
    permute(a, &to)
}

/// Row-major strides of `shape`.
fn strides(shape: &[usize]) -> Vec<usize> {
    let mut s = vec![1; shape.len()];
    for i in (0..shape.len().saturating_sub(1)).rev() {
        s[i] = s[i + 1] * shape[i + 1];
    }
    s
}

/// The next row-major index within `shape`.
fn step(index: &mut [usize], shape: &[usize]) {
    for i in (0..index.len()).rev() {
        index[i] += 1;
        if index[i] < shape[i] {
            return;
        }
        index[i] = 0;
    }
}
