//! Structural kernels on generic arrays (B4, B10).

use proptest::prelude::*;
use xetal_array::{Array, ArrayError};
use xetal_struct::{cat, drop, first, reshape, select, take};

fn v(xs: &[i64]) -> Array<i64> {
    Array::vector(xs.to_vec())
}

fn m(shape: &[usize], xs: &[i64]) -> Array<i64> {
    Array::new(shape.to_vec(), xs.to_vec()).unwrap()
}

#[test]
fn reshape_reuses_items_cyclically() {
    assert_eq!(
        reshape(vec![2, 2], &[1, 2, 3]).unwrap(),
        m(&[2, 2], &[1, 2, 3, 1])
    );
    assert_eq!(reshape(vec![0], &[1]).unwrap(), m(&[0], &[]));
    assert_eq!(
        reshape(vec![3], &[] as &[i64]).unwrap_err(),
        ArrayError::Empty
    );
    assert_eq!(reshape(vec![], &[9, 8]).unwrap(), m(&[], &[9]));
}

#[test]
fn take_and_drop_along_the_leading_axis() {
    assert_eq!(take(2, &v(&[1, 2, 3]), None).unwrap(), v(&[1, 2]));
    assert_eq!(take(-2, &v(&[1, 2, 3]), None).unwrap(), v(&[2, 3]));
    assert_eq!(take(4, &v(&[1, 2]), Some(0)).unwrap(), v(&[1, 2, 0, 0]));
    assert_eq!(take(-3, &v(&[1]), Some(0)).unwrap(), v(&[0, 0, 1]));
    assert_eq!(take(2, &v(&[1]), None).unwrap_err(), ArrayError::Empty);
    let mat = m(&[2, 2], &[1, 2, 3, 4]);
    assert_eq!(
        take(3, &mat, Some(0)).unwrap(),
        m(&[3, 2], &[1, 2, 3, 4, 0, 0])
    );
    assert_eq!(drop(1, &mat), m(&[1, 2], &[3, 4]));
    assert_eq!(drop(-1, &v(&[1, 2, 3])), v(&[1, 2]));
    assert_eq!(drop(5, &v(&[1, 2, 3])), v(&[]));
}

#[test]
fn select_and_first_take_major_cells() {
    let mat = m(&[2, 3], &[1, 2, 3, 4, 5, 6]);
    assert_eq!(select(&v(&[2]), &mat).unwrap(), m(&[1, 3], &[4, 5, 6]));
    assert_eq!(select(&m(&[], &[2]), &mat).unwrap(), v(&[4, 5, 6]));
    assert_eq!(
        select(&v(&[3, 1]), &v(&[10, 20, 30])).unwrap(),
        v(&[30, 10])
    );
    assert_eq!(
        select(&v(&[4]), &v(&[10, 20, 30])).unwrap_err(),
        ArrayError::Index { index: 4, len: 3 }
    );
    assert_eq!(first(&mat).unwrap(), v(&[1, 2, 3]));
    assert_eq!(first(&v(&[7, 8])).unwrap(), m(&[], &[7]));
    assert_eq!(first(&v(&[])).unwrap_err(), ArrayError::Empty);
}

#[test]
fn cat_joins_major_cells() {
    assert_eq!(cat(&v(&[1, 2]), &v(&[3])).unwrap(), v(&[1, 2, 3]));
    let mat = m(&[2, 2], &[1, 2, 3, 4]);
    assert_eq!(
        cat(&mat, &v(&[5, 6])).unwrap(),
        m(&[3, 2], &[1, 2, 3, 4, 5, 6])
    );
    assert_eq!(
        cat(&v(&[0, 9]), &mat).unwrap(),
        m(&[3, 2], &[0, 9, 1, 2, 3, 4])
    );
    assert_eq!(cat(&m(&[], &[0]), &v(&[1])).unwrap(), v(&[0, 1]));
    assert!(matches!(
        cat(&mat, &v(&[5, 6, 7])).unwrap_err(),
        ArrayError::Shape { .. }
    ));
}

proptest! {
    #[test]
    fn reshape_has_the_requested_shape(
        shape in prop::collection::vec(0usize..4, 0..4),
        items in prop::collection::vec(any::<i64>(), 1..10),
    ) {
        let a = reshape(shape.clone(), &items).unwrap();
        prop_assert_eq!(a.shape(), &shape[..]);
    }

    #[test]
    fn take_then_drop_rebuilds(items in prop::collection::vec(any::<i64>(), 0..10), n in 0usize..10) {
        let a = Array::vector(items.clone());
        let n = n.min(items.len()) as i64;
        let front = take(n, &a, None).unwrap();
        prop_assert_eq!(front.shape()[0] as i64, n);
        prop_assert_eq!(cat(&front, &drop(n, &a)).unwrap(), a);
    }
}
