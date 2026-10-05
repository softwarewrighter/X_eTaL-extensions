//! The model as JSON (written by hand: no dependency), two spaces an
//! indent, keys in a fixed order so goldens are stable.

use xetal_doccom::Doc;

use crate::model::{DocFile, Item};

/// The files as a JSON document.
pub fn to_json(files: &[DocFile]) -> String {
    let files: Vec<String> = files.iter().map(file).collect();
    format!("{{\n  \"files\": [\n{}\n  ]\n}}\n", files.join(",\n"))
}

fn file(f: &DocFile) -> String {
    let items: Vec<String> = f.items.iter().map(item).collect();
    let imports: Vec<String> = f
        .imports
        .iter()
        .map(|i| {
            let files: Vec<String> = i.files.iter().map(|f| string(f)).collect();
            let files = format!("[{}]", files.join(", "));
            inline(&[
                ("alias", string(&i.alias)),
                ("spec", string(&i.spec)),
                ("files", files),
            ])
        })
        .collect();
    let fields = [
        ("name", string(&f.name)),
        ("kind", string(f.kind)),
        ("doc", doc_text(&f.doc)),
        ("imports", format!("[{}]", imports.join(", "))),
        ("items", format!("[\n{}\n      ]", items.join(",\n"))),
    ];
    object(&fields, 4)
}

fn item(i: &Item) -> String {
    let uses: Vec<String> = i
        .uses
        .iter()
        .map(|u| {
            let (w, f, it) = (string(&u.written), string(&u.file), string(&u.item));
            inline(&[("written", w), ("file", f), ("item", it)])
        })
        .collect();
    let examples: Vec<String> = i
        .examples()
        .iter()
        .map(|e| {
            format!(
                "{{\"code\": {}, \"output\": {}}}",
                string(&e.code),
                string(&e.output)
            )
        })
        .collect();
    let fields = [
        ("name", string(&i.name)),
        ("kind", string(i.kind)),
        ("public", i.public.to_string()),
        ("type", string(&i.ty)),
        ("line", i.line.to_string()),
        (
            "section",
            i.section.as_deref().map_or("null".into(), string),
        ),
        ("doc", doc_text(&i.doc)),
        ("examples", format!("[{}]", examples.join(", "))),
        ("source", string(&i.source)),
        ("uses", format!("[{}]", uses.join(", "))),
    ];
    object(&fields, 8)
}

/// `{"key": value, ...}` on one line.
fn inline(fields: &[(&str, String)]) -> String {
    let shown: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("\"{k}\": {v}"))
        .collect();
    format!("{{{}}}", shown.join(", "))
}

fn doc_text(doc: &Option<Doc>) -> String {
    doc.as_ref().map_or("null".into(), |d| string(&d.text))
}

/// `{ "key": value, ... }` at `indent` spaces, one field a line.
fn object(fields: &[(&str, String)], indent: usize) -> String {
    let pad = " ".repeat(indent);
    let lines: Vec<String> = fields
        .iter()
        .map(|(k, v)| format!("{pad}  \"{k}\": {v}"))
        .collect();
    format!("{pad}{{\n{}\n{pad}}}", lines.join(",\n"))
}

/// A JSON string literal.
fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
