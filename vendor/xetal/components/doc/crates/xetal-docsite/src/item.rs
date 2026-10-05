//! One item on its file's page: name and type, kind and place, doc
//! comment and examples, source drawn decorated, where it is used.

use xetal_doc::Item;
use xetal_doccom::Doc;
use xetal_dochtml::{anchor, escape, example, page, prose};
use xetal_doclink::Target;

use crate::context::Ctx;
use crate::layout::short;

/// Item `index` of file `file`.
pub(crate) fn item(cx: &Ctx, file: usize, index: usize) -> String {
    let i = &cx.files[file].items[index];
    let id = anchor(&i.name);
    let ty = match i.ty.is_empty() {
        true => String::new(),
        false => format!(" <span class=\"type\">: {}</span>", escape(&i.ty)),
    };
    format!(
        "<div class=\"item\" id=\"{id}\">\n<h3><a href=\"#{id}\">{}</a>{ty}</h3>\n\
         <div class=\"meta\">{}</div>\n{}<pre class=\"code\">{}</pre>\n{}</div>\n",
        cx.name(&i.name),
        meta(cx, file, i),
        doc(cx, file, i.doc.as_ref()),
        cx.draw(file, &i.source),
        used(cx, file, index)
    )
}

/// What the item is, and the line it is defined on (a link into the
/// source page).
fn meta(cx: &Ctx, file: usize, i: &Item) -> String {
    let private = match i.public {
        true => "",
        false => " <span class=\"private\">(private)</span>",
    };
    let src = page(&cx.files[file].name);
    format!(
        "{}{private} &middot; <a href=\"{src}.src.html#L{line}\">line {line}</a>",
        i.kind,
        line = i.line
    )
}

/// A doc comment: its prose, then its examples as a session.
pub(crate) fn doc(cx: &Ctx, file: usize, doc: Option<&Doc>) -> String {
    let Some(doc) = doc else {
        return String::new();
    };
    let draw = |code: &str| cx.draw(file, code);
    let mut out = format!("<div class=\"doc\">\n{}", prose(&doc.text, &draw));
    for e in &doc.examples {
        out.push_str(&example(&draw(&e.code), &e.output));
    }
    out.push_str("</div>\n");
    out
}

/// Where the item is used: the item each use is in, else its line.
fn used(cx: &Ctx, file: usize, index: usize) -> String {
    let Some(places) = cx.uses.get(&Target::Item { file, item: index }) else {
        return String::new();
    };
    let mut shown: Vec<String> = places.iter().map(|&(f, l)| place(cx, f, l)).collect();
    shown.dedup();
    format!("<div class=\"uses\">Used in: {}</div>\n", shown.join(", "))
}

/// A use on line `line` of file `file`: the item whose source holds
/// it, or the line of the file.
fn place(cx: &Ctx, file: usize, line: usize) -> String {
    let items = &cx.files[file].items;
    let holds = |i: &Item| i.line <= line && line < i.line + i.source.lines().count().max(1);
    match items.iter().position(holds) {
        Some(item) => {
            let href = cx.href(Target::Item { file, item });
            format!("<a href=\"{href}\">{}</a>", cx.name(&items[item].name))
        }
        None => {
            let (name, src) = (&cx.files[file].name, page(&cx.files[file].name));
            let at = escape(short(name));
            format!("<a href=\"{src}.src.html#L{line}\">{at}:{line}</a>")
        }
    }
}
