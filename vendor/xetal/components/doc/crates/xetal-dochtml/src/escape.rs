//! Escaping text, and the anchors and page names of the site.

/// `text` safe inside HTML text and attribute values.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// An element id for `name`: letters, digits, `_` and `-` as they are,
/// `:` as `.`, anything else as `-` and its code in hex (`s:i_f<` is
/// `s.i_f-3c`), so it needs no escaping in a URL.
pub fn anchor(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            c if c.is_ascii_alphanumeric() || c == '_' => c.to_string(),
            ':' => ".".to_string(),
            c => format!("-{:x}", c as u32),
        })
        .collect()
}

/// The page (without `.html`) for the file reported as `name`: its
/// path without leading `./` and `../`, `/` as `-`; text given on the
/// command line (`-e`) is `program`.
pub fn page(name: &str) -> String {
    if name == "-e" {
        return "program".into();
    }
    let parts: Vec<&str> = name
        .split('/')
        .skip_while(|p| matches!(*p, "" | "." | ".."))
        .collect();
    parts.join("-")
}
