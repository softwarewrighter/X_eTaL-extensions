//! The terminal's input in the live demo (Saga 25, after web-sw-tos's
//! keys): while a program waits for a line, keys are translated
//! (`xetal-lineedit`, a pure function) and typed into a line with a
//! cursor and history; Enter gives the line to the program, Ctrl-C
//! stops it. Meta and Alt stay with the browser.

use xetal_lineedit::{LineEditor, Outcome, key};
use xetal_runner::Runs;
use yew::prelude::*;

/// The line being typed, where its cursor is, and the key handler.
#[derive(Clone, PartialEq)]
pub struct Typing {
    pub line: String,
    pub cursor: usize,
    pub key: Callback<KeyboardEvent>,
}

/// Typing for a waiting program: `submit` takes each line typed, `stop`
/// is Ctrl-C.
#[hook]
pub fn use_typing(submit: Callback<String>, stop: Callback<()>) -> Typing {
    let editor = use_mut_ref(LineEditor::default);
    let redraw = use_force_update();
    let e = editor.clone();
    let on_key = Callback::from(move |event: KeyboardEvent| {
        if event.meta_key() || event.alt_key() {
            return;
        }
        let Some(k) = key(&event.key(), event.ctrl_key()) else {
            return;
        };
        event.prevent_default();
        let outcome = e.borrow_mut().handle(k);
        match outcome {
            Outcome::Submit(line) => submit.emit(line),
            Outcome::Interrupt => stop.emit(()),
            Outcome::Editing => {}
        }
        redraw.force_update();
    });
    let editor = editor.borrow();
    Typing {
        line: editor.line(),
        cursor: editor.cursor(),
        key: on_key,
    }
}

/// The line being typed, drawn with its cursor (a block on the
/// character under it).
pub fn typed_line(typing: &Typing) -> Html {
    let chars: Vec<char> = typing.line.chars().collect();
    let at = typing.cursor.min(chars.len());
    let before: String = chars[..at].iter().collect();
    let under = chars.get(at).map_or(' ', |c| *c);
    let after: String = chars
        .get(at + 1..)
        .map(|r| r.iter().collect())
        .unwrap_or_default();
    html! {
        <span class="typing">{ before }<span class="cursor">{ under }</span>{ after }</span>
    }
}

/// While a program waits for a line, keys go to the terminal unless the
/// source is being edited (`in_source`); the page's own keys (Command,
/// Escape, Ctrl-Enter) stay the page's.
pub fn terminal_keys(
    typing: &Typing,
    page: Callback<KeyboardEvent>,
    runs: &Runs,
    in_source: bool,
) -> Callback<KeyboardEvent> {
    let (terminal, press) = (typing.key.clone(), runs.press_key.clone());
    let (waiting, wants_key) = (runs.output.waiting, runs.output.wants_key);
    Callback::from(move |e: KeyboardEvent| {
        let page_key = e.meta_key() || (e.ctrl_key() && e.key() == "Enter");
        let named = (wants_key && !e.ctrl_key())
            .then(|| xetal_lineedit::key_name(&e.key()))
            .flatten();
        match (waiting && !in_source && !page_key, named) {
            (true, Some(name)) => {
                e.prevent_default();
                press.emit(name);
            }
            (true, None) if !wants_key && e.key() != "Escape" => terminal.emit(e),
            _ => page.emit(e),
        }
    })
}

/// When a program starts waiting for a line, `to_output` makes the
/// output pane the current one and `pane` takes the keyboard.
#[hook]
pub fn use_waiting(waiting: bool, to_output: Callback<()>, pane: NodeRef) {
    use_effect_with(waiting, move |waiting| {
        if *waiting {
            to_output.emit(());
            if let Some(pre) = pane.cast::<web_sys::HtmlElement>() {
                let _ = pre.focus();
            }
        }
    });
}

/// The terminal of `runs`: the line being typed, and the page's key
/// handler with keys routed to the terminal while the program waits
/// (unless the source is being edited, `in_source`); when it starts
/// waiting, `to_output` makes the output pane current and `pane` takes
/// the keyboard.
#[hook]
pub fn use_terminal(
    terminal: (&Runs, bool, Callback<()>),
    page: Callback<KeyboardEvent>,
    pane: NodeRef,
) -> (Typing, Callback<KeyboardEvent>) {
    let (runs, in_source, to_output) = terminal;
    let typing = use_typing(runs.type_line.clone(), runs.stop.clone());
    use_waiting(runs.output.waiting, to_output, pane);
    let keys = terminal_keys(&typing, page, runs, in_source);
    (typing, keys)
}

/// The output drawn as a terminal's grid (24 rows of 80 columns) when a
/// program placed or styled its text (QD6: the screen functions'
/// sequences); `None` for plain output.
pub fn screen(out: &str) -> Option<Html> {
    if !out.contains('\x1b') {
        return None;
    }
    let mut grid = xetal_screen::Grid::new(24, 80);
    grid.write(out);
    let rows = grid.rows().iter().map(|row| {
        let runs = row.chunk_by(|a, b| a.style == b.style).map(|cells| {
            let text: String = cells.iter().map(|c| c.ch).collect();
            html! { <span class={style_classes(cells[0].style)}>{ text }</span> }
        });
        html! { <>{ for runs }{ "\n" }</> }
    });
    Some(html! { <span class="grid">{ for rows }</span> })
}

fn style_classes(style: xetal_screen::Style) -> Classes {
    let colour = |prefix: &str, c: xetal_screen::Color| match c {
        xetal_screen::Color::Default => None,
        c => Some(format!("{prefix}-{}", format!("{c:?}").to_lowercase())),
    };
    classes!(
        colour("fg", style.fg),
        colour("bg", style.bg),
        style.bold.then_some("bold")
    )
}
