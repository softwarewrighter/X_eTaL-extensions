//! Rotate and reverse the major cells of an array (rank 1 or more).

use xetal_array::Array;

fn cell_len<T>(a: &Array<T>) -> usize {
    a.shape()[1..].iter().product()
}

/// Turn the major cells by `n`: a positive amount moves cells toward
/// the front (APL, J, BQN), and amounts wrap around.
pub fn rotate<T: Clone>(n: i64, a: &Array<T>) -> Array<T> {
    let (len, cell) = (a.shape()[0], cell_len(a));
    if len == 0 {
        return a.clone();
    }
    let start = n.rem_euclid(len as i64) as usize * cell;
    let data = [&a.data()[start..], &a.data()[..start]].concat();
    Array::new(a.shape().to_vec(), data).unwrap_or_else(|_| a.clone())
}

/// The major cells in reverse order.
pub fn reverse<T: Clone>(a: &Array<T>) -> Array<T> {
    let cell = cell_len(a).max(1);
    let data: Vec<T> = match cell_len(a) {
        0 => a.data().to_vec(),
        _ => a.data().chunks(cell).rev().flatten().cloned().collect(),
    };
    Array::new(a.shape().to_vec(), data).unwrap_or_else(|_| a.clone())
}
