//! The live demo's notebook: a program shown a statement at a time,
//! each statement's source (with the comments above it) handed over
//! just before it runs and its output after, as `just show` lays it
//! out; and stepping, a program run up to one statement and no further.

use std::io::Write;

use xetal_base::Span;
use xetal_core::Item;
use xetal_program::Loaded;

use crate::Run;
use crate::engine::{finish, ready};

/// Where an item starts in the program's combined text.
fn span(item: &Item) -> Span {
    match item {
        Item::Def { value, .. } | Item::Let { value, .. } | Item::Set { value, .. } => value.span,
        Item::Eval(e) => e.span,
    }
}

/// The last line (from 0) of each statement written in the program's
/// own file (not its libraries'), in order.
fn last_lines(loaded: &Loaded, src: &str) -> Vec<usize> {
    let mut lines: Vec<usize> = Vec::new();
    for item in &loaded.program.items {
        let at = loaded.sources.locate(span(item).end.max(1) - 1);
        let line = src[..at.offset.min(src.len())].matches('\n').count();
        if at.index == 0 && lines.last().is_none_or(|&l| line > l) {
            lines.push(line);
        }
    }
    lines
}

/// How many statements the program has (none if it does not load).
pub fn statements(src: &str) -> usize {
    ready(src).map_or(0, |loaded| last_lines(&loaded, src).len())
}

/// Run `src` as a notebook, up to statement `upto` if given: `cell` is
/// handed each statement's source as it starts, `out` its output.
pub fn notebook_to(
    src: &str,
    seed: u64,
    upto: Option<usize>,
    cell: &mut (dyn FnMut(&str) + Send),
    out: &mut (dyn Write + Send),
) -> Run {
    let loaded = match ready(src) {
        Ok(l) => l,
        Err(run) => return run,
    };
    let ends = last_lines(&loaded, src);
    let src: String = match upto.and_then(|k| ends.get(k.max(1) - 1)) {
        Some(&last) => src.lines().take(last + 1).collect::<Vec<_>>().join("\n"),
        None => src.to_string(),
    };
    let loaded = match ready(&src) {
        Ok(l) => l,
        Err(run) => return run,
    };
    let lines: Vec<&str> = src.lines().collect();
    let mut next = 0;
    let mut before = |span: Span| {
        let at = loaded.sources.locate(span.end.max(1) - 1);
        let line = src[..at.offset.min(src.len())].matches('\n').count();
        if at.index == 0 && line >= next {
            cell(&lines[next..=line.min(lines.len() - 1)].join("\n"));
            next = line + 1;
        }
    };
    xetal_store::take_shown();
    let (warnings, result) = xetal_eval::eval_items(&loaded.program, out, Some(seed), &mut before);
    let rest = lines.get(next..).unwrap_or_default().join("\n");
    if !rest.trim().is_empty() {
        cell(rest.trim_end());
    }
    finish(&loaded, warnings, result)
}
