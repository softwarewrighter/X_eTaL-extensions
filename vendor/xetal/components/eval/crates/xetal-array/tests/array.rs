//! Dense row-major arrays: construction, scalar extension and layout.

use xetal_array::{Array, ArrayError, layout, zip};

fn strs(xs: &[i64]) -> Vec<String> {
    xs.iter().map(ToString::to_string).collect()
}

#[test]
fn construction_checks_the_shape_against_the_data() {
    let a = Array::new(vec![2, 3], (1..=6).collect::<Vec<i64>>()).unwrap();
    assert_eq!(a.shape(), &[2, 3]);
    assert_eq!(a.rank(), 2);
    assert_eq!(a.data(), &[1, 2, 3, 4, 5, 6]);
    assert_eq!(
        Array::new(vec![2, 2], vec![1, 2, 3]).unwrap_err(),
        ArrayError::Length {
            shape: vec![2, 2],
            len: 3
        }
    );
    let v = Array::vector(vec![7, 8]);
    assert_eq!(v.shape(), &[2]);
    let e: Array<i64> = Array::vector(Vec::new());
    assert_eq!(e.shape(), &[0]);
}

#[test]
fn map_keeps_the_shape() {
    let a = Array::new(vec![2, 2], vec![1, 2, 3, 4]).unwrap();
    let b: Array<i64> = a.map(|x| Ok::<_, ()>(x * 10)).unwrap();
    assert_eq!((b.shape(), b.data()), (&[2, 2][..], &[10, 20, 30, 40][..]));
    assert_eq!(a.map(|_| Err::<i64, _>("no")).unwrap_err(), "no");
}

#[test]
fn zip_needs_equal_shapes() {
    let a = Array::vector(vec![1, 2, 3]);
    let b = Array::vector(vec![10, 20, 30]);
    let c = zip(&a, &b, |x, y| Ok::<_, ArrayError>(x + y)).unwrap();
    assert_eq!(c.data(), &[11, 22, 33]);
    let short = Array::vector(vec![1, 2]);
    assert_eq!(
        zip(&a, &short, |x, y| Ok::<_, ArrayError>(x + y)).unwrap_err(),
        ArrayError::Shape {
            left: vec![3],
            right: vec![2]
        }
    );
}

#[test]
fn errors_convert_to_diagnostics() {
    let d: xetal_base::Diagnostic = ArrayError::Shape {
        left: vec![3],
        right: vec![2],
    }
    .into();
    assert_eq!(d.code, "shape-mismatch");
    assert_eq!(d.message, "shapes differ: 3 and 2");
}

#[test]
fn layout_follows_the_printing_rules() {
    assert_eq!(layout(&[3], &strs(&[1, 2, 3]), " "), "1 2 3");
    assert_eq!(layout(&[0], &[], " "), "");
    assert_eq!(
        layout(&[3], &["a".into(), "b".into(), "c".into()], ""),
        "abc"
    );
    assert_eq!(
        layout(&[2, 3], &strs(&[1, 20, 3, 400, 5, 6]), " "),
        "  1 20 3\n400  5 6"
    );
    assert_eq!(layout(&[2, 1, 2], &strs(&[1, 2, 3, 4]), " "), "1 2\n\n3 4");
    assert_eq!(layout(&[2, 1, 1, 1], &strs(&[1, 2]), " "), "1\n\n\n2");
}
