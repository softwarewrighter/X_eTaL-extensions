//! Rotate and reverse on generic arrays.

use proptest::prelude::*;
use xetal_array::Array;
use xetal_rotate::{reverse, rotate};

fn m(shape: &[usize], xs: &[i64]) -> Array<i64> {
    Array::new(shape.to_vec(), xs.to_vec()).unwrap()
}

#[test]
fn rotate_moves_cells_toward_the_front() {
    assert_eq!(rotate(1, &m(&[3], &[1, 2, 3])), m(&[3], &[2, 3, 1]));
    assert_eq!(rotate(-4, &m(&[3], &[1, 2, 3])), m(&[3], &[3, 1, 2]));
    assert_eq!(
        rotate(1, &m(&[2, 2], &[1, 2, 3, 4])),
        m(&[2, 2], &[3, 4, 1, 2])
    );
    assert_eq!(rotate(5, &m(&[0], &[])), m(&[0], &[]));
}

#[test]
fn reverse_reverses_cells() {
    assert_eq!(reverse(&m(&[3], &[1, 2, 3])), m(&[3], &[3, 2, 1]));
    assert_eq!(
        reverse(&m(&[2, 2], &[1, 2, 3, 4])),
        m(&[2, 2], &[3, 4, 1, 2])
    );
    assert_eq!(reverse(&m(&[2, 0], &[])), m(&[2, 0], &[]));
}

proptest! {
    #[test]
    fn rotate_back_and_reverse_twice_are_identity(xs in prop::collection::vec(-9i64..10, 0..8), n in -20i64..20) {
        let a = m(&[xs.len()], &xs);
        prop_assert_eq!(rotate(-n, &rotate(n, &a)), a.clone());
        prop_assert_eq!(reverse(&reverse(&a)), a);
    }
}
