//! Running a program for display: instead of printing each top-level
//! value, keep it as a grid (`xetal-grid`), in order with the text that
//! `p_rint!` printed around it. The editor shows these; so will the
//! stepping debugger.

use std::io::Write;
use std::sync::{Arc, Mutex};

use xetal_base::Diagnostic;
use xetal_core::Program;
use xetal_grid::Grid;

use crate::run::on_worker;

/// What a run showed, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Text printed by `p_rint!`.
    Printed(String),
    /// A top-level expression's value.
    Value(Grid),
}

/// Printed text, shared between the machine's output and its log.
#[derive(Clone, Default)]
pub(crate) struct Shared(Arc<Mutex<Vec<u8>>>);

impl Shared {
    pub(crate) fn len(&self) -> usize {
        self.0.lock().map_or(0, |b| b.len())
    }

    fn take(&self) -> Vec<u8> {
        self.0.lock().map(|b| b.clone()).unwrap_or_default()
    }
}

impl Write for Shared {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| std::io::Error::other("poisoned"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// The values a machine kept, each with the printed length before it.
#[derive(Default)]
pub(crate) struct Shown {
    pub text: Shared,
    pub values: Vec<(usize, Grid)>,
}

/// Run `program`, returning its warnings, what it showed and how it ended.
pub fn eval_events(
    program: &Program,
    seed: Option<u64>,
) -> (Vec<Diagnostic>, Vec<Event>, Result<(), Diagnostic>) {
    let warnings = xetal_lint::warnings(program);
    let shown = Shown::default();
    let (text, mut out) = (shown.text.clone(), shown.text.clone());
    let (result, values) =
        match on_worker(|| crate::run::run_showing(program, &mut out, seed, shown)) {
            Ok((result, values)) => (result, values),
            Err(e) => (Err(e), Vec::new()),
        };
    (warnings, interleave(&text.take(), &values), result)
}

fn interleave(text: &[u8], values: &[(usize, Grid)]) -> Vec<Event> {
    let mut events = Vec::new();
    let mut at = 0;
    for (end, grid) in values {
        if *end > at {
            events.push(Event::Printed(
                String::from_utf8_lossy(&text[at..*end]).into_owned(),
            ));
        }
        events.push(Event::Value(grid.clone()));
        at = *end;
    }
    if text.len() > at {
        events.push(Event::Printed(
            String::from_utf8_lossy(&text[at..]).into_owned(),
        ));
    }
    events
}
