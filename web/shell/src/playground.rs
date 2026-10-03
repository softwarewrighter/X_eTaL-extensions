//! The page body: an X_eTaL program, editable and run in the browser,
//! its decorated source, its output (or error), and the native
//! functions the page links in.

use web_sys::HtmlTextAreaElement;
use xetal_ext_loader::FunctionInfo;
use yew::prelude::*;

use crate::chrome::{footer, header, panel};
use crate::run::{Outcome, run};
use crate::source::{block, code};

#[derive(Properties, PartialEq)]
pub struct PlaygroundProps {
    /// The page's title.
    pub title: AttrValue,
    /// One paragraph under the title.
    pub lede: AttrValue,
    /// The program shown first (a demo's `.xtl`).
    pub program: AttrValue,
    /// How to run the same program on the command line.
    pub command: AttrValue,
    /// The functions linked in (from `run::install`).
    pub functions: Vec<FunctionInfo>,
}

#[function_component(Playground)]
pub fn playground(p: &PlaygroundProps) -> Html {
    let src = use_state(|| p.program.to_string());
    let outcome = use_state(|| run(&p.program));
    let edited = *src != *p.program;

    let oninput = {
        let src = src.clone();
        Callback::from(move |e: InputEvent| {
            let area: HtmlTextAreaElement = e.target_unchecked_into();
            src.set(area.value());
        })
    };
    let on_run = {
        let (src, outcome) = (src.clone(), outcome.clone());
        Callback::from(move |_| outcome.set(run(&src)))
    };
    let on_reset = {
        let (src, outcome, program) = (src.clone(), outcome.clone(), p.program.to_string());
        Callback::from(move |_| {
            outcome.set(run(&program));
            src.set(program.clone());
        })
    };

    html! { <>
        { header(&p.title, &p.lede) }
        <main>
            <div class="controls">
                <button class="run" onclick={on_run}>{ "Run" }</button>
                <button onclick={on_reset} disabled={!edited}>{ "Reset" }</button>
                <span class="gen">{ timing(&outcome) }</span>
            </div>
            <div class="layout">
                <div class="col">
                    { panel("Program", "Edit it and press Run: it runs here, in your browser, with the native extension compiled to WebAssembly.", html! {
                        <textarea class="editor" spellcheck="false" rows={rows(&src)} value={(*src).clone()} {oninput} />
                    }) }
                    { panel("As X_eTaL shows it", "", block(&src)) }
                </div>
                <div class="col">
                    { panel("Output", "", output(&outcome)) }
                    { panel("Native functions", "What the page links in, with their X_eTaL types; the facade gives each a typed X_eTaL name.", functions(&p.functions)) }
                    { panel("On the command line", "", html! { <pre class="source">{ p.command.to_string() }</pre> }) }
                </div>
            </div>
        </main>
        { footer() }
    </> }
}

fn rows(src: &str) -> String {
    (src.lines().count() + 1).clamp(8, 40).to_string()
}

fn timing(o: &Outcome) -> String {
    match o.millis > 0.0 {
        true => format!("ran in {:.1} ms", o.millis),
        false => String::new(),
    }
}

fn output(o: &Outcome) -> Html {
    html! { <>
        if !o.out.is_empty() { <pre class="out">{ o.out.clone() }</pre> }
        if !o.err.is_empty() { <pre class="error">{ o.err.clone() }</pre> }
        if o.out.is_empty() && o.err.is_empty() { <p class="note">{ "(no output)" }</p> }
    </> }
}

fn functions(fs: &[FunctionInfo]) -> Html {
    html! {
        <table class="fns">
            { for fs.iter().map(|f| html! {
                <tr>
                    <td><code>{ format!("{}/{}", f.extension, f.name) }</code></td>
                    <td>{ code(&f.signature) }</td>
                    <td class="doc">{ f.doc.clone() }</td>
                </tr>
            }) }
        </table>
    }
}
