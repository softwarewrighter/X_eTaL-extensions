//! The terminal's screen (Saga 25, after web-sw-tos's ui.rs): text
//! written into lines of cells, wrapped at the width, kept in a
//! scrollback, and shown as the last rows that fit.

use xetal_screen::{Cell, Color, Screen, Style};

fn text(rows: &[Vec<Cell>]) -> Vec<String> {
    rows.iter()
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
fn lines_are_shown_bottom_up_padded_to_the_size() {
    let mut s = Screen::new(100);
    s.write("one\ntwo\nthree\n", Style::default());
    let rows = s.rows(5, 2);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.len() == 5));
    assert_eq!(text(&rows), ["two", "three"]);
}

#[test]
fn an_unfinished_line_is_the_last_row() {
    let mut s = Screen::new(100);
    s.write("name? ", Style::default());
    s.write("Ada", Style::default());
    assert_eq!(text(&s.rows(20, 3)), ["", "", "name? Ada"]);
}

#[test]
fn long_lines_wrap_at_the_width() {
    let mut s = Screen::new(100);
    s.write("abcdefghij\n", Style::default());
    assert_eq!(text(&s.rows(4, 3)), ["abcd", "efgh", "ij"]);
}

#[test]
fn styles_travel_with_the_text() {
    let mut s = Screen::new(100);
    s.write("ok ", Style::default());
    s.write(
        "bad",
        Style {
            fg: Color::Red,
            ..Style::default()
        },
    );
    let row = &s.rows(6, 1)[0];
    assert_eq!(row[0].style.fg, Color::Default);
    assert_eq!(row[3].style.fg, Color::Red);
}

#[test]
fn the_scrollback_keeps_the_newest_lines() {
    let mut s = Screen::new(3);
    for i in 0..10 {
        s.write(&format!("line {i}\n"), Style::default());
    }
    assert_eq!(text(&s.rows(10, 5)), ["", "", "line 7", "line 8", "line 9"]);
}

#[test]
fn clear_empties_the_screen() {
    let mut s = Screen::new(100);
    s.write("old\nstuff", Style::default());
    s.clear();
    s.write("new", Style::default());
    assert_eq!(text(&s.rows(5, 2)), ["", "new"]);
}

#[test]
fn the_transcript_is_the_text_written() {
    let mut s = Screen::new(100);
    s.write("a\nb", Style::default());
    assert_eq!(s.transcript(), "a\nb");
}
