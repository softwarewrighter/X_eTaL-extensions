//! clock, live: the time, and what the bridge costs, in the browser.

use clock_web::{COMMAND, PROGRAM, linked};
use xetal_ext_shell::Playground;
use xetal_ext_shell::run::install;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let functions = use_memo((), |()| install(&linked()));
    html! {
        <Playground
            title="clock"
            lede="Wall-clock and monotonic time for X_eTaL, which has no clock of its own: here Rust reads the browser's clocks. The program measures what the bridge between X_eTaL and Rust costs in this browser -- calls per second, and Floats per second carried out and back -- with clock and hello both compiled to WebAssembly. Run it again for fresh numbers."
            program={PROGRAM}
            command={COMMAND}
            functions={(*functions).clone()}
        />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
