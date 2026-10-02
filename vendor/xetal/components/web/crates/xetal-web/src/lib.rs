//! The live demo: the xetal editor in the browser. The ASCII source on
//! the left, drawn decorated on the right as you type, the types (or,
//! after Run, the output) below, as in `xetal edit`; Open (the demos,
//! the standard libraries and your files), Save, Run and Zoom. Files
//! and a program's keyboard are the browser's local storage and a
//! prompt. The language itself is `xetal-play`.

mod app;
mod demos;
mod keys;
mod panes;
mod storage;

pub use app::App;
pub use demos::{DEMOS, Demo, choices, open, seed};
pub use keys::{Action, action};

/// Start the app in the page's body.
pub fn start() {
    console_error_panic_hook::set_once();
    xetal_store::install(std::sync::Arc::new(storage::Local));
    seed();
    yew::Renderer::<App>::new().render();
}
