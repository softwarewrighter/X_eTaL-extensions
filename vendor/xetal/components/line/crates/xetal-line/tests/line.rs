//! The line editor as a state machine: keys in, a drawn line out.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xetal_line::{Line, Outcome};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

fn typed(line: &mut Line, text: &str) {
    for c in text.chars() {
        assert_eq!(line.handle(key(KeyCode::Char(c))), Outcome::Editing);
    }
}

fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut esc = false;
    for c in s.chars() {
        match (esc, c) {
            (false, '\u{1b}') => esc = true,
            (true, 'm') => esc = false,
            (true, _) => {}
            (false, c) => out.push(c),
        }
    }
    out
}

#[test]
fn the_line_is_drawn_decorated_as_it_is_typed() {
    let mut line = Line::default();
    typed(&mut line, "u:s_quare := { _r * _r }");
    let (text, col) = line.draw("xetal> ");
    assert_eq!(
        plain(&text),
        "xetal> \u{1d58}s\u{332}quare \u{2190} { \u{2375} \u{d7} \u{2375} }"
    );
    assert_eq!(col, 7 + 19);
}

#[test]
fn enter_submits_the_ascii_text() {
    let mut line = Line::default();
    typed(&mut line, "x^2");
    assert_eq!(
        line.handle(key(KeyCode::Enter)),
        Outcome::Submit("x^2".into())
    );
    assert_eq!(plain(&line.draw("> ").0), "> ");
}

#[test]
fn the_cursor_maps_into_the_decorated_line() {
    let mut line = Line::default();
    typed(&mut line, "r_ev x");
    line.handle(ctrl('a'));
    line.handle(ctrl('f'));
    line.handle(ctrl('f'));
    line.handle(ctrl('f'));
    line.handle(ctrl('f'));
    assert_eq!(line.draw("").1, 3);
}

#[test]
fn up_and_down_walk_the_history() {
    let mut line = Line::default();
    for entry in ["1 + 2", "r_ev 1 2 3"] {
        typed(&mut line, entry);
        line.handle(key(KeyCode::Enter));
    }
    typed(&mut line, "draft");
    line.handle(key(KeyCode::Up));
    assert_eq!(line.text(), "r_ev 1 2 3");
    line.handle(key(KeyCode::Up));
    line.handle(key(KeyCode::Up));
    assert_eq!(line.text(), "1 + 2");
    line.handle(key(KeyCode::Down));
    line.handle(key(KeyCode::Down));
    assert_eq!(line.text(), "draft");
}

#[test]
fn ctrl_d_ends_on_an_empty_line_and_ctrl_c_clears() {
    let mut line = Line::default();
    typed(&mut line, "abc");
    assert_eq!(line.handle(ctrl('d')), Outcome::Editing);
    assert_eq!(line.handle(ctrl('c')), Outcome::Editing);
    assert_eq!(line.text(), "");
    assert_eq!(line.handle(ctrl('d')), Outcome::End);
}
