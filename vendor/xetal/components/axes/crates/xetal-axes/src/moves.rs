//! Moving one axis of an array to another position.

use xetal_array::Array;

/// `a` with axis `from` moved to position `to` (0-origin; both below
/// the rank), the other axes keeping their order.
pub fn move_axis<T: Clone>(a: &Array<T>, from: usize, to: usize) -> Array<T> {
    let mut order: Vec<usize> = (0..a.rank()).collect();
    let axis = order.remove(from);
    order.insert(to, axis);
    let shape: Vec<usize> = order.iter().map(|&i| a.shape()[i]).collect();
    let mut strides = vec![1; a.rank()];
    for i in (0..a.rank().saturating_sub(1)).rev() {
        strides[i] = strides[i + 1] * a.shape()[i + 1];
    }
    let mut index = vec![0; a.rank()];
    let mut data = Vec::with_capacity(a.data().len());
    for _ in 0..a.data().len() {
        let at: usize = order
            .iter()
            .zip(&index)
            .map(|(&src, &i)| i * strides[src])
            .sum();
        data.push(a.data()[at].clone());
        step(&mut index, &shape);
    }
    Array::new(shape, data).unwrap_or_else(|_| a.clone())
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
