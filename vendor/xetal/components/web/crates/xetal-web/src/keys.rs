//! The editor's keys in the browser. Browsers keep Ctrl-T (a new tab)
//! for themselves, so Zoom is Ctrl-. here; Run is Ctrl-Enter (and
//! Ctrl-R where the browser lets a page have it).

use yew::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Run,
    Zoom,
}

/// The action for a key pressed with Control (or Command) held.
pub fn action(key: &str, ctrl: bool) -> Option<Action> {
    match (ctrl, key) {
        (true, "Enter" | "r") => Some(Action::Run),
        (true, ".") => Some(Action::Zoom),
        _ => None,
    }
}

/// Keys for the whole page: Run, Zoom, and Escape to close Help.
pub(crate) fn keys(
    run: Callback<()>,
    zoom: Callback<()>,
    help: Callback<bool>,
) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        if e.key() == "Escape" {
            help.emit(false);
        }
        if let Some(act) = action(&e.key(), e.ctrl_key() || e.meta_key()) {
            e.prevent_default();
            match act {
                Action::Run => run.emit(()),
                Action::Zoom => zoom.emit(()),
            }
        }
    })
}
