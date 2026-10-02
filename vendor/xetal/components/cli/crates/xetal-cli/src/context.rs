//! `xetal run --context FILE BLOCK`: run BLOCK after the program in
//! FILE, as one session, showing only BLOCK's output. Org-babel
//! sessions use it: a block sees what the blocks before it defined.

use xetal_base::Diagnostic;

/// Run `source` after the context file `path`; errors go to stderr and
/// fail the run.
pub(crate) fn after_context(
    path: &str,
    origin: &str,
    source: &str,
    seed: Option<u64>,
) -> Result<String, Diagnostic> {
    let context = std::fs::read_to_string(path)
        .map_err(|e| Diagnostic::new("unreadable", format!("cannot read {path}: {e}")))?;
    let seed = seed.unwrap_or_else(xetal_eval::Rng::fresh_seed);
    let cell = xetal_repl::continued(origin, &context, source, seed);
    print!("{}", cell.out);
    eprint!("{}", cell.err);
    match cell.err.lines().any(|l| l.starts_with("error[")) {
        true => Err(Diagnostic::new("failed", "the block failed (shown above)")),
        false => Ok(String::new()),
    }
}
