//! The editor's state and how keys change it.

use std::cell::Cell;
use std::path::{Path, PathBuf};

use ratatui::crossterm::event::KeyEvent;
use xetal_buffer::Buffer;
use xetal_keys::{Action, Command, apply, command};
use xetal_panes::Scroll;

use crate::check::{Report, check, run};
use crate::focus::{Pane, motion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Continue,
    Quit,
}

pub struct Editor {
    pub(crate) path: PathBuf,
    pub(crate) buffer: Buffer,
    pub(crate) focus: Pane,
    /// Scrolls of the ASCII, Rendered and Output panes (the first two
    /// follow the cursor while the ASCII pane has the focus).
    pub(crate) left: Cell<Scroll>,
    pub(crate) right: Cell<Scroll>,
    pub(crate) out: Cell<Scroll>,
    pub(crate) report: Report,
    /// True while the bottom pane shows a run's output.
    pub(crate) ran: bool,
    pub(crate) status: String,
    /// Only the focused pane is shown, full screen (Ctrl-T).
    pub(crate) zoom: bool,
    confirm_quit: bool,
}

impl Editor {
    /// Edit `path`; a missing file starts empty and is created on save.
    pub fn open(path: &Path) -> std::io::Result<Self> {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t.strip_suffix('\n').map_or(t.clone(), String::from),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e),
        };
        let fresh = || Cell::new(Scroll::default());
        Ok(Editor {
            path: path.into(),
            buffer: Buffer::new(&text),
            focus: Pane::Source,
            left: fresh(),
            right: fresh(),
            out: fresh(),
            report: check(&text, &path.display().to_string()),
            ran: false,
            status: "^S save  ^R run  ^Q quit  Tab pane  ^T zoom".into(),
            zoom: false,
            confirm_quit: false,
        })
    }

    /// Handle one key.
    pub fn handle(&mut self, key: KeyEvent) -> Flow {
        let Some(cmd) = command(key) else {
            return Flow::Continue;
        };
        match (cmd, motion(cmd)) {
            (Command::App(Action::NextPane), _) => self.cycle(1),
            (Command::App(Action::PrevPane), _) => self.cycle(-1),
            (_, Some(delta)) if self.focus != Pane::Source => self.scroll_focused(delta),
            _ => return self.edit(cmd),
        }
        Flow::Continue
    }

    /// Apply a command to the text (from any pane: typing returns the
    /// focus to the ASCII pane); re-check when the text changed.
    fn edit(&mut self, cmd: Command) -> Flow {
        if !matches!(cmd, Command::App(_)) {
            self.focus = Pane::Source;
        }
        let before = self.buffer.text();
        let flow = match apply(&mut self.buffer, cmd) {
            Some(action) => self.act(action),
            None => Flow::Continue,
        };
        if self.buffer.text() != before {
            (self.report, self.ran, self.confirm_quit) =
                (check(&self.buffer.text(), &self.origin()), false, false);
        }
        flow
    }

    fn act(&mut self, action: Action) -> Flow {
        match action {
            Action::Save => self.save(),
            Action::Run => {
                (self.report, self.ran) = (run(&self.buffer.text(), &self.origin()), true)
            }
            Action::Quit if self.buffer.is_dirty() && !self.confirm_quit => {
                self.confirm_quit = true;
                self.status = "unsaved changes: ^Q again to quit, ^S to save".into();
            }
            Action::Quit => return Flow::Quit,
            Action::Zoom => self.zoom = !self.zoom,
            Action::NextPane | Action::PrevPane => {}
        }
        Flow::Continue
    }

    /// Where the file's libraries are found from: its own path.
    fn origin(&self) -> String {
        self.path.display().to_string()
    }

    fn save(&mut self) {
        self.status = match std::fs::write(&self.path, self.buffer.text() + "\n") {
            Ok(()) => {
                self.buffer.mark_saved();
                format!("saved {}", self.path.display())
            }
            Err(e) => format!("cannot save {}: {e}", self.path.display()),
        };
        self.confirm_quit = false;
    }
}
