//! Source drawn decorated (`xetal-view`), each token in the class the
//! live demo colors it by, its names linked.

use xetal_doclink::{Link, Target};
use xetal_view::{Class, Segment, lines, view};

use crate::escape;

/// How a link's target becomes a URL.
pub type Href<'a> = &'a dyn Fn(Target) -> String;

/// `text` drawn decorated, for inside `<pre>` or `<code>`; a token
/// whose bytes are a link's is an `<a>` to its target.
pub fn code(text: &str, links: &[Link], href: Href) -> String {
    view(text).iter().map(|s| segment(s, links, href)).collect()
}

/// `text` drawn decorated line by line, each line numbered and
/// anchored (`id="L3"`), for a source page's `<pre>`.
pub fn lined(text: &str, links: &[Link], href: Href) -> String {
    let all = lines(&view(text));
    let count = text.lines().count();
    let mut out = String::new();
    for (i, line) in all.iter().take(count).enumerate() {
        let n = i + 1;
        let shown: String = line.iter().map(|s| segment(s, links, href)).collect();
        out.push_str(&format!(
            "<span class=\"line\" id=\"L{n}\"><a class=\"ln\" href=\"#L{n}\">{n}</a>{shown}</span>\n"
        ));
    }
    out
}

fn segment(s: &Segment, links: &[Link], href: Href) -> String {
    let text = escape(&s.text);
    let class = css(s.class).map_or(String::new(), |c| format!(" class=\"{c}\""));
    match links.iter().find(|l| l.span == s.raw) {
        Some(l) => format!("<a href=\"{}\"{class}>{text}</a>", href(l.target)),
        None if class.is_empty() => text,
        None => format!("<span{class}>{text}</span>"),
    }
}

/// The CSS class of a class of token (`c-builtin`, ...); none for plain
/// text, as `xetal render --html` names them.
fn css(class: Class) -> Option<String> {
    match class {
        Class::Variable | Class::Punct | Class::Unit | Class::Space => None,
        other => Some(format!("c-{other:?}").to_lowercase()),
    }
}
