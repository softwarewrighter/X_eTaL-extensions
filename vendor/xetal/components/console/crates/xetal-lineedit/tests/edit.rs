//! Typing a line (Saga 25): browser key names translated to keys (a
//! pure function, as web-sw-tos's translate.rs), and a line editor with
//! a cursor and history that submits on Enter.

use xetal_lineedit::{Key, LineEditor, Outcome, key};

fn typed(editor: &mut LineEditor, keys: &[Key]) -> Vec<Outcome> {
    keys.iter().map(|k| editor.handle(*k)).collect()
}

fn chars(s: &str) -> Vec<Key> {
    s.chars().map(Key::Char).collect()
}

#[test]
fn browser_key_names_become_keys() {
    assert_eq!(key("a", false), Some(Key::Char('a')));
    assert_eq!(key("Enter", false), Some(Key::Enter));
    assert_eq!(key("Backspace", false), Some(Key::Backspace));
    assert_eq!(key("ArrowUp", false), Some(Key::Up));
    assert_eq!(key("ArrowLeft", false), Some(Key::Left));
    assert_eq!(key("Shift", false), None);
    assert_eq!(key("F5", false), None);
    assert_eq!(key("u", true), Some(Key::KillLine));
    assert_eq!(key("a", true), Some(Key::Home));
    assert_eq!(key("e", true), Some(Key::End));
    assert_eq!(key("c", true), Some(Key::Interrupt));
}

#[test]
fn a_line_is_typed_and_submitted() {
    let mut e = LineEditor::default();
    typed(&mut e, &chars("go north"));
    assert_eq!(e.line(), "go north");
    assert_eq!(e.handle(Key::Enter), Outcome::Submit("go north".into()));
    assert_eq!(e.line(), "");
}

#[test]
fn backspace_and_the_cursor_edit_in_place() {
    let mut e = LineEditor::default();
    typed(&mut e, &chars("lok"));
    typed(&mut e, &[Key::Left, Key::Left, Key::Char('o')]);
    assert_eq!((e.line(), e.cursor()), ("look".to_string(), 2));
    typed(&mut e, &[Key::End, Key::Backspace, Key::Home, Key::Delete]);
    assert_eq!(e.line(), "oo");
    typed(&mut e, &[Key::KillLine]);
    assert_eq!(e.line(), "");
}

#[test]
fn history_recalls_earlier_lines() {
    let mut e = LineEditor::default();
    for line in ["look", "take lamp"] {
        typed(&mut e, &chars(line));
        e.handle(Key::Enter);
    }
    e.handle(Key::Up);
    assert_eq!(e.line(), "take lamp");
    e.handle(Key::Up);
    assert_eq!(e.line(), "look");
    e.handle(Key::Down);
    assert_eq!(e.line(), "take lamp");
    e.handle(Key::Down);
    assert_eq!(e.line(), "");
}

#[test]
fn interrupt_is_reported() {
    let mut e = LineEditor::default();
    typed(&mut e, &chars("x"));
    assert_eq!(e.handle(Key::Interrupt), Outcome::Interrupt);
    assert_eq!(e.line(), "");
}

#[test]
fn a_pressed_key_is_named_as_programs_see_it() {
    use xetal_lineedit::key_name;
    assert_eq!(key_name("a"), Some("a".into()));
    assert_eq!(key_name("ArrowUp"), Some("Up".into()));
    assert_eq!(key_name("Enter"), Some("Enter".into()));
    assert_eq!(key_name("Escape"), Some("Escape".into()));
    assert_eq!(key_name("Shift"), None);
    assert_eq!(key_name("F5"), None);
}
