//! Pictures and replays. A session runs its accepted source again for
//! each new line, so the pictures that source shows were shown before:
//! `replay(n)` at the start of a run lets the first n through silently,
//! and `shown()` after it says how many the run asked for. `muted(true)`
//! counts pictures without showing them (a context run silently).
//! The counts are process-wide, as the store is.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static SKIP: AtomicUsize = AtomicUsize::new(0);
static ASKED: AtomicUsize = AtomicUsize::new(0);
static MUTED: AtomicBool = AtomicBool::new(false);

/// Start a run whose first `skip` pictures were shown before.
pub fn replay(skip: usize) {
    SKIP.store(skip, Ordering::SeqCst);
    ASKED.store(0, Ordering::SeqCst);
}

/// How many pictures this run has asked to show (skipped ones too).
pub fn shown() -> usize {
    ASKED.load(Ordering::SeqCst)
}

/// While muted, pictures are counted but not shown.
pub fn muted(on: bool) {
    MUTED.store(on, Ordering::SeqCst);
}

/// Count a picture; true when it should really be shown.
pub(crate) fn admit() -> bool {
    let n = ASKED.fetch_add(1, Ordering::SeqCst);
    n >= SKIP.load(Ordering::SeqCst) && !MUTED.load(Ordering::SeqCst)
}
