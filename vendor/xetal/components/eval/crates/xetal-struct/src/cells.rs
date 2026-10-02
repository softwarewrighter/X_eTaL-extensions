//! Major cells: select, first, replicate, and joining along the
//! leading axis.

use xetal_array::{Array, ArrayError, size};

/// Items in one major cell of `a` (rank 1 or more).
pub(crate) fn cell_len<T>(a: &Array<T>) -> usize {
    a.shape()[1..].iter().product()
}

/// The major cells of `a` at 1-origin `indices`; the result has the
/// shape of `indices` followed by the cell shape.
pub fn select<T: Clone>(indices: &Array<i64>, a: &Array<T>) -> Result<Array<T>, ArrayError> {
    let (len, cell) = (a.shape()[0], cell_len(a));
    let mut data = Vec::with_capacity(indices.data().len() * cell);
    for &index in indices.data() {
        let at = usize::try_from(index)
            .ok()
            .filter(|i| (1..=len).contains(i))
            .ok_or(ArrayError::Index { index, len })?;
        data.extend_from_slice(&a.data()[(at - 1) * cell..at * cell]);
    }
    Array::new([indices.shape(), &a.shape()[1..]].concat(), data)
}

/// Each major cell of `a` repeated `counts` times, in order (B11);
/// one count per cell.
pub fn replicate<T: Clone>(counts: &[usize], a: &Array<T>) -> Result<Array<T>, ArrayError> {
    let (len, cell) = (a.shape()[0], cell_len(a));
    if counts.len() != len {
        return Err(ArrayError::Shape {
            left: vec![counts.len()],
            right: a.shape().to_vec(),
        });
    }
    let mut shape = a.shape().to_vec();
    shape[0] = counts
        .iter()
        .try_fold(0usize, |n, k| n.checked_add(*k))
        .ok_or(ArrayError::TooLarge)?;
    size(&shape)?;
    let mut data = Vec::new();
    for (i, k) in counts.iter().enumerate() {
        for _ in 0..*k {
            data.extend_from_slice(&a.data()[i * cell..(i + 1) * cell]);
        }
    }
    Array::new(shape, data)
}

/// The first major cell; an empty array has none (no fill, B10).
pub fn first<T: Clone>(a: &Array<T>) -> Result<Array<T>, ArrayError> {
    if a.shape()[0] == 0 {
        return Err(ArrayError::Empty);
    }
    Array::new(a.shape()[1..].to_vec(), a.data()[..cell_len(a)].to_vec())
}

/// Join along the leading axis (B10): an argument one rank lower than
/// the other is a single major cell; the cell shapes must match.
pub fn cat<T: Clone>(a: &Array<T>, b: &Array<T>) -> Result<Array<T>, ArrayError> {
    let rank = a.rank().max(b.rank()).max(1);
    let cells = |x: &Array<T>| match rank - x.rank() {
        0 => Some((x.shape()[0], x.shape()[1..].to_vec())),
        1 => Some((1, x.shape().to_vec())),
        _ => None,
    };
    match (cells(a), cells(b)) {
        (Some((na, ca)), Some((nb, cb))) if ca == cb => Array::new(
            [&[na + nb], &ca[..]].concat(),
            [a.data(), b.data()].concat(),
        ),
        _ => Err(ArrayError::Shape {
            left: a.shape().to_vec(),
            right: b.shape().to_vec(),
        }),
    }
}
