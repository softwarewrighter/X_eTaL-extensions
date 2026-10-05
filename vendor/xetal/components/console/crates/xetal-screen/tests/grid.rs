//! The terminal's grid (QD6): text with the screen functions' ANSI
//! sequences drawn into fixed rows and columns - the cursor placed,
//! the screen cleared, colours and bold applied - as a terminal does.

use xetal_screen::{Color, Grid};

fn text(g: &Grid) -> Vec<String> {
    g.rows()
        .iter()
        .map(|r| {
            r.iter()
                .map(|c| c.ch)
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

#[test]
fn plain_lines_fill_rows_from_the_top() {
    let mut g = Grid::new(3, 10);
    g.write("one\ntwo\n");
    assert_eq!(text(&g), ["one", "two", ""]);
}

#[test]
fn the_cursor_is_placed_and_the_screen_cleared() {
    let mut g = Grid::new(3, 10);
    g.write("junk\n\x1b[2J\x1b[H\x1b[2;4Hhi\x1b[1;1H@");
    assert_eq!(text(&g), ["@", "   hi", ""]);
}

#[test]
fn colours_and_bold_apply_until_reset() {
    let mut g = Grid::new(1, 10);
    g.write("\x1b[31m\x1b[1mab\x1b[22m\x1b[39mc");
    let row = &g.rows()[0];
    assert_eq!((row[0].style.fg, row[0].style.bold), (Color::Red, true));
    assert_eq!(
        (row[2].style.fg, row[2].style.bold),
        (Color::Default, false)
    );
}

#[test]
fn writing_past_the_bottom_scrolls() {
    let mut g = Grid::new(2, 5);
    g.write("a\nb\nc\n");
    assert_eq!(text(&g), ["c", ""]);
}

#[test]
fn a_long_line_wraps() {
    let mut g = Grid::new(2, 3);
    g.write("abcde");
    assert_eq!(text(&g), ["abc", "de"]);
}
