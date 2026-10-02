//! A notebook in one pass: the program runs once, and the evaluator
//! says when each top-level statement starts, so its source is shown
//! then and its output streams under it; nothing runs twice. A file
//! with warnings or type errors, and the rest of a file after a runtime
//! error, go a statement at a time instead (`live`).

use std::io;

use xetal_base::{Diagnostic, Span};
use xetal_repl::Session;

use crate::live::{Indent, RED, RESET, stream, unclosed};

/// Lines `from..to` of `lines` shown as statements, grouped as the
/// statement-at-a-time notebook groups them; where the last began.
fn cells(lines: &[&str], from: usize, to: usize, show: &mut dyn FnMut(&str)) -> usize {
    let (mut start, mut last) = (from, from);
    for end in from + 1..=to {
        let text = lines[start..end].join("\n");
        if !unclosed(&text) {
            show(&text);
            (last, start) = (start, end);
        }
    }
    last
}

/// The notebook in one pass; `None` when the file must go a statement
/// at a time (it does not check, or has warnings), else whether a
/// statement failed.
pub(crate) fn once(
    origin: &str,
    source: &str,
    run: (u64, bool),
    show: &mut (dyn FnMut(&str) + Send),
) -> Option<bool> {
    let (seed, untyped) = run;
    let mut loaded = xetal_program::load(origin, source).ok()?;
    if !untyped && xetal_types::check_program(&mut loaded.program).is_err() {
        return None;
    }
    if !xetal_lint::warnings(&loaded.program).is_empty() {
        return None;
    }
    let lines: Vec<&str> = source.lines().collect();
    let (mut next, mut cell) = (0, 0);
    let mut before = |span: Span| {
        let at = loaded.sources.locate(span.end.max(span.start + 1) - 1);
        let line = source[..at.offset.min(source.len())].matches('\n').count();
        if at.index == 0 && line >= next {
            cell = cells(&lines, next, line + 1, &mut *show);
            next = line + 1;
        }
    };
    let mut out = Indent {
        at_start: true,
        out: io::stdout(),
    };
    let (_, result) = xetal_eval::eval_items(&loaded.program, &mut out, Some(seed), &mut before);
    Some(match result {
        Ok(()) => {
            cells(&lines, next, lines.len(), show);
            false
        }
        Err(e) => after_error(
            &loaded.sources,
            e,
            (origin, source),
            (cell, next),
            run,
            show,
        ),
    })
}

/// A runtime error: shown under its statement as the statement-at-a-
/// time notebook shows it; the statement is dropped and the rest of
/// the file goes on a statement at a time, after the statements before
/// it (run once more, silently).
fn after_error(
    sources: &xetal_sources::Sources,
    e: Diagnostic,
    (origin, source): (&str, &str),
    (cell, next): (usize, usize),
    (seed, untyped): (u64, bool),
    show: &mut (dyn FnMut(&str) + Send),
) -> bool {
    let lines: Vec<&str> = source.lines().collect();
    let offset: usize = lines[..cell].iter().map(|l| l.len() + 1).sum();
    let mut d = xetal_program::in_program(sources, e);
    d.span = d
        .span
        .map(|s| Span::new(s.start.saturating_sub(offset), s.end.saturating_sub(offset)));
    for line in d.to_string().lines() {
        println!("{RED}{line}{RESET}");
    }
    let mut session = Session::new(origin, seed);
    session.untyped = untyped;
    if cell > 0 {
        session.feed_to(&lines[..cell].join("\n"), &mut io::sink());
    }
    stream(&mut session, &lines[next..].join("\n"), show);
    true
}
