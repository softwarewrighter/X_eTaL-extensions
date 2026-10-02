//! Core desugaring tests (one test binary so helpers are shared).

mod accept;
mod equiv;
mod notes;
mod reject;

use xetal_core::lower;

/// The Core dump of `src`, one line per top-level item.
pub fn core(src: &str) -> String {
    match lower(src) {
        Ok(program) => program.to_string(),
        Err(e) => panic!("{src:?} should desugar, got {e:?}"),
    }
}

pub fn reject(src: &str) -> String {
    match lower(src) {
        Ok(p) => panic!("{src:?} should be rejected, got {p}"),
        Err(e) => e.code,
    }
}
