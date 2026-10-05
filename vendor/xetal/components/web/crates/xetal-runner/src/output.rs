//! What the page shows of a run as its events arrive: the output
//! appended a line at a time, pictures and errors as they come, and
//! whether it is still running.

use std::rc::Rc;

use xetal_play::Run;
use yew::Reducible;

use crate::Event;

/// The run shown (none: the types are shown instead) and whether it is
/// still going.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    pub run: Option<Run>,
    pub running: bool,
    /// The program waits for a typed line (the terminal takes keys).
    pub waiting: bool,
    /// What it waits for is one key, sent at once, not echoed.
    pub wants_key: bool,
    /// In a notebook, each statement and where its output and pictures
    /// begin in the run's; none for a plain run.
    pub cells: Vec<Cell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub source: String,
    out_at: usize,
    pictures_at: usize,
}

impl Output {
    /// What `cell` printed: from its start to the next cell's.
    pub fn out_of(&self, cell: &Cell) -> &str {
        let out = self.run.as_ref().map_or("", |r| r.out.as_str());
        let end = self.after(cell).map_or(out.len(), |c| c.out_at);
        &out[cell.out_at..end]
    }

    /// The pictures `cell` showed.
    pub fn pictures_of(&self, cell: &Cell) -> &[String] {
        let all = self.run.as_ref().map_or(&[][..], |r| r.pictures.as_slice());
        let end = self.after(cell).map_or(all.len(), |c| c.pictures_at);
        &all[cell.pictures_at..end]
    }

    fn after(&self, cell: &Cell) -> Option<&Cell> {
        let i = self.cells.iter().position(|c| std::ptr::eq(c, cell))?;
        self.cells.get(i + 1)
    }
}

/// What happens to it.
pub enum Action {
    /// A run begins: empty output, running.
    Start,
    /// Something the run did, as it happened.
    Event(Event),
    /// A run that could not start (no worker) ended with this.
    Finished(Run),
    /// A line typed for the waiting program, echoed as a terminal does.
    Typed(String),
    /// A key pressed for the waiting program (not echoed).
    Pressed,
    /// Stopped by hand: what was shown stays, and says so.
    Stop,
    /// Back to showing the types.
    Clear,
}

impl Reducible for Output {
    type Action = Action;

    fn reduce(self: Rc<Self>, action: Action) -> Rc<Self> {
        let mut next = (*self).clone();
        let run = next.run.get_or_insert_with(Run::default);
        match action {
            Action::Start => {
                return Rc::new(Output {
                    run: Some(Run::default()),
                    running: true,
                    waiting: false,
                    wants_key: false,
                    cells: Vec::new(),
                });
            }
            Action::Event(Event::Source(source)) => {
                let (out_at, pictures_at) = (run.out.len(), run.pictures.len());
                next.cells.push(Cell {
                    source,
                    out_at,
                    pictures_at,
                });
            }
            Action::Event(Event::Out(line)) => run.out += &format!("{line}\n"),
            Action::Event(Event::Err(line)) => run.err += &format!("{line}\n"),
            Action::Event(Event::Picture(svg)) => run.pictures.push(svg),
            Action::Event(Event::Wrote(..) | Event::Ready) => {}
            Action::Event(Event::Done) => (next.running, next.waiting) = (false, false),
            Action::Event(Event::Waiting) => (next.waiting, next.wants_key) = (true, false),
            Action::Event(Event::WaitingKey) => (next.waiting, next.wants_key) = (true, true),
            Action::Pressed => (next.waiting, next.wants_key) = (false, false),
            Action::Typed(line) => {
                run.out += &format!("{line}\n");
                next.waiting = false;
            }
            Action::Finished(done) => (next.run, next.running) = (Some(done), false),
            Action::Stop => {
                run.err += "stopped\n";
                (next.running, next.waiting) = (false, false);
            }
            Action::Clear => return Rc::new(Output::default()),
        }
        Rc::new(next)
    }
}
