//! The line being typed: characters with a cursor, and the lines typed
//! before, recalled with Up and Down.

use crate::keys::Key;

/// What a key did.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Still typing.
    Editing,
    /// Enter: this line is typed.
    Submit(String),
    /// Ctrl-C: stop the program.
    Interrupt,
}

/// A line under edit, and the history.
#[derive(Default)]
pub struct LineEditor {
    text: Vec<char>,
    cursor: usize,
    history: Vec<String>,
    /// Which history entry Up and Down are on (`history.len()`: none).
    recalled: Option<usize>,
}

impl LineEditor {
    /// The line typed so far, and where the cursor is in it.
    pub fn line(&self) -> String {
        self.text.iter().collect()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    /// Take one key.
    pub fn handle(&mut self, key: Key) -> Outcome {
        match key {
            Key::Enter => return self.submit(),
            Key::Interrupt => {
                self.set(String::new());
                return Outcome::Interrupt;
            }
            Key::Up | Key::Down => self.recall(key == Key::Up),
            Key::Char(c) => {
                self.text.insert(self.cursor, c);
                self.cursor += 1;
            }
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
            }
            Key::Delete if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
            }
            Key::Left => self.cursor = self.cursor.saturating_sub(1),
            Key::Right => self.cursor = (self.cursor + 1).min(self.text.len()),
            Key::Home => self.cursor = 0,
            Key::End => self.cursor = self.text.len(),
            Key::KillLine => self.set(String::new()),
            Key::Backspace | Key::Delete => {}
        }
        Outcome::Editing
    }

    fn submit(&mut self) -> Outcome {
        let line = self.line();
        if !line.is_empty() {
            self.history.push(line.clone());
        }
        self.recalled = None;
        self.set(String::new());
        Outcome::Submit(line)
    }

    /// Up (older) or Down (newer) through the history; past the newest
    /// is an empty line.
    fn recall(&mut self, older: bool) {
        let n = self.history.len();
        let at = self.recalled.unwrap_or(n);
        let next = match older {
            true => at.saturating_sub(1),
            false => (at + 1).min(n),
        };
        self.recalled = Some(next);
        self.set(self.history.get(next).cloned().unwrap_or_default());
    }

    fn set(&mut self, line: String) {
        self.text = line.chars().collect();
        self.cursor = self.text.len();
    }
}
