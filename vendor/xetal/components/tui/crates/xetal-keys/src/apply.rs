//! Applying a command: editing commands change the buffer; the others
//! are handed back to the application.

use xetal_buffer::Buffer;

use crate::{Action, Command};

/// Lines moved by PageUp and PageDown.
pub const PAGE: usize = 10;

/// Apply `command` to `buffer`; an application action is returned.
pub fn apply(buffer: &mut Buffer, command: Command) -> Option<Action> {
    match command {
        Command::Insert(c) => buffer.insert(c),
        Command::Newline => buffer.newline(),
        Command::Backspace => buffer.backspace(),
        Command::Delete => buffer.delete(),
        Command::Left => buffer.left(),
        Command::Right => buffer.right(),
        Command::Up => buffer.up(),
        Command::Down => buffer.down(),
        Command::Home => buffer.home(),
        Command::End => buffer.end(),
        Command::PageUp => (0..PAGE).for_each(|_| buffer.up()),
        Command::PageDown => (0..PAGE).for_each(|_| buffer.down()),
        Command::App(action) => return Some(action),
    }
    None
}
