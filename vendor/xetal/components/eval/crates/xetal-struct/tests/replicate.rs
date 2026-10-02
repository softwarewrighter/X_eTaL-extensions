//! The replicate kernel (B11): each major cell repeated its count.

use proptest::prelude::*;
use xetal_array::{Array, ArrayError};
use xetal_struct::replicate;

fn v(xs: &[i64]) -> Array<i64> {
    Array::vector(xs.to_vec())
}

#[test]
fn counts_repeat_and_drop_cells() {
    assert_eq!(
        replicate(&[1, 0, 2], &v(&[7, 8, 9])).unwrap(),
        v(&[7, 9, 9])
    );
    assert_eq!(replicate(&[0, 0], &v(&[1, 2])).unwrap(), v(&[]));
}

#[test]
fn rows_are_the_cells_of_a_matrix() {
    let mat = Array::new(vec![2, 2], vec![1, 2, 3, 4]).unwrap();
    let want = Array::new(vec![3, 2], vec![3, 4, 3, 4, 3, 4]).unwrap();
    assert_eq!(replicate(&[0, 3], &mat).unwrap(), want);
}

#[test]
fn one_count_per_cell() {
    assert!(matches!(
        replicate(&[1, 1], &v(&[1, 2, 3])).unwrap_err(),
        ArrayError::Shape { .. }
    ));
}

#[test]
fn too_many_items_is_an_error() {
    let huge = [usize::MAX, 1];
    assert_eq!(
        replicate(&huge, &v(&[1, 2])).unwrap_err(),
        ArrayError::TooLarge
    );
}

proptest! {
    #[test]
    fn length_is_the_sum_of_the_counts(
        pairs in prop::collection::vec((any::<i64>(), 0usize..4), 0..10),
    ) {
        let (items, counts): (Vec<i64>, Vec<usize>) = pairs.into_iter().unzip();
        let out = replicate(&counts, &Array::vector(items)).unwrap();
        prop_assert_eq!(out.shape()[0], counts.iter().sum::<usize>());
    }

    #[test]
    fn ones_keep_every_cell(items in prop::collection::vec(any::<i64>(), 0..10)) {
        let a = Array::vector(items.clone());
        prop_assert_eq!(replicate(&vec![1; items.len()], &a).unwrap(), a);
    }
}
