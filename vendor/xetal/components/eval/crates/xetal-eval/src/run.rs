//! Running a program on a worker thread with a large stack.

use std::collections::HashMap;
use std::io::Write;

use xetal_arith::Rng;
use xetal_base::{Diagnostic, Span};
use xetal_core::Program;

use crate::events::Shown;

/// Stack reserved for evaluation (virtual memory; pages are used only as
/// recursion deepens).
const STACK_BYTES: usize = 1 << 30;

/// Evaluate a program, writing each top-level expression's value (and
/// any `p_rint!` output) to `out`. Returns the warnings, and the first
/// error if evaluation stopped.
///
/// Evaluation runs on its own thread with a large stack, so deep (but
/// finite) recursion works; runaway recursion is a `stack-overflow`
/// error, never a crash.
pub fn eval_source(
    src: &str,
    out: &mut (dyn Write + Send),
) -> (Vec<Diagnostic>, Result<(), Diagnostic>) {
    match xetal_core::lower(src) {
        Ok(program) => eval_program(&program, out, None),
        Err(e) => (Vec::new(), Err(e)),
    }
}

/// Evaluate an already lowered (and possibly type-elaborated) program;
/// see [`eval_source`]. `r_oll!` draws from `seed`, or from a fresh
/// unpredictable seed when there is none.
pub fn eval_program(
    program: &Program,
    out: &mut (dyn Write + Send),
    seed: Option<u64>,
) -> (Vec<Diagnostic>, Result<(), Diagnostic>) {
    eval_items(program, out, seed, &mut |_| {})
}

/// [`eval_program`], calling `before` with each top-level item's span
/// just before the item runs (a notebook shows the item's source then,
/// and its output as it is written).
pub fn eval_items(
    program: &Program,
    out: &mut (dyn Write + Send),
    seed: Option<u64>,
    before: &mut (dyn FnMut(Span) + Send),
) -> (Vec<Diagnostic>, Result<(), Diagnostic>) {
    let warnings = xetal_lint::warnings(program);
    let result = on_worker(|| {
        let mut machine = crate::machine::Machine {
            globals: HashMap::new(),
            out,
            depth: 0,
            rng: Rng::seeded(seed.unwrap_or_else(Rng::fresh_seed)),
            shown: None,
            before: Some(before),
        };
        machine.run(program).map_err(|d| program.annotate(d))
    })
    .and_then(|r| r);
    (warnings, result)
}

/// Run `program`, keeping its top-level values in `shown`.
pub(crate) fn run_showing(
    program: &Program,
    out: &mut (dyn Write + Send),
    seed: Option<u64>,
    shown: Shown,
) -> (Result<(), Diagnostic>, Vec<(usize, xetal_grid::Grid)>) {
    let mut machine = crate::machine::Machine {
        globals: HashMap::new(),
        out,
        depth: 0,
        rng: Rng::seeded(seed.unwrap_or_else(Rng::fresh_seed)),
        shown: Some(shown),
        before: None,
    };
    let result = machine.run(program).map_err(|d| program.annotate(d));
    (result, machine.shown.map(|s| s.values).unwrap_or_default())
}

/// Run `work` on a thread with a large stack (deep recursion is an
/// error, never a crash).
pub(crate) fn on_worker<T: Send>(work: impl FnOnce() -> T + Send) -> Result<T, Diagnostic> {
    // WebAssembly has no threads: there the work runs here, on a stack
    // the host makes large when it links (the live demo does).
    if cfg!(target_arch = "wasm32") {
        return Ok(work());
    }
    std::thread::scope(|scope| {
        let worker = std::thread::Builder::new()
            .stack_size(STACK_BYTES)
            .spawn_scoped(scope, work);
        match worker {
            Ok(handle) => handle
                .join()
                .map_err(|_| Diagnostic::new("internal", "the evaluator failed")),
            Err(e) => Err(Diagnostic::new(
                "internal",
                format!("cannot start the evaluator: {e}"),
            )),
        }
    })
}

pub(crate) fn err(code: &str, span: Span, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(code, message).with_span(span)
}
