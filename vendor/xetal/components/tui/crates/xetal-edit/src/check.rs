//! What the bottom pane shows: types or the first diagnostic (live),
//! or a run's output (Ctrl-R only, so effects never run on a keystroke),
//! each value laid out as a grid (`xetal-grid`).

use xetal_base::Diagnostic;

/// Lines to show, and the span of an error to highlight.
#[derive(Debug, Clone, Default)]
pub(crate) struct Report {
    pub lines: Vec<String>,
    pub mark: Option<(usize, usize)>,
}

fn failed(d: Diagnostic) -> Report {
    let mark = d.span.map(|s| (s.start, s.end));
    Report {
        lines: vec![d.to_string()],
        mark,
    }
}

/// The type of each top-level item (libraries loaded, found from
/// `origin`), or the error.
pub(crate) fn check(src: &str, origin: &str) -> Report {
    let mut loaded = match xetal_program::load(origin, src) {
        Ok(l) => l,
        Err(d) => return failed(d),
    };
    match xetal_types::check_program(&mut loaded.program) {
        Ok(lines) => Report {
            lines: xetal_program::program_types(lines),
            mark: None,
        },
        Err(d) => failed(xetal_program::in_program(&loaded.sources, d)),
    }
}

/// Printed text as its lines; a value laid out as a grid (type and
/// shape, matrices boxed, higher ranks as slices).
fn shown(event: &xetal_eval::Event) -> Vec<String> {
    match event {
        xetal_eval::Event::Printed(text) => text.lines().map(String::from).collect(),
        xetal_eval::Event::Value(grid) => grid.lines(),
    }
}

/// Type-check and run, collecting what the program shows.
pub(crate) fn run(src: &str, origin: &str) -> Report {
    let xetal_program::Loaded {
        sources,
        mut program,
    } = match xetal_program::load(origin, src) {
        Ok(l) => l,
        Err(d) => return failed(d),
    };
    let here = |d| xetal_program::in_program(&sources, d);
    if let Err(d) = xetal_types::check_program(&mut program) {
        return failed(here(d));
    }
    let (_, events, result) = xetal_eval::eval_events(&program, None);
    let mut report = Report {
        lines: events.iter().flat_map(shown).collect(),
        mark: None,
    };
    if let Err(d) = result {
        let err = failed(here(d));
        report.lines.extend(err.lines);
        report.mark = err.mark;
    }
    report
}
