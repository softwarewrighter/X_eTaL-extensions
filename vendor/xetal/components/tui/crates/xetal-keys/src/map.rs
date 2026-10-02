//! Key events to commands. Control keys come from [`KEYMAP`]: nano's
//! file keys and Emacs motions (Ctrl-A/E line start and end, Ctrl-B/F
//! back and forward a character, Ctrl-P/N previous and next line) and
//! Ctrl-T to zoom the focused pane (for once not transpose); a
//! printable character without Control inserts itself.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What the application (not the buffer) does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Save,
    Quit,
    Run,
    /// Move the focus to the next (or previous) pane.
    NextPane,
    PrevPane,
    /// Toggle between all panes and the focused pane full screen.
    Zoom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Insert(char),
    Newline,
    Backspace,
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    App(Action),
}

const CTRL: KeyModifiers = KeyModifiers::CONTROL;
const NONE: KeyModifiers = KeyModifiers::NONE;

/// Keys with a fixed meaning, as (code, modifiers, command).
pub const KEYMAP: &[(KeyCode, KeyModifiers, Command)] = &[
    (KeyCode::Char('s'), CTRL, Command::App(Action::Save)),
    (KeyCode::Char('o'), CTRL, Command::App(Action::Save)),
    (KeyCode::Char('q'), CTRL, Command::App(Action::Quit)),
    (KeyCode::Char('x'), CTRL, Command::App(Action::Quit)),
    (KeyCode::Char('r'), CTRL, Command::App(Action::Run)),
    (KeyCode::Char('t'), CTRL, Command::App(Action::Zoom)),
    (KeyCode::Char('a'), CTRL, Command::Home),
    (KeyCode::Char('e'), CTRL, Command::End),
    (KeyCode::Char('b'), CTRL, Command::Left),
    (KeyCode::Char('f'), CTRL, Command::Right),
    (KeyCode::Char('n'), CTRL, Command::Down),
    (KeyCode::Char('p'), CTRL, Command::Up),
    (KeyCode::Enter, NONE, Command::Newline),
    (KeyCode::Backspace, NONE, Command::Backspace),
    (KeyCode::Delete, NONE, Command::Delete),
    (KeyCode::Left, NONE, Command::Left),
    (KeyCode::Right, NONE, Command::Right),
    (KeyCode::Up, NONE, Command::Up),
    (KeyCode::Down, NONE, Command::Down),
    (KeyCode::Home, NONE, Command::Home),
    (KeyCode::End, NONE, Command::End),
    (KeyCode::PageUp, NONE, Command::PageUp),
    (KeyCode::PageDown, NONE, Command::PageDown),
    (KeyCode::Tab, NONE, Command::App(Action::NextPane)),
    (KeyCode::BackTab, NONE, Command::App(Action::PrevPane)),
];

/// The command for a key event, if it has one.
pub fn command(key: KeyEvent) -> Option<Command> {
    let mods = key.modifiers - KeyModifiers::SHIFT;
    let fixed = KEYMAP
        .iter()
        .find(|(code, m, _)| *code == key.code && *m == mods);
    match (fixed, key.code) {
        (Some((_, _, c)), _) => Some(*c),
        (None, KeyCode::Char(c)) if !mods.contains(CTRL) && !mods.contains(KeyModifiers::ALT) => {
            Some(Command::Insert(c))
        }
        _ => None,
    }
}
