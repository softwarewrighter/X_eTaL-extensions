//! The three panes: the ASCII source (a text area), the source drawn
//! decorated, and the types or, after a run, the output.

use web_sys::{Element, HtmlTextAreaElement};
use xetal_play::{Class, check, decorate};
use xetal_runner::Runs;
use yew::prelude::*;

use crate::app::Pane;

/// A pane's frame: the current one is marked (a bright border and a
/// triangle before its title).
fn frame(title: &str, pane: Pane, current: Pane, focus: Callback<Pane>, body: Html) -> Html {
    let here = pane == current;
    let class = classes!("pane", pane.css(), here.then_some("current"));
    let heading = if here {
        format!("\u{25b6} {title}")
    } else {
        title.to_string()
    };
    html! {
        <section class={class} onfocusin={focus.reform(move |_: FocusEvent| pane)}>
            <h2>{ heading }</h2>
            { body }
        </section>
    }
}

/// The ASCII pane; as it scrolls (by hand, or as the browser follows
/// the cursor), the drawn pane scrolls to the same line.
pub(crate) fn source(
    text: &str,
    current: Pane,
    focus: Callback<Pane>,
    edit: Callback<String>,
    drawn: NodeRef,
) -> Html {
    let input = Callback::from(move |e: InputEvent| {
        let area: HtmlTextAreaElement = e.target_unchecked_into();
        edit.emit(area.value());
    });
    let follow = Callback::from(move |e: Event| {
        let area: HtmlTextAreaElement = e.target_unchecked_into();
        if let Some(pre) = drawn.cast::<Element>() {
            pre.set_scroll_top(area.scroll_top());
        }
    });
    let body = html! {
        <textarea value={text.to_string()} oninput={input} onscroll={follow}
            spellcheck="false" autocomplete="off" aria-label="source, as typed"/>
    };
    frame("ASCII", Pane::Source, current, focus, body)
}

fn class_name(class: Class) -> String {
    format!("c-{class:?}").to_lowercase()
}

pub(crate) fn rendered(text: &str, current: Pane, focus: Callback<Pane>, drawn: NodeRef) -> Html {
    let spans = decorate(text).into_iter().map(|s| {
        html! { <span class={class_name(s.class)}>{ s.text }</span> }
    });
    let body = html! { <pre tabindex="0" ref={drawn}>{ for spans }</pre> };
    frame("Rendered", Pane::Rendered, current, focus, body)
}

/// The types, or after a run its output (scrolled to the end by the
/// app, so the newest output shows).
/// A picture a run showed (`[]S_HOW`), as an image: an SVG data URL, so
/// its animation plays and nothing in it runs as script.
fn picture(svg: &str) -> Html {
    let url = format!(
        "data:image/svg+xml;charset=utf-8,{}",
        js_sys::encode_uri_component(svg)
    );
    html! { <span class="picture"><img src={url} alt="a picture the program showed"/></span> }
}

pub(crate) fn output(
    text: &str,
    runs: &Runs,
    current: Pane,
    focus: Callback<Pane>,
    printed: NodeRef,
    typing: &xetal_typing::Typing,
) -> Html {
    let o = &runs.output;
    let input = (o.waiting && !o.wants_key).then(|| xetal_typing::typed_line(typing));
    let title = match (&o.run, o.running, o.cells.is_empty()) {
        (None, ..) => "Types".to_string(),
        (_, true, _) if o.waiting && o.wants_key => "Output (waiting for a key)".to_string(),
        (_, true, _) if o.waiting => "Output (waiting for a line: type it, Enter)".to_string(),
        (_, true, _) => "Output (running...)".to_string(),
        (_, false, true) => "Output".to_string(),
        (_, false, false) if runs.stepped > 0 => format!("Notebook (step {})", runs.stepped),
        (_, false, false) => "Notebook".to_string(),
    };
    let body = match &o.run {
        None => html! { <pre tabindex="0">{ check(text).join("\n") }</pre> },
        Some(r) if o.cells.is_empty() => html! {
            <pre tabindex="0" ref={printed.clone()}>{ xetal_typing::screen(&r.out).unwrap_or_else(|| html! { { &r.out } }) }{ input }<span class="c-error">{ &r.err }</span>
                { for r.pictures.iter().map(|svg| picture(svg)) }</pre>
        },
        Some(r) => html! {
            <pre tabindex="0" ref={printed.clone()}>{ notebook(runs) }<span class="c-error">{ &r.err }</span></pre>
        },
    };
    frame(&title, Pane::Output, current, focus, body)
}

/// A notebook: each statement drawn decorated, indented six spaces as
/// in an APL session, its output and pictures under it; while stepping,
/// the last statement run is marked.
fn notebook(runs: &Runs) -> Html {
    let o = &runs.output;
    let last = o.cells.len().saturating_sub(1);
    let cells = o.cells.iter().enumerate().map(|(i, cell)| {
        let indented: String = cell
            .source
            .lines()
            .map(|l| format!("      {l}\n"))
            .collect();
        let spans = decorate(&indented).into_iter().map(|s| {
            html! { <span class={class_name(s.class)}>{ s.text }</span> }
        });
        let here = runs.stepped > 0 && i == last;
        html! {
            <>
                <span class={classes!("statement", here.then_some("current"))}>{ for spans }</span>
                { o.out_of(cell) }
                { for o.pictures_of(cell).iter().map(|svg| picture(svg)) }
            </>
        }
    });
    html! { <>{ for cells }</> }
}
