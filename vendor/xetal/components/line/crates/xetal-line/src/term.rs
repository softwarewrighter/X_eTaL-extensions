//! Reading one line on a terminal: raw mode only while the line is
//! being typed, the line redrawn in place after every key.

use std::io::{self, Write};

use ratatui::crossterm::cursor::MoveToColumn;
use ratatui::crossterm::event::{self, Event, KeyEventKind};
use ratatui::crossterm::terminal::{self, Clear, ClearType};
use ratatui::crossterm::{execute, queue};

use crate::{Line, Outcome};

/// Read a line after `prompt`; `None` at the end of input (Ctrl-D).
pub fn read_line(line: &mut Line, prompt: &str) -> io::Result<Option<String>> {
    terminal::enable_raw_mode()?;
    let result = edit(line, prompt);
    terminal::disable_raw_mode()?;
    println!();
    result
}

fn edit(line: &mut Line, prompt: &str) -> io::Result<Option<String>> {
    let mut out = io::stdout();
    loop {
        let (text, col) = line.draw(prompt);
        queue!(out, MoveToColumn(0), Clear(ClearType::CurrentLine))?;
        write!(out, "{text}")?;
        execute!(out, MoveToColumn(col as u16))?;
        let Event::Key(key) = event::read()? else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        match line.handle(key) {
            Outcome::Editing => {}
            Outcome::Submit(text) => return Ok(Some(text)),
            Outcome::End => return Ok(None),
        }
    }
}
