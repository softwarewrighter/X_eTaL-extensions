//! Open: a button and the menu it shows, the demos, libraries and your
//! files in collapsible groups, each header a caret, its name and a
//! count (all closed at first, one open at a time). The arrows move
//! between rows, Enter opens one, Escape or a click outside closes it.

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};
use yew::prelude::*;

/// A group's name and its rows, as (value, label).
type Group = (&'static str, Vec<(String, String)>);

/// What Open offers, as (group, value, label), and what a pick does.
#[derive(Properties, PartialEq)]
pub struct MenuProps {
    pub options: Vec<(&'static str, String, String)>,
    pub pick: Callback<String>,
}

#[function_component(OpenMenu)]
pub fn open_menu(props: &MenuProps) -> Html {
    let (shown, expanded) = (use_state(|| false), use_state(|| None));
    let (panel, button) = (use_node_ref(), use_node_ref());
    let s = shown.clone();
    let toggle = Callback::from(move |_: MouseEvent| s.set(!*s));
    let (s, p) = (shown.clone(), props.pick.clone());
    let pick = Callback::from(move |value: String| {
        s.set(false);
        p.emit(value);
    });
    let rows = groups(&props.options)
        .into_iter()
        .map(|g| group(g, &expanded, &pick));
    html! {
        <div class="open-menu" onkeydown={keys(shown.clone(), panel.clone(), button.clone())}>
            <button class="open" ref={button} onclick={toggle.clone()} aria-haspopup="menu" aria-expanded={shown.to_string()}
                title="Open a demo, a library or one of your files">{ "Open \u{25be}" }</button>
            if *shown {
                <div class="menu-backdrop" onclick={toggle} />
                <div class="menu" role="menu" ref={panel}>{ for rows }</div>
            }
        </div>
    }
}

/// The options in their groups, in the order given.
fn groups(options: &[(&'static str, String, String)]) -> Vec<Group> {
    let mut out: Vec<Group> = Vec::new();
    for (name, value, label) in options {
        match out.last_mut() {
            Some((last, rows)) if last == name => rows.push((value.clone(), label.clone())),
            _ => out.push((name, vec![(value.clone(), label.clone())])),
        }
    }
    out
}

/// One group: its header (caret, name, count) and, when open, its rows.
fn group(
    (name, rows): Group,
    expanded: &UseStateHandle<Option<&'static str>>,
    pick: &Callback<String>,
) -> Html {
    let open = **expanded == Some(name);
    let e = expanded.clone();
    let flip = Callback::from(move |_: MouseEvent| e.set(if open { None } else { Some(name) }));
    let count = rows.len();
    let items = rows.into_iter().map(|(value, label)| {
        let onclick = pick.reform(move |_: MouseEvent| value.clone());
        html! { <button class="item" role="menuitem" {onclick}>{ label }</button> }
    });
    html! {
        <div class="group">
            <button class="group-header" onclick={flip} aria-expanded={open.to_string()}>
                <span class="caret">{ if open { "\u{25be}" } else { "\u{25b8}" } }</span>
                <span class="label">{ name }</span>
                <span class="count">{ count }</span>
            </button>
            if open { { for items } }
        </div>
    }
}

/// Escape closes the menu; the arrows open it, then move between rows.
fn keys(shown: UseStateHandle<bool>, panel: NodeRef, button: NodeRef) -> Callback<KeyboardEvent> {
    Callback::from(move |e: KeyboardEvent| {
        let step = match e.key().as_str() {
            "Escape" if *shown => 0,
            "ArrowDown" => 1,
            "ArrowUp" => -1,
            _ => return,
        };
        e.prevent_default();
        e.stop_propagation();
        match (step, *shown) {
            (0, _) => {
                shown.set(false);
                if let Some(open) = button.cast::<HtmlElement>() {
                    let _ = open.focus();
                }
            }
            (_, false) => shown.set(true),
            _ => move_focus(&panel, step),
        }
    })
}

/// Focus the row `step` away from the focused one, wrapping around.
fn move_focus(panel: &NodeRef, step: i32) {
    let Some(list) = panel
        .cast::<Element>()
        .and_then(|m| m.query_selector_all("button").ok())
    else {
        return;
    };
    let rows: Vec<HtmlElement> = (0..list.length())
        .filter_map(|i| list.get(i)?.dyn_into().ok())
        .collect();
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    let at = rows
        .iter()
        .position(|r| Some(r.unchecked_ref::<Element>()) == active.as_ref());
    let next = match at {
        Some(i) => (i as i32 + step).rem_euclid(rows.len() as i32) as usize,
        None if step > 0 => 0,
        None => rows.len().saturating_sub(1),
    };
    if let Some(row) = rows.get(next) {
        let _ = row.focus();
    }
}
