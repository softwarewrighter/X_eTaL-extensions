//! The editor's toolbar: the logo, Open, the file's name, Save, Save
//! as, Clear, the run buttons (Run or Stop, Notebook, Step, Reset), Zoom
//! and Help.

use yew::prelude::*;

use crate::menu::OpenMenu;
use crate::running::{RunButtons, run_buttons};

/// What the toolbar's controls do.
pub struct Bar {
    pub load: Callback<(String, String)>,
    pub save: Callback<bool>,
    pub runs: RunButtons,
    pub zoom: Callback<()>,
    pub help: Callback<bool>,
    /// What Open offers, as (group, value, label), and how a value opens.
    pub options: Vec<(&'static str, String, String)>,
    pub open: fn(&str) -> Option<(String, String)>,
    /// The file's name, and whether a pane is zoomed.
    pub name: String,
    pub zoomed: bool,
}

/// The logo, Open (demos, libraries, your files), the file's name,
/// Save, Save as, Clear, Run, Zoom and Help.
pub fn toolbar(bar: Bar) -> Html {
    let (load, open) = (bar.load.clone(), bar.open);
    let pick = Callback::from(move |value: String| {
        if let Some(opened) = open(&value) {
            load.emit(opened);
        }
    });
    html! {
        <nav class="toolbar">
            <img class="logo" src="modern-xetal-logo.jpg" alt="X_eTaL"/>
            <OpenMenu options={bar.options.clone()} {pick}/>
            <span class="name" title="The file being edited">{ &bar.name }</span>
            <button onclick={bar.save.reform(|_| false)} title="Save in this browser">{ "Save" }</button>
            <button onclick={bar.save.reform(|_| true)} title="Save under another name">{ "Save as" }</button>
            <button onclick={clearing(&bar)} title="Stop any run, and an empty editor">{ "Clear" }</button>
            { run_buttons(&bar.runs) }
            <button onclick={bar.zoom.reform(|_| ())} title="Zoom the current pane (Ctrl-.)">
                { if bar.zoomed { "Unzoom" } else { "Zoom" } }
            </button>
            <button class="help" onclick={bar.help.reform(|_| true)} title="How it works">{ "Help" }</button>
        </nav>
    }
}

/// Clear: stop any run, then an empty editor.
fn clearing(bar: &Bar) -> Callback<MouseEvent> {
    let (clear, load) = (bar.runs.clear.clone(), bar.load.clone());
    Callback::from(move |_| {
        clear.emit(());
        load.emit(("untitled.xtl".to_string(), String::new()));
    })
}
