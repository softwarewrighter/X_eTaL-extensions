//! Any drawable array gives one well-formed SVG document.

use proptest::prelude::*;
use xetal_draw::{Cells, grid};

fn balanced(svg: &str) -> bool {
    let opens = svg.matches("<g ").count() + svg.matches("<text ").count();
    let closes = svg.matches("</g>").count() + svg.matches("</text>").count();
    opens == closes && svg.matches("<svg ").count() == 1 && svg.ends_with("</svg>\n")
}

proptest! {
    #[test]
    fn numbers_draw_well_formed(frames in 1usize..4, rows in 1usize..5, cols in 1usize..5,
                                seed in proptest::collection::vec(-5.0f64..5.0, 64)) {
        let n = frames * rows * cols;
        let cells = Cells::Numbers(seed[..n].to_vec());
        let svg = grid(&[frames, rows, cols], &cells).unwrap();
        prop_assert!(balanced(&svg));
    }

    #[test]
    fn characters_draw_well_formed(text in "[ -~]{1,20}") {
        let chars: Vec<char> = text.chars().collect();
        let svg = grid(&[chars.len()], &Cells::Chars(chars)).unwrap();
        prop_assert!(balanced(&svg));
    }
}
