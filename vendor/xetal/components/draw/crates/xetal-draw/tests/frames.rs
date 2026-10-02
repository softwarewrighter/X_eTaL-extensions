//! A rank-3 array: frames along the leading axis, animated in turn.

use xetal_draw::{Cells, grid};

#[test]
fn each_frame_is_shown_for_its_share_of_the_loop() {
    let cells = Cells::Numbers(vec![1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    let svg = grid(&[3, 1, 2], &cells).unwrap();
    assert!(svg.contains("width=\"48\" height=\"24\""));
    assert_eq!(svg.matches("<g ").count(), 3);
    assert!(svg.contains("<g opacity=\"1\"><animate attributeName=\"opacity\" values=\"1;0;0\" calcMode=\"discrete\" dur=\"1.2s\" repeatCount=\"indefinite\"/>"));
    assert!(svg.contains("<g opacity=\"0\"><animate attributeName=\"opacity\" values=\"0;1;0\""));
    assert!(svg.contains("values=\"0;0;1\""));
}

#[test]
fn the_palette_spans_every_frame() {
    let svg = grid(&[2, 1, 1], &Cells::Numbers(vec![3.0, 9.0])).unwrap();
    assert!(svg.contains("fill=\"#440154\"") && svg.contains("fill=\"#fde725\""));
}

#[test]
fn one_frame_is_not_animated() {
    let svg = grid(&[1, 1, 2], &Cells::Numbers(vec![1.0, 0.0])).unwrap();
    assert!(!svg.contains("<animate"));
}
