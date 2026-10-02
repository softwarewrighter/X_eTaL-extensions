//! Segments as HTML spans (`xetal render --html`): each run of one
//! class is a `<span class="c-...">`, named as the live demo's
//! Rendered pane names them, so one stylesheet colors both; plain text
//! (names, punctuation, spaces) is left bare. Text is escaped.

use crate::{Class, Segment};

/// The CSS class of a class of token; none for plain text.
fn css(class: Class) -> Option<String> {
    match class {
        Class::Variable | Class::Punct | Class::Unit | Class::Space => None,
        other => Some(format!("c-{other:?}").to_lowercase()),
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The segments as HTML, for inside a `<pre>`.
pub fn html(segments: &[Segment]) -> String {
    let mut out = String::new();
    let mut rest = segments;
    while let Some(first) = rest.first() {
        let n = rest.iter().take_while(|s| s.class == first.class).count();
        let run: String = rest[..n].iter().map(|s| s.text.as_str()).collect();
        match css(first.class) {
            Some(c) => out.push_str(&format!("<span class=\"{c}\">{}</span>", escape(&run))),
            None => out.push_str(&escape(&run)),
        }
        rest = &rest[n..];
    }
    out
}
