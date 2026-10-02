//! Values laid out for display.

use xetal_grid::Grid;

fn grid(kind: &str, shape: &[usize], items: &[&str]) -> Grid {
    Grid {
        kind: kind.into(),
        shape: shape.to_vec(),
        items: items.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn a_scalar_is_one_line_with_its_type() {
    assert_eq!(grid("Int", &[], &["42"]).lines(), ["42  : Int"]);
}

#[test]
fn a_vector_shows_its_length() {
    assert_eq!(
        grid("Int", &[3], &["1", "20", "3"]).lines(),
        ["1 20 3  : Int 3"]
    );
    assert_eq!(grid("Int", &[0], &[]).lines(), ["(empty)  : Int 0"]);
}

#[test]
fn a_string_is_quoted() {
    assert_eq!(
        grid("Char", &[5], &["h", "e", "l", "l", "o"]).lines(),
        ["\"hello\"  : Char 5"]
    );
}

#[test]
fn a_matrix_is_a_box_with_aligned_columns() {
    let got = grid("Int", &[2, 3], &["1", "20", "3", "40", "5", "6"]).lines();
    assert_eq!(
        got,
        [
            "Int 2 3",
            "\u{250c}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2510}",
            "\u{2502}  1 20 3 \u{2502}",
            "\u{2502} 40  5 6 \u{2502}",
            "\u{2514}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2500}\u{2518}"
        ]
    );
}

#[test]
fn higher_ranks_are_labelled_slices() {
    let items: Vec<String> = (1..=8).map(|i| i.to_string()).collect();
    let g = Grid {
        kind: "Int".into(),
        shape: vec![2, 2, 2],
        items,
    };
    let got = g.lines();
    assert_eq!(got[0], "Int 2 2 2");
    assert_eq!(got[1], "[1]");
    assert!(got.contains(&"[2]".to_string()), "{got:?}");
    assert!(got.iter().any(|l| l.contains("7 8")), "{got:?}");
}

#[test]
fn a_char_matrix_shows_its_rows_as_text() {
    let g = grid("Char", &[2, 2], &["a", "b", "c", "d"]);
    assert!(
        g.lines().iter().any(|l| l.contains("ab")),
        "{:?}",
        g.lines()
    );
}
