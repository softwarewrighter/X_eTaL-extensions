//! Editing text with a cursor.

use proptest::prelude::*;
use xetal_buffer::Buffer;

fn typed(text: &str) -> Buffer {
    let mut b = Buffer::new("");
    for c in text.chars() {
        match c {
            '\n' => b.newline(),
            c => b.insert(c),
        }
    }
    b
}

#[test]
fn typing_builds_lines_and_moves_the_cursor() {
    let b = typed("x := 1\ny");
    assert_eq!(b.text(), "x := 1\ny");
    assert_eq!(b.cursor(), (1, 1));
    assert_eq!(b.offset(), 8);
    assert!(b.is_dirty());
}

#[test]
fn backspace_and_delete_join_lines() {
    let mut b = Buffer::new("ab\ncd");
    b.down();
    b.backspace();
    assert_eq!(b.text(), "abcd");
    assert_eq!(b.cursor(), (0, 2));
    b.delete();
    assert_eq!(b.text(), "abd");
    b.end();
    b.delete();
    assert_eq!(b.text(), "abd");
}

#[test]
fn motion_stays_inside_the_text() {
    let mut b = Buffer::new("long line\nx");
    b.end();
    b.down();
    assert_eq!(b.cursor(), (1, 1));
    b.down();
    b.right();
    assert_eq!(b.cursor(), (1, 1));
    b.home();
    b.left();
    assert_eq!(b.cursor(), (0, 9));
    b.up();
    assert_eq!(b.cursor(), (0, 9));
}

#[test]
fn saving_clears_the_dirty_flag() {
    let mut b = Buffer::new("x");
    assert!(!b.is_dirty());
    b.insert('y');
    b.mark_saved();
    assert!(!b.is_dirty());
}

#[test]
fn cursor_columns_count_characters() {
    let mut b = Buffer::new("");
    b.insert('\u{e9}');
    b.insert('x');
    assert_eq!(b.cursor(), (0, 2));
    assert_eq!(b.offset(), 3);
    b.backspace();
    b.backspace();
    assert_eq!(b.text(), "");
}

proptest! {
    #[test]
    fn typing_any_text_reproduces_it(text in "\\PC{0,30}(\n\\PC{0,10}){0,3}") {
        let b = typed(&text);
        prop_assert_eq!(b.text(), text.clone());
        prop_assert_eq!(b.offset(), text.len());
    }
}
