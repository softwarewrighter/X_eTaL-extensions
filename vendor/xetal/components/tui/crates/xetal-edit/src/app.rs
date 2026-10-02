//! Running the editor on the terminal.

use std::io;
use std::path::Path;

use ratatui::crossterm::event::{self, Event, KeyEventKind};

use crate::{Editor, Flow};

/// Edit `path` until the user quits.
pub fn run(path: &Path) -> io::Result<()> {
    let mut editor = Editor::open(path)?;
    xetal_term::with_terminal(|terminal| {
        loop {
            terminal.draw(|f| editor.draw(f))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && editor.handle(key) == Flow::Quit
            {
                return Ok(());
            }
        }
    })
}
