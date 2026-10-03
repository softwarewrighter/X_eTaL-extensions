//! hello, live: the tour, editable, run in the browser.

use hello_web::{COMMAND, PROGRAM, linked};
use xetal_ext_shell::Playground;
use xetal_ext_shell::run::install;
use yew::prelude::*;

#[function_component(App)]
fn app() -> Html {
    let functions = use_memo((), |()| install(&linked()));
    html! {
        <Playground
            title="hello"
            lede="The smallest X_eTaL native extension: Rust functions compiled to WebAssembly and linked into this page, called from X_eTaL through the Hello facade. Arrays go to Rust and come back with their shape; a Rust error or panic is an X_eTaL error."
            program={PROGRAM}
            command={COMMAND}
            functions={(*functions).clone()}
        />
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
