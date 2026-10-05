//! Transpose kernels: element moves and the laws they keep (B17).

use proptest::prelude::*;
use xetal_array::Array;
use xetal_transpose::{permutation, permute, reverse_axes, swap_axes};

fn array(shape: &[usize]) -> Array<i64> {
    let n: usize = shape.iter().product();
    Array::new(shape.to_vec(), (1..=n as i64).collect()).expect("shape")
}

#[test]
fn a_matrix_transposes_rows_to_columns() {
    let t = reverse_axes(&array(&[2, 3]));
    assert_eq!(t.shape(), &[3, 2]);
    assert_eq!(t.data(), &[1, 4, 2, 5, 3, 6]);
}

#[test]
fn rank_three_moves_i_j_k_to_k_j_i() {
    let t = reverse_axes(&array(&[2, 3, 4]));
    assert_eq!(t.shape(), &[4, 3, 2]);
    // t[k][j][i] = a[i][j][k]; a[1][2][3] (0-origin) is 1 + 12 + 8 + 3.
    assert_eq!(t.data()[3 * 6 + 2 * 2 + 1], 24);
}

#[test]
fn swapping_two_axes_moves_only_those() {
    let t = swap_axes(&array(&[2, 3, 4]), 1, 2);
    assert_eq!(t.shape(), &[2, 4, 3]);
    assert_eq!(&t.data()[..3], &[1, 5, 9]);
}

#[test]
fn a_permutation_lists_each_axis_once() {
    assert_eq!(permutation(&[2, 1, 3], 3).expect("ok"), vec![1, 0, 2]);
    assert_eq!(permutation(&[1, 1], 2).expect_err("repeat").code, "domain");
    assert_eq!(permutation(&[0, 1], 2).expect_err("range").code, "domain");
    assert_eq!(permutation(&[1, 2, 3], 2).expect_err("len").code, "length");
}

fn shapes() -> impl Strategy<Value = Vec<usize>> {
    prop::collection::vec(0usize..4, 0..5)
}

proptest! {
    #[test]
    fn transposing_twice_is_the_identity(shape in shapes()) {
        let a = array(&shape);
        prop_assert_eq!(reverse_axes(&reverse_axes(&a)), a);
    }

    #[test]
    fn the_reversed_identity_permutation_is_transpose(shape in shapes()) {
        let a = array(&shape);
        let to: Vec<usize> = (0..shape.len()).rev().collect();
        prop_assert_eq!(permute(&a, &to), reverse_axes(&a));
    }

    #[test]
    fn a_swap_undone_is_the_identity(shape in prop::collection::vec(1usize..4, 2..5), j in 0usize..5, k in 0usize..5) {
        let (j, k) = (j % shape.len(), k % shape.len());
        let a = array(&shape);
        prop_assert_eq!(swap_axes(&swap_axes(&a, j, k), j, k), a);
    }
}
