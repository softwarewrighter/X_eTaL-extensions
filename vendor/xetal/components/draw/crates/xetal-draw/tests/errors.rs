//! What cannot be drawn.

use xetal_draw::{Cells, DrawError, grid};

#[test]
fn rank_above_three_is_an_error() {
    let e = grid(&[1, 1, 1, 1], &Cells::Numbers(vec![1.0])).unwrap_err();
    assert_eq!(e, DrawError::Rank(4));
    assert_eq!(e.code(), "rank");
}

#[test]
fn an_empty_array_is_an_error() {
    let e = grid(&[2, 0], &Cells::Numbers(vec![])).unwrap_err();
    assert_eq!(e, DrawError::Empty);
    assert_eq!(e.code(), "empty");
}

#[test]
fn the_cells_must_fill_the_shape() {
    let e = grid(&[2, 2], &Cells::Numbers(vec![1.0])).unwrap_err();
    assert_eq!(
        e,
        DrawError::Length {
            expected: 4,
            found: 1
        }
    );
}
