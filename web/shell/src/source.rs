//! X_eTaL source drawn decorated (as X_eTaL renders it, colored).
//! Copied from X_eTaL-demos' shared/microscope.

use xetal_play::{Class, decorate};
use yew::prelude::*;

fn css(class: Class) -> &'static str {
    match class {
        Class::Builtin => "t-builtin",
        Class::UserFunc | Class::LibFunc | Class::Macro => "t-user",
        Class::LambdaArg => "t-arg",
        Class::Number | Class::Exponent => "t-num",
        Class::Symbol | Class::Quote => "t-sym",
        Class::Comment => "t-comment",
        _ => "t-plain",
    }
}

fn segments(src: &str) -> Html {
    html! { for decorate(src).into_iter().map(|s| html! { <span class={css(s.class)}>{s.text}</span> }) }
}

/// Any X_eTaL snippet, drawn decorated, inline.
pub fn code(src: &str) -> Html {
    html! { <code class="xtl">{ segments(src) }</code> }
}

/// A program (several lines, with comments), drawn decorated.
pub fn block(src: &str) -> Html {
    html! { <pre class="source">{ segments(src) }</pre> }
}

/// A program drawn decorated, as HTML text: a span a token, classed as
/// [`block`]'s are -- for a page that is not drawn by yew.
pub fn markup(src: &str) -> String {
    let mut out = String::new();
    for s in decorate(src) {
        out.push_str("<span class=\"");
        out.push_str(css(s.class));
        out.push_str("\">");
        for c in s.text.chars() {
            match c {
                '&' => out.push_str("&amp;"),
                '<' => out.push_str("&lt;"),
                '>' => out.push_str("&gt;"),
                _ => out.push(c),
            }
        }
        out.push_str("</span>");
    }
    out
}
