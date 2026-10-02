//! The Help dialog's frame: closed by its X, a click outside it, or
//! Escape (the app's keys).

use yew::prelude::*;

/// The Help dialog; closed by its X, a click outside it, or Escape.
pub fn help(close: Callback<()>) -> Html {
    let outside = close.reform(|_: MouseEvent| ());
    let inside = Callback::from(|e: MouseEvent| e.stop_propagation());
    html! {
        <div class="overlay" onclick={outside}>
            <div class="dialog" onclick={inside} role="dialog" aria-label="Help">
                <button class="close" onclick={close.reform(|_| ())} title="Close (Escape)">
                    { "\u{00d7}" }
                </button>
                { crate::help::help_text() }
            </div>
        </div>
    }
}
