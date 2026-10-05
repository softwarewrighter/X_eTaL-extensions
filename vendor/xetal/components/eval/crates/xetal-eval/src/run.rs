//! Running a program on a worker thread with a large stack.

use std::io::Write;

use xetal_arith::Rng;
use xetal_base::{Diagnostic, Span};
use xetal_core::Program;

use crate::events::Shared;
use xetal_step::{Machine, Status};
use xetal_value::Value;

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
        let rng = Rng::seeded(seed.unwrap_or_else(Rng::fresh_seed));
        let mut machine = Machine::new(program, out, rng).hooks(None, Some(before));
        machine.finish().map_err(|d| program.annotate(d))
    })
    .and_then(|r| r);
    (warnings, result)
}

/// [`eval_source`], taken `budget` transitions at a time (D50): the
/// output is the same whatever the budget.
pub fn eval_in_slices(
    src: &str,
    budget: usize,
    out: &mut (dyn Write + Send),
) -> (Vec<Diagnostic>, Result<(), Diagnostic>) {
    let program = match xetal_core::lower(src) {
        Ok(program) => program,
        Err(e) => return (Vec::new(), Err(e)),
    };
    let warnings = xetal_lint::warnings(&program);
    let result = on_worker(|| {
        let rng = Rng::seeded(Rng::fresh_seed());
        let mut machine = Machine::new(&program, out, rng);
        while machine.run(budget)? == Status::Running {}
        Ok(())
    })
    .and_then(|r: Result<(), Diagnostic>| r.map_err(|d| program.annotate(d)));
    (warnings, result)
}

/// Run `program`, keeping its top-level values, each with the length of
/// the text printed before it.
pub(crate) fn run_showing(
    program: &Program,
    out: &mut (dyn Write + Send),
    seed: Option<u64>,
    text: &Shared,
) -> (Result<(), Diagnostic>, Vec<(usize, xetal_grid::Grid)>) {
    let mut values = Vec::new();
    let mut keep = |v: &Value<'_>| values.push((text.len(), xetal_value::grid(v)));
    let rng = Rng::seeded(seed.unwrap_or_else(Rng::fresh_seed));
    let result = Machine::new(program, out, rng)
        .hooks(Some(&mut keep), None)
        .finish()
        .map_err(|d| program.annotate(d));
    (result, values)
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
