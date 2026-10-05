//! The command line's terminal (QD6): `[]K_EY` reads one key in raw
//! mode when input is a terminal (else one character of standard input),
//! `[]T_E` reports the window's size, `[]E_RR` writes standard error.

use std::io::IsTerminal;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::crossterm::terminal;
use xetal_tty::{Plain, Tty};

/// The terminal `xetal` runs programs on.
pub struct Terminal;

impl Tty for Terminal {
    fn error(&self, line: &str) {
        Plain.error(line);
    }

    fn key(&self) -> Result<String, String> {
        if !std::io::stdin().is_terminal() {
            return Plain.key();
        }
        terminal::enable_raw_mode().map_err(|e| e.to_string())?;
        let key = one_key();
        let _ = terminal::disable_raw_mode();
        key.map_err(|e| e.to_string())
    }

    fn facts(&self) -> [i64; 4] {
        let tty = std::io::stdout().is_terminal();
        let (cols, rows) = terminal::size().unwrap_or((80, 24));
        [
            i64::from(rows),
            i64::from(cols),
            i64::from(tty),
            i64::from(tty),
        ]
    }
}

/// The next key pressed, by name.
fn one_key() -> std::io::Result<String> {
    loop {
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let name = match key.code {
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Up => "Up".into(),
            KeyCode::Down => "Down".into(),
            KeyCode::Left => "Left".into(),
            KeyCode::Right => "Right".into(),
            KeyCode::Enter => "Enter".into(),
            KeyCode::Esc => "Escape".into(),
            KeyCode::Backspace => "Backspace".into(),
            KeyCode::Tab => "Tab".into(),
            KeyCode::Delete => "Delete".into(),
            KeyCode::Home => "Home".into(),
            KeyCode::End => "End".into(),
            _ => continue,
        };
        return Ok(name);
    }
}
