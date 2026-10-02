//! Parser tests (one test binary so helpers are shared).

mod accept;
mod props;
mod reject;

use xetal_syntax::parse;

/// The S-expression dump of `src`, one line per statement.
pub fn tree(src: &str) -> String {
    match parse(src) {
        Ok(program) => program.to_string(),
        Err(e) => panic!("{src:?} should parse, got {e:?}"),
    }
}

/// The error code and span for a source that must be rejected.
pub fn reject(src: &str) -> (String, (usize, usize)) {
    match parse(src) {
        Ok(p) => panic!("{src:?} should be rejected, parsed as {p}"),
        Err(e) => {
            let span = e.span.expect("parse errors carry a span");
            (e.code, (span.start, span.end))
        }
    }
}
