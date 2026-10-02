//! Moving an axis of a generic array.

use proptest::prelude::*;
use xetal_array::Array;
use xetal_axes::move_axis;

#[test]
fn moving_axis_two_to_the_front_transposes_a_matrix() {
    let m = Array::new(vec![2, 3], (1..=6).collect()).unwrap();
    let t = move_axis(&m, 1, 0);
    assert_eq!(t.shape(), &[3, 2]);
    assert_eq!(t.data(), &[1, 4, 2, 5, 3, 6]);
}

#[test]
fn moving_an_axis_keeps_the_others_in_order() {
    let a = Array::new(vec![2, 3, 4], (0..24).collect()).unwrap();
    let t = move_axis(&a, 2, 0);
    assert_eq!(t.shape(), &[4, 2, 3]);
    assert_eq!(&t.data()[..3], &[0, 4, 8]);
}

proptest! {
    #[test]
    fn moving_back_restores_the_array(dims in prop::collection::vec(0usize..4, 1..4), from in 0usize..3, to in 0usize..3) {
        let (from, to) = (from % dims.len(), to % dims.len());
        let n = dims.iter().product::<usize>();
        let a = Array::new(dims.clone(), (0..n as i64).collect()).unwrap();
        prop_assert_eq!(move_axis(&move_axis(&a, from, to), to, from), a);
    }
}
