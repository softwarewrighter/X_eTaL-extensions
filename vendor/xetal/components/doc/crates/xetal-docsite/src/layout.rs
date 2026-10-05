//! The frame of every page: head, the side bar (the files, and what is
//! on this page), the theme switch.

use xetal_base::LANG_NAME;
use xetal_dochtml::{escape, page};

use crate::context::Ctx;

/// A whole page: `title`, the side bar's `toc` (this page's own
/// entries, HTML list items) and the `body`.
pub(crate) fn shell(cx: &Ctx, title: &str, toc: &str, body: &str) -> String {
    let side = sidebar(cx, toc);
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{} - {LANG_NAME} doc</title>\n<link rel=\"stylesheet\" href=\"style.css\">\n\
         <script src=\"theme.js\"></script>\n</head>\n<body>\n<nav class=\"side\">\n{side}</nav>\n\
         <main>\n<button class=\"theme\" title=\"Light or dark\">theme</button>\n{body}</main>\n\
         </body>\n</html>\n",
        escape(title)
    )
}

/// The side bar: home, every file by kind, then this page's entries.
fn sidebar(cx: &Ctx, toc: &str) -> String {
    let mut out = format!("<a class=\"home\" href=\"index.html\">{LANG_NAME} doc</a>\n");
    out.push_str("<h2>Files</h2>\n<ul>\n");
    for f in cx.files {
        let name = escape(short(&f.name));
        out.push_str(&format!(
            "<li><a href=\"{}.html\">{name}</a></li>\n",
            page(&f.name)
        ));
    }
    out.push_str("<li><a href=\"builtins.html\">built-ins</a></li>\n</ul>\n");
    if !toc.is_empty() {
        out.push_str(&format!("<h2>On this page</h2>\n<ul>\n{toc}</ul>\n"));
    }
    out
}

/// A file's name without its directories.
pub(crate) fn short(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}
