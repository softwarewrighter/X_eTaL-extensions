//! Splitting values into major cells and joining them back.

use std::rc::Rc;

use xetal_array::Array;
use xetal_hof::{join, major_cells};
use xetal_value::Value;

fn ints(shape: Vec<usize>, items: &[i64]) -> Value<'static> {
    let data = items.iter().map(|i| Value::Int(*i)).collect();
    Value::Array(Rc::new(Array::new(shape, data).unwrap()))
}

#[test]
fn a_matrix_splits_into_rows() {
    let (cells, shape) = major_cells(&ints(vec![2, 3], &[1, 2, 3, 4, 5, 6]));
    assert_eq!(shape, vec![3]);
    let printed: Vec<String> = cells.iter().map(ToString::to_string).collect();
    assert_eq!(printed, ["1 2 3", "4 5 6"]);
}

#[test]
fn a_scalar_is_one_cell() {
    let (cells, shape) = major_cells(&Value::Int(7));
    assert!(shape.is_empty());
    assert_eq!(cells.len(), 1);
}

#[test]
fn joining_rows_rebuilds_the_matrix() {
    let m = ints(vec![2, 3], &[1, 2, 3, 4, 5, 6]);
    let (cells, shape) = major_cells(&m);
    let back = join(&cells, &shape).unwrap();
    assert_eq!(back.to_string(), m.to_string());
}

#[test]
fn joining_no_cells_keeps_the_cell_shape() {
    let empty = join(&[], &[3]).unwrap();
    match empty {
        Value::Array(a) => assert_eq!(a.shape(), &[0, 3]),
        other => panic!("expected an array, got {other}"),
    }
}

#[test]
fn a_cell_of_another_shape_is_an_error() {
    let err = join(&[Value::Int(1), ints(vec![2], &[1, 2])], &[]).unwrap_err();
    assert_eq!(err.code, "shape-mismatch");
}
