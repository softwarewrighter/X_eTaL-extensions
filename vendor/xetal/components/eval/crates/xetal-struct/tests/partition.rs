//! The partition kernel (B14, APL2's partition).

use proptest::prelude::*;
use xetal_array::{Array, ArrayError};
use xetal_struct::partition;

fn v(xs: &[i64]) -> Array<i64> {
    Array::vector(xs.to_vec())
}

#[test]
fn a_new_piece_where_the_key_increases_and_0_drops() {
    let pieces = partition(&[1, 1, 0, 1, 1, 1, 0, 1], &v(&[1, 2, 3, 4, 5, 6, 7, 8])).unwrap();
    assert_eq!(pieces, [v(&[1, 2]), v(&[4, 5, 6]), v(&[8])]);
    let pieces = partition(&[1, 1, 2, 2, 2, 1, 1], &v(&[1, 2, 3, 4, 5, 6, 7])).unwrap();
    assert_eq!(pieces, [v(&[1, 2]), v(&[3, 4, 5, 6, 7])]);
}

#[test]
fn rows_are_the_cells_of_a_matrix() {
    let mat = Array::new(vec![3, 2], vec![1, 2, 3, 4, 5, 6]).unwrap();
    let pieces = partition(&[1, 0, 1], &mat).unwrap();
    let row = |a: i64, b: i64| Array::new(vec![1, 2], vec![a, b]).unwrap();
    assert_eq!(pieces, [row(1, 2), row(5, 6)]);
}

#[test]
fn one_key_per_cell() {
    assert!(matches!(
        partition(&[1], &v(&[1, 2])).unwrap_err(),
        ArrayError::Shape { .. }
    ));
}

proptest! {
    #[test]
    fn pieces_hold_exactly_the_cells_with_a_key(
        pairs in prop::collection::vec((any::<i64>(), 0usize..3), 0..12),
    ) {
        let (items, keys): (Vec<i64>, Vec<usize>) = pairs.into_iter().unzip();
        let pieces = partition(&keys, &Array::vector(items.clone())).unwrap();
        let kept: Vec<i64> = items.iter().zip(&keys).filter(|(_, k)| **k > 0).map(|(x, _)| *x).collect();
        let joined: Vec<i64> = pieces.iter().flat_map(|p| p.data().to_vec()).collect();
        prop_assert_eq!(joined, kept);
        prop_assert!(pieces.iter().all(|p| !p.data().is_empty()));
    }
}
