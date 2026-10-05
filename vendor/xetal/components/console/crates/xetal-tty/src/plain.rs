//! The plain terminal: standard error, a key read from standard input
//! (on a terminal it arrives when Enter is pressed), and a fixed size.

use std::io::{IsTerminal, Read};

use crate::Tty;

/// Standard streams, no screen control of its own.
pub struct Plain;

impl Tty for Plain {
    fn error(&self, line: &str) {
        eprintln!("{line}");
    }

    fn key(&self) -> Result<String, String> {
        let mut byte = [0u8; 1];
        match std::io::stdin().read(&mut byte) {
            Ok(1) => Ok(key_name(char::from(byte[0])).to_string()),
            Ok(_) => Err("no more input".into()),
            Err(e) => Err(e.to_string()),
        }
    }

    fn facts(&self) -> [i64; 4] {
        let tty = i64::from(std::io::stdout().is_terminal());
        [24, 80, tty, tty]
    }
}

/// The name a program sees for a character typed: itself, or Enter,
/// Escape, Backspace or Tab.
pub fn key_name(c: char) -> String {
    match c {
        '\n' | '\r' => "Enter".into(),
        '\x1b' => "Escape".into(),
        '\x7f' | '\x08' => "Backspace".into(),
        '\t' => "Tab".into(),
        c => c.to_string(),
    }
}
