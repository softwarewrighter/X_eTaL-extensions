//! The index (every file and every item), the built-ins page, and the
//! site as a whole.

use std::path::Path;

use xetal_base::Diagnostic;
use xetal_doc::DocFile;
use xetal_dochtml::{STYLE, THEME, anchor, escape, page};
use xetal_doclink::Target;

use crate::context::Ctx;
use crate::layout::shell;
use crate::page::{file_page, source_page};

/// Every page of the site of `files`: (path in the site, content).
pub fn site(files: &[DocFile]) -> Vec<(String, String)> {
    let cx = Ctx::new(files);
    let mut out = vec![
        ("index.html".to_string(), index(&cx)),
        ("builtins.html".to_string(), builtins(&cx)),
        ("style.css".to_string(), STYLE.to_string()),
        ("theme.js".to_string(), THEME.to_string()),
    ];
    for (i, f) in files.iter().enumerate() {
        out.push((format!("{}.html", page(&f.name)), file_page(&cx, i)));
        out.push((format!("{}.src.html", page(&f.name)), source_page(&cx, i)));
    }
    out
}

/// The site of `files` written into `dir` (created if missing); the
/// paths written, in order.
pub fn write(dir: &Path, files: &[DocFile]) -> Result<Vec<String>, Diagnostic> {
    let failed =
        |e: std::io::Error| Diagnostic::new("io", format!("cannot write {}: {e}", dir.display()));
    std::fs::create_dir_all(dir).map_err(failed)?;
    let mut written = Vec::new();
    for (path, content) in site(files) {
        std::fs::write(dir.join(&path), content).map_err(failed)?;
        written.push(dir.join(&path).display().to_string());
    }
    Ok(written)
}

/// The index: each file with the first paragraph of its doc, then
/// every item with its kind, type and file.
fn index(cx: &Ctx) -> String {
    let mut body = format!(
        "<h1>{}</h1>\n<h2>Files</h2>\n<ul>\n",
        escape(&cx.files[0].name)
    );
    for f in cx.files {
        let first = f.doc.as_ref().and_then(|d| d.text.split("\n\n").next());
        let about = first.map_or(String::new(), |p| format!(": {}", escape(p)));
        body.push_str(&format!(
            "<li>{} <a href=\"{}.html\">{}</a>{about}</li>\n",
            f.kind,
            page(&f.name),
            escape(&f.name)
        ));
    }
    body.push_str("</ul>\n");
    body.push_str(&items(cx));
    shell(cx, &cx.files[0].name, "", &body)
}

/// Every item: name, kind and type, linked to its documentation.
fn items(cx: &Ctx) -> String {
    let mut out = String::from("<h2>Items</h2>\n<table class=\"items\">\n");
    out.push_str("<tr><th>name</th><th>kind</th><th>type</th></tr>\n");
    for (fi, f) in cx.files.iter().enumerate() {
        for (ii, i) in f.items.iter().enumerate() {
            let href = cx.href(Target::Item { file: fi, item: ii });
            let (name, ty) = (cx.name(&i.name), escape(&i.ty));
            out.push_str(&format!(
                "<tr><td><a href=\"{href}\">{name}</a></td><td>{}</td><td><code>{ty}</code></td></tr>\n",
                i.kind
            ));
        }
    }
    out.push_str("</table>\n");
    out
}

/// The built-ins, from the catalog: name drawn, type, the rules that
/// define them.
fn builtins(cx: &Ctx) -> String {
    let mut body = String::from(
        "<h1>Built-ins</h1>\n<table class=\"items\">\n<tr><th>name</th><th>type</th><th>rules</th></tr>\n",
    );
    for b in xetal_catalog::BUILTINS.iter().filter(|b| b.implemented) {
        body.push_str(&format!(
            "<tr id=\"{}\"><td><code>{}</code></td><td><code>{}</code></td><td>{}</td></tr>\n",
            anchor(b.name),
            cx.name(b.name),
            escape(b.sig),
            escape(b.rule)
        ));
    }
    body.push_str("</table>\n");
    shell(cx, "built-ins", "", &body)
}
