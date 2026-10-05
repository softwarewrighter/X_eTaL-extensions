//! A program running in the worker a slice at a time (D50): a slice,
//! then a yield so the worker hears messages (a typed line; the page
//! ends the worker to stop it), until the program waits for a line or
//! ends.

use std::cell::RefCell;

use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::DedicatedWorkerGlobalScope;
use xetal_play::{Interactive, Step};

use crate::Event;
use crate::protocol::{fields, frame};
use crate::worker::post;

/// Transitions run per slice before the worker yields (to hear a typed
/// line, or be stopped).
const SLICE: usize = 200_000;

thread_local! {
    /// The program running a slice at a time, while it runs.
    static CURRENT: RefCell<Option<Interactive>> = const { RefCell::new(None) };
}

/// Start `src` stepping, its output posted a line at a time.
pub(crate) fn start(src: &str, seed: u64) {
    let out = xetal_play::Lines::new(|line: &str| post(Event::Out(line.into())));
    match Interactive::start(src, seed, Box::new(out)) {
        Ok(running) => {
            CURRENT.with(|c| *c.borrow_mut() = Some(running));
            tick();
        }
        Err(done) => finish(done),
    }
}

/// One slice of the running program, then: another slice soon (after
/// the worker has heard any message), a wait for a typed line, or the
/// end.
fn tick() {
    let step = CURRENT.with(|c| c.borrow_mut().as_mut().map(|r| r.step(SLICE)));
    match step {
        Some(Step::Running) => later(tick),
        Some(Step::Waiting) => post(Event::Waiting),
        Some(Step::WaitingKey) => post(Event::WaitingKey),
        Some(Step::Done(run)) => {
            CURRENT.with(|c| *c.borrow_mut() = None);
            finish(run);
        }
        None => {}
    }
}

/// A typed line for the waiting program, which goes on.
pub(crate) fn typed(line: String) {
    CURRENT.with(|c| {
        if let Some(r) = c.borrow_mut().as_mut() {
            r.feed(line);
        }
    });
    tick();
}

/// Call `f` once the worker has handled what is waiting for it.
fn later(f: fn()) {
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();
    let callback = Closure::once_into_js(f);
    let _ =
        scope.set_timeout_with_callback_and_timeout_and_arguments_0(callback.unchecked_ref(), 0);
}

/// A run's end: what it printed that was not yet posted (a library's
/// types), its warnings and error, then Done.
pub(crate) fn finish(run: xetal_play::Run) {
    run.out.lines().for_each(|l| post(Event::Out(l.into())));
    run.err.lines().for_each(|l| post(Event::Err(l.into())));
    post(Event::Done);
}

/// The page's message giving the waiting program a typed line (a frame
/// no request can be: a request has four fields or more).
pub fn line_message(line: &str) -> String {
    frame(&["line", line])
}

/// The typed line a message carries, if it is one.
pub fn typed_line(message: &str) -> Option<String> {
    match fields(message)?.as_slice() {
        ["line", line] => Some(line.to_string()),
        _ => None,
    }
}
