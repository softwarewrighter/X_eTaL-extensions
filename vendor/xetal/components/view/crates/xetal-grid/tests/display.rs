//! Nested arrays drawn as APL2's DISPLAY draws them.

use xetal_grid::{Body, Shown, display};

fn text(shape: &[usize], mark: char, lines: &[&str]) -> Shown {
    Shown::Frame {
        shape: shape.to_vec(),
        mark,
        body: Body::Text(lines.iter().map(|s| s.to_string()).collect()),
    }
}

#[test]
fn a_flat_vector_has_an_arrow_and_its_mark() {
    assert_eq!(
        display(&text(&[3], '~', &["1 2 3"])),
        ["┌→────┐", "│1 2 3│", "└~────┘"]
    );
    assert_eq!(display(&text(&[2], '─', &["ab"])), ["┌→─┐", "│ab│", "└──┘"]);
}

#[test]
fn a_matrix_has_a_down_arrow_and_a_box_of_a_scalar_none() {
    let m = text(&[2, 2], '~', &["1 2", "3 4"]);
    assert_eq!(display(&m), ["┌→──┐", "↓1 2│", "│3 4│", "└~──┘"]);
    assert_eq!(display(&text(&[], '~', &["5"])), ["┌─┐", "│5│", "└~┘"]);
}

#[test]
fn an_empty_vector_shows_the_empty_axis() {
    assert_eq!(display(&text(&[0], '~', &[])), ["┌⊖┐", "│ │", "└~┘"]);
}

#[test]
fn items_are_framed_side_by_side_and_centred() {
    let nested = Shown::Frame {
        shape: vec![2],
        mark: '∊',
        body: Body::Items(vec![Shown::Atom("7".into()), text(&[2], '─', &["ab"])]),
    };
    let want = [
        "┌→───────┐",
        "│   ┌→─┐ │",
        "│ 7 │ab│ │",
        "│   └──┘ │",
        "└∊───────┘",
    ];
    assert_eq!(display(&nested), want);
}

#[test]
fn ascii_keeps_the_shape_with_plain_characters() {
    let lines = display(&text(&[2], '─', &["ab"]));
    let plain: Vec<String> = lines.iter().map(|l| xetal_grid::to_ascii(l)).collect();
    assert_eq!(plain, [".>-.", "|ab|", "'--'"]);
    assert_eq!(xetal_grid::to_ascii("↓∊⊖~"), "veO~");
}
