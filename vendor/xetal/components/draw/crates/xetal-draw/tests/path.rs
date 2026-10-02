//! Paths: a 2-row matrix of points (x over y) as one polyline, fitted
//! to the picture; a rank-3 array as frames.

use xetal_draw::{DrawError, path};

#[test]
fn a_square_is_fitted_with_a_margin_and_y_points_up() {
    // (0,0) (1,0) (1,1) (0,1) (0,0)
    let pts = [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0];
    let svg = path(&[2, 5], &pts).unwrap();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"424\" height=\"424\" viewBox=\"0 0 424 424\" role=\"img\">"));
    assert!(svg.contains("<polyline points=\"12,412 412,412 412,12 12,12 12,412\" fill=\"none\" stroke=\"#1f2937\" stroke-width=\"1.5\" stroke-linejoin=\"round\"/>"));
    assert!(!svg.contains("<path "), "no grid lines on a path");
}

#[test]
fn a_wide_path_keeps_its_aspect() {
    let svg = path(&[2, 2], &[0.0, 2.0, 0.0, 1.0]).unwrap();
    assert!(svg.contains("width=\"424\" height=\"224\""));
    assert!(svg.contains("points=\"12,212 412,12\""));
}

#[test]
fn frames_share_one_fit_and_take_turns() {
    let pts = [0.0, 1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 2.0];
    let svg = path(&[2, 2, 2], &pts).unwrap();
    assert_eq!(svg.matches("<polyline ").count(), 2);
    assert_eq!(svg.matches("<animate ").count(), 2);
    assert!(svg.contains("points=\"12,412 212,412\""));
}

#[test]
fn a_path_needs_two_rows_and_two_points() {
    assert_eq!(
        path(&[3, 2], &[0.0; 6]).unwrap_err(),
        DrawError::Points(vec![3, 2])
    );
    assert_eq!(
        path(&[2, 1], &[0.0; 2]).unwrap_err(),
        DrawError::Points(vec![2, 1])
    );
    assert_eq!(
        path(&[2, 1], &[0.0; 2]).unwrap_err().code(),
        "shape-mismatch"
    );
    assert_eq!(
        path(&[2, 2], &[0.0; 3]).unwrap_err(),
        DrawError::Length {
            expected: 4,
            found: 3
        }
    );
}

#[test]
fn one_point_repeated_is_still_a_picture() {
    let svg = path(&[2, 2], &[1.0, 1.0, 1.0, 1.0]).unwrap();
    assert!(svg.contains("<polyline points=\"12,12 12,12\""));
}
