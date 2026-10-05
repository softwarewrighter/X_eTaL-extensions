//! A doc comment's prose: paragraphs (split by a blank line), inline
//! code between backquotes drawn as code, an indented paragraph a code
//! block; and its examples, drawn as a session transcript.

use crate::escape;

/// How a piece of code is drawn (decorated and linked by the caller).
pub type Draw<'a> = &'a dyn Fn(&str) -> String;

/// `text` as HTML paragraphs.
pub fn prose(text: &str, draw: Draw) -> String {
    let mut out = String::new();
    for para in text.split("\n\n").filter(|p| !p.trim().is_empty()) {
        let lines: Vec<&str> = para.lines().collect();
        if lines.iter().all(|l| l.starts_with("  ")) {
            let code: Vec<&str> = lines.iter().map(|l| &l[2..]).collect();
            out.push_str(&format!(
                "<pre class=\"code\">{}</pre>\n",
                draw(&code.join("\n"))
            ));
        } else {
            out.push_str(&format!("<p>{}</p>\n", inline(para, draw)));
        }
    }
    out
}

/// One example: the code typed (indented six spaces, as an APL session
/// shows input), then what it prints.
pub fn example(code: &str, output: &str) -> String {
    let shown = match output.is_empty() {
        true => String::new(),
        false => format!("\n{}", escape(output)),
    };
    format!("<pre class=\"example\"><span class=\"prompt\">      </span>{code}{shown}</pre>\n")
}

/// A paragraph: text escaped, `code` drawn (an unmatched backquote is
/// text).
fn inline(para: &str, draw: Draw) -> String {
    let pieces: Vec<&str> = para.split('`').collect();
    let last = pieces.len() - 1;
    let open = pieces.len().is_multiple_of(2);
    pieces
        .iter()
        .enumerate()
        .map(|(i, piece)| match (i % 2, open && i == last) {
            (_, true) => format!("`{}", escape(piece)),
            (1, false) => format!("<code>{}</code>", draw(piece)),
            _ => escape(piece),
        })
        .collect()
}
