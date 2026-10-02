//! A file run as a notebook: each statement (one line, or several while
//! a bracket is open) with the output and errors it produced, from a
//! session fed line by line (so definitions persist, printing is not
//! repeated and one seed keeps the rolls consistent).

use crate::{Reply, Session};

/// One statement's source and what running it printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    pub source: String,
    pub out: String,
    pub err: String,
}

/// The cells of `src` (libraries found from `origin`), rolls drawn from
/// `seed`.
pub fn notebook(origin: &str, src: &str, seed: u64, untyped: bool) -> Vec<Cell> {
    let mut session = Session::new(origin, seed);
    session.untyped = untyped;
    let (mut cells, mut source) = (Vec::new(), Vec::new());
    for line in src.lines() {
        source.push(line);
        if let Reply::Done { out, err } = session.feed(line) {
            cells.push(Cell {
                source: source.join("\n"),
                out,
                err,
            });
            source.clear();
        }
    }
    if !source.is_empty() {
        let err = "error[unclosed]: the file ends inside a bracket\n".to_string();
        cells.push(Cell {
            source: source.join("\n"),
            out: String::new(),
            err,
        });
    }
    cells
}

/// `block` run after `context` (earlier blocks, run silently in the
/// same session): what the block printed, as one cell. For org-babel
/// sessions, where each block continues the ones before it.
pub fn continued(origin: &str, context: &str, block: &str, seed: u64) -> Cell {
    let mut session = Session::new(origin, seed);
    xetal_store::muted(true); // the context runs silently: no pictures either
    for line in context.lines() {
        session.feed(line);
    }
    xetal_store::muted(false);
    let (mut out, mut err) = (String::new(), String::new());
    for line in block.lines() {
        if let Reply::Done { out: o, err: e } = session.feed(line) {
            out.push_str(&o);
            err.push_str(&e);
        }
    }
    Cell {
        source: block.to_string(),
        out,
        err,
    }
}
