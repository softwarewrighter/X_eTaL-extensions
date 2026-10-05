//! A file's pages: its documentation (doc, imports, sections with their
//! items) and its source, drawn decorated, every line numbered.

use xetal_doc::DocFile;
use xetal_dochtml::{anchor, escape, lined, page};
use xetal_doclink::links;

use crate::context::Ctx;
use crate::item::{doc, item};
use crate::layout::shell;

/// The page of file `file`.
pub(crate) fn file_page(cx: &Ctx, file: usize) -> String {
    let f = &cx.files[file];
    let mut body = format!(
        "<h1><span class=\"kind\">{}</span>{}</h1>\n{}<p class=\"meta\"><a href=\"{}.src.html\">source</a>{}</p>\n",
        f.kind,
        escape(&f.name),
        doc(cx, file, f.doc.as_ref()),
        page(&f.name),
        imports(cx, f)
    );
    let mut toc = String::new();
    for (n, (title, items)) in groups(f).iter().enumerate() {
        if let Some(title) = title {
            let title = escape(title);
            body.push_str(&format!(
                "<h2 class=\"section\" id=\"section-{n}\">{title}</h2>\n"
            ));
            toc.push_str(&format!("<li><a href=\"#section-{n}\">{title}</a></li>\n"));
        }
        for &i in items {
            body.push_str(&item(cx, file, i));
            toc.push_str(&entry(
                cx,
                &format!("#{}", anchor(&f.items[i].name)),
                &f.items[i].name,
            ));
        }
    }
    shell(cx, &f.name, &toc, &body)
}

/// The items of `f` by section, in order: those before any section
/// first (number 0, no title), then each section (numbered from 1).
fn groups(f: &DocFile) -> Vec<(Option<String>, Vec<usize>)> {
    let mut out: Vec<(Option<String>, Vec<usize>)> = vec![(None, Vec::new())];
    for (i, it) in f.items.iter().enumerate() {
        match out.iter_mut().find(|(t, _)| *t == it.section) {
            Some((_, items)) => items.push(i),
            None => out.push((it.section.clone(), vec![i])),
        }
    }
    out
}

/// The file's imports: each alias, and the files it found, linked.
fn imports(cx: &Ctx, f: &DocFile) -> String {
    let shown: Vec<String> = f
        .imports
        .iter()
        .map(|i| {
            let found: Vec<String> = i
                .files
                .iter()
                .map(|n| format!("<a href=\"{}.html\">{}</a>", page(n), escape(n)))
                .collect();
            format!(
                "<code>{}</code> {}",
                cx.name(&format!("{}:", i.alias)),
                found.join(" ")
            )
        })
        .collect();
    match shown.is_empty() {
        true => String::new(),
        false => format!(" &middot; imports {}", shown.join("; ")),
    }
}

/// The source page of file `file`: every line numbered and anchored,
/// every name linked; each item's line links to its documentation.
pub(crate) fn source_page(cx: &Ctx, file: usize) -> String {
    let f = &cx.files[file];
    let drawn = lined(&f.text, &links(&cx.r, file, &f.text), &|t| cx.href(t));
    let toc: String = f
        .items
        .iter()
        .map(|i| entry(cx, &format!("#L{}", i.line), &i.name))
        .collect();
    let body = format!(
        "<h1><span class=\"kind\">source</span><a href=\"{}.html\">{}</a></h1>\n<pre class=\"source\">{drawn}</pre>\n",
        page(&f.name),
        escape(&f.name)
    );
    shell(cx, &f.name, &toc, &body)
}

/// A side bar entry for an item: its name drawn, linked to `href`.
fn entry(cx: &Ctx, href: &str, name: &str) -> String {
    format!(
        "<li class=\"item\"><a href=\"{href}\">{}</a></li>\n",
        cx.name(name)
    )
}
