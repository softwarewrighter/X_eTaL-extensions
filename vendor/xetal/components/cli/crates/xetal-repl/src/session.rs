//! A session keeps the source it has accepted. Each new line is checked
//! and run together with that source (so definitions, types and `!`
//! variables persist), and only the output past what was already shown
//! is returned (and pictures shown before are not shown again); a line
//! that fails is not accepted, so its effects roll back. An unclosed
//! bracket waits for more lines.

use std::fmt::Write;

use xetal_base::{Diagnostic, Span};

use crate::live::Live;

/// Every replay rolls from the session's seed, so `r_oll!` results of
/// accepted lines keep their values from line to line.
#[derive(Debug)]
pub struct Session {
    accepted: String,
    pending: String,
    shown: usize,
    warned: usize,
    /// Pictures (`[]S_HOW`) the accepted source showed; a replay skips them.
    pictures: usize,
    seed: u64,
    /// Where libraries are found from: a file's path, or `-e` for the
    /// working directory.
    origin: String,
    /// Run without the type checker (`--untyped`).
    pub untyped: bool,
}

impl Default for Session {
    /// A session with an unpredictable seed.
    fn default() -> Self {
        Session::new("-e", xetal_eval::Rng::fresh_seed())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Reply {
    /// The input is incomplete (an unclosed bracket); send more lines.
    More,
    /// New output, and new warnings or an error.
    Done { out: String, err: String },
}

/// Output, warnings and the outcome of running `source` from the start,
/// its libraries (found relative to `origin`) loaded first; errors are
/// placed in the program's own text.
fn run(
    source: &str,
    session: &Session,
    sink: &mut (dyn std::io::Write + Send),
) -> (Vec<u8>, Vec<Diagnostic>, Result<(), Diagnostic>) {
    let (origin, seed, untyped) = (&session.origin, session.seed, session.untyped);
    let loaded = match xetal_program::load(origin, source) {
        Ok(l) => l,
        Err(e) => return (Vec::new(), Vec::new(), Err(e)),
    };
    let (sources, mut program) = (loaded.sources, loaded.program);
    let here = |d| xetal_program::in_program(&sources, d);
    let checked = match untyped {
        true => Ok(Vec::new()),
        false => xetal_types::check_program(&mut program),
    };
    if let Err(e) = checked {
        return (Vec::new(), Vec::new(), Err(here(e)));
    }
    let mut out = Live {
        skip: session.shown,
        all: Vec::new(),
        sink,
    };
    xetal_store::replay(session.pictures);
    let (warnings, result) = xetal_eval::eval_program(&program, &mut out, Some(seed));
    (
        out.all,
        warnings.into_iter().map(here).collect(),
        result.map_err(here),
    )
}

/// A diagnostic with its span made relative to the new input.
fn shifted(mut d: Diagnostic, offset: usize) -> Diagnostic {
    d.span = d.span.map(|s| match s.start >= offset {
        true => Span::new(s.start - offset, s.end - offset),
        false => s,
    });
    d
}

impl Session {
    /// A session whose libraries are found from `origin` (a file's
    /// path, or `-e` for the working directory) and whose rolls come
    /// from `seed`.
    pub fn new(origin: &str, seed: u64) -> Self {
        Session {
            accepted: String::new(),
            pending: String::new(),
            shown: 0,
            warned: 0,
            pictures: 0,
            seed,
            origin: origin.into(),
            untyped: false,
        }
    }

    /// Feed one line of input.
    pub fn feed(&mut self, line: &str) -> Reply {
        self.feed_to(line, &mut std::io::sink())
    }

    /// Feed one line of input, its new output also written to `sink`
    /// as the program writes it (to watch a long computation).
    pub fn feed_to(&mut self, line: &str, sink: &mut (dyn std::io::Write + Send)) -> Reply {
        let text = match self.pending.is_empty() {
            true => line.to_string(),
            false => format!("{}\n{line}", self.pending),
        };
        let source = match self.accepted.is_empty() {
            true => text.clone(),
            false => format!("{}\n{text}", self.accepted),
        };
        let offset = source.len() - text.len();
        let (out, warnings, result) = run(&source, self, sink);
        if matches!(&result, Err(d) if d.code == "unclosed") {
            self.pending = text;
            return Reply::More;
        }
        self.pending.clear();
        let new_out =
            String::from_utf8_lossy(out.get(self.shown..).unwrap_or_default()).into_owned();
        let mut err = String::new();
        match result {
            Ok(()) => {
                for w in &warnings[self.warned.min(warnings.len())..] {
                    let _ = writeln!(err, "{}", shifted(w.clone(), offset));
                }
                (self.accepted, self.shown, self.warned) = (source, out.len(), warnings.len());
                self.pictures = xetal_store::shown();
            }
            Err(e) => {
                let _ = writeln!(err, "{}", shifted(e, offset));
            }
        }
        Reply::Done { out: new_out, err }
    }
}
