//! Browser key names (`KeyboardEvent.key`) to terminal keys. Meta and
//! Alt combinations stay with the browser (the page decides that);
//! here a Ctrl letter means its Emacs and shell editing command.

/// A key the line editor understands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Delete,
    Left,
    Right,
    Home,
    End,
    Up,
    Down,
    /// Ctrl-U: erase the line.
    KillLine,
    /// Ctrl-C: stop the program.
    Interrupt,
}

/// The key a browser reports (`name`, with Ctrl held or not), or `None`
/// for one the terminal ignores (Shift alone, function keys).
pub fn key(name: &str, ctrl: bool) -> Option<Key> {
    if ctrl {
        return match name {
            "a" => Some(Key::Home),
            "e" => Some(Key::End),
            "b" => Some(Key::Left),
            "f" => Some(Key::Right),
            "p" => Some(Key::Up),
            "n" => Some(Key::Down),
            "d" => Some(Key::Delete),
            "h" => Some(Key::Backspace),
            "u" => Some(Key::KillLine),
            "c" => Some(Key::Interrupt),
            _ => None,
        };
    }
    let mut chars = name.chars();
    match (name, chars.next(), chars.next()) {
        ("Enter", ..) => Some(Key::Enter),
        ("Backspace", ..) => Some(Key::Backspace),
        ("Delete", ..) => Some(Key::Delete),
        ("ArrowLeft", ..) => Some(Key::Left),
        ("ArrowRight", ..) => Some(Key::Right),
        ("ArrowUp", ..) => Some(Key::Up),
        ("ArrowDown", ..) => Some(Key::Down),
        ("Home", ..) => Some(Key::Home),
        ("End", ..) => Some(Key::End),
        (_, Some(c), None) => Some(Key::Char(c)),
        _ => None,
    }
}

/// A key pressed, named as a program reading one key sees it (`[]K_EY`,
/// QD6): its character, or `Up`, `Down`, `Left`, `Right`, `Enter`,
/// `Escape`, `Backspace`, `Tab`, `Delete`, `Home`, `End`; `None` for a
/// key a program never gets (Shift alone, function keys).
pub fn key_name(name: &str) -> Option<String> {
    let named = match name {
        "ArrowUp" => "Up",
        "ArrowDown" => "Down",
        "ArrowLeft" => "Left",
        "ArrowRight" => "Right",
        "Enter" | "Escape" | "Backspace" | "Tab" | "Delete" | "Home" | "End" => name,
        _ if name.chars().count() == 1 => name,
        _ => return None,
    };
    Some(named.to_string())
}
