//! One frame: a matrix (or a vector, or a single value) as a grid.

use xetal_draw::{Cells, grid};

fn nums(v: &[f64]) -> Cells {
    Cells::Numbers(v.to_vec())
}

#[test]
fn a_bit_matrix_draws_its_ones_dark_on_paper() {
    let svg = grid(&[2, 3], &nums(&[1.0, 0.0, 1.0, 0.0, 0.0, 1.0])).unwrap();
    assert!(svg.starts_with("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"72\" height=\"48\" viewBox=\"0 0 72 48\" role=\"img\">"));
    assert!(svg.ends_with("</svg>\n"));
    assert!(svg.contains("<rect width=\"72\" height=\"48\" fill=\"#f8fafc\"/>"));
    let on =
        |x, y| format!("<rect x=\"{x}\" y=\"{y}\" width=\"24\" height=\"24\" fill=\"#1f2937\"/>");
    assert!(svg.contains(&on(0, 0)) && svg.contains(&on(48, 0)) && svg.contains(&on(48, 24)));
    assert_eq!(svg.matches("fill=\"#1f2937\"").count(), 3);
}

#[test]
fn grid_lines_are_one_path() {
    let svg = grid(&[2, 3], &nums(&[0.0; 6])).unwrap();
    assert_eq!(svg.matches("<path ").count(), 1);
    assert!(svg.contains("d=\"M0 0H72M0 24H72M0 48H72M0 0V48M24 0V48M48 0V48M72 0V48\""));
}

#[test]
fn a_vector_is_one_row_and_a_scalar_one_cell() {
    let row = grid(&[4], &nums(&[1.0, 0.0, 0.0, 1.0])).unwrap();
    assert!(row.contains("width=\"96\" height=\"24\""));
    let one = grid(&[], &nums(&[1.0])).unwrap();
    assert!(one.contains("width=\"24\" height=\"24\" viewBox"));
}

#[test]
fn other_numbers_go_through_the_palette_from_least_to_greatest() {
    let svg = grid(&[1, 3], &nums(&[2.0, 7.0, 12.0])).unwrap();
    assert_eq!(svg.matches("<rect x=").count(), 3);
    assert!(svg.contains("x=\"0\" y=\"0\" width=\"24\" height=\"24\" fill=\"#440154\""));
    assert!(svg.contains("x=\"48\" y=\"0\" width=\"24\" height=\"24\" fill=\"#fde725\""));
}

#[test]
fn characters_are_drawn_in_their_cells_escaped_and_spaces_left_empty() {
    let svg = grid(&[1, 3], &Cells::Chars(vec!['X', ' ', '<'])).unwrap();
    assert!(svg.contains(">X</text>") && svg.contains(">&lt;</text>"));
    assert_eq!(svg.matches("<text ").count(), 2);
    assert!(svg.contains("<text x=\"12\" y=\"12\""));
}
