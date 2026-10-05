//! One block as one session: the context run silently, then each
//! example in turn, compared with its output.

use xetal_doccom::Example;
use xetal_repl::{Reply, Session};

use crate::Outcome;
use crate::compare::{actual, matches};

/// The seed every example's rolls are drawn from.
const SEED: u64 = 1;

/// Run `examples` after `context` (S10: as `-e` text would run).
pub(crate) fn run_block(context: &str, item: Option<String>, examples: &[Example]) -> Vec<Outcome> {
    let mut session = Session::new("-e", SEED);
    xetal_store::muted(true);
    for line in context.lines() {
        session.feed(line);
    }
    xetal_store::muted(false);
    examples
        .iter()
        .map(|e| {
            let (out, err) = match session.feed(&e.code) {
                Reply::Done { out, err } => (out, err),
                _ => (
                    String::new(),
                    "error[unclosed]: the example ends inside a bracket".into(),
                ),
            };
            Outcome {
                item: item.clone(),
                code: e.code.clone(),
                expected: e.output.clone(),
                actual: actual(&out, &err),
                passed: matches(&e.output, &out, &err),
            }
        })
        .collect()
}
