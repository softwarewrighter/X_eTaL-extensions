//! Key events become commands; editing commands change a buffer.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use xetal_buffer::Buffer;
use xetal_keys::{Action, Command, apply, command};

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

fn ctrl(c: char) -> KeyEvent {
    KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
}

#[test]
fn nano_keys_map_to_commands() {
    assert_eq!(command(ctrl('s')), Some(Command::App(Action::Save)));
    assert_eq!(command(ctrl('o')), Some(Command::App(Action::Save)));
    assert_eq!(command(ctrl('q')), Some(Command::App(Action::Quit)));
    assert_eq!(command(ctrl('x')), Some(Command::App(Action::Quit)));
    assert_eq!(command(ctrl('r')), Some(Command::App(Action::Run)));
    assert_eq!(command(ctrl('a')), Some(Command::Home));
    assert_eq!(command(ctrl('e')), Some(Command::End));
    assert_eq!(command(ctrl('b')), Some(Command::Left));
    assert_eq!(command(ctrl('f')), Some(Command::Right));
    assert_eq!(command(ctrl('n')), Some(Command::Down));
    assert_eq!(command(ctrl('p')), Some(Command::Up));
    assert_eq!(command(key(KeyCode::Char('x'))), Some(Command::Insert('x')));
    assert_eq!(
        command(KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT)),
        Some(Command::Insert('X'))
    );
    assert_eq!(
        command(key(KeyCode::Tab)),
        Some(Command::App(Action::NextPane))
    );
    assert_eq!(
        command(KeyEvent::new(KeyCode::BackTab, KeyModifiers::SHIFT)),
        Some(Command::App(Action::PrevPane))
    );
    assert_eq!(command(ctrl('t')), Some(Command::App(Action::Zoom)));
    assert_eq!(command(key(KeyCode::PageDown)), Some(Command::PageDown));
    assert_eq!(command(ctrl('z')), None);
}

#[test]
fn a_scripted_session_edits_the_buffer() {
    let mut b = Buffer::new("");
    let keys = "x^2".chars().map(|c| key(KeyCode::Char(c))).chain([
        key(KeyCode::Enter),
        key(KeyCode::Char('y')),
        key(KeyCode::Up),
        key(KeyCode::End),
        key(KeyCode::Backspace),
        key(KeyCode::Char('3')),
    ]);
    let mut actions = Vec::new();
    for k in keys {
        if let Some(a) = command(k).and_then(|c| apply(&mut b, c)) {
            actions.push(a);
        }
    }
    assert_eq!(b.text(), "x^3\ny");
    assert!(actions.is_empty());
    assert_eq!(
        command(ctrl('s')).and_then(|c| apply(&mut b, c)),
        Some(Action::Save)
    );
}
