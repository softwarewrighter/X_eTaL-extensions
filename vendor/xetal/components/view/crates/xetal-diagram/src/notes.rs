//! The notes file: `== SOURCE` (the line to draw), `== TITLE`, and a
//! `== NOTE anchor` section per callout (`== NOTE anchor #n` for the
//! n-th occurrence), whose first line is the callout's title and the
//! rest its text.

use xetal_base::Diagnostic;

pub(crate) struct Note {
    pub anchor: String,
    pub nth: usize,
    pub title: String,
    pub body: String,
}

pub(crate) struct Notes {
    pub source: String,
    pub title: String,
    pub notes: Vec<Note>,
}

/// The sections of `text`, each a heading and its lines.
fn sections(text: &str) -> Vec<(&str, Vec<&str>)> {
    let mut out: Vec<(&str, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        match (line.strip_prefix("== "), out.last_mut()) {
            (Some(head), _) => out.push((head, Vec::new())),
            (None, Some((_, body))) => body.push(line),
            (None, None) => {}
        }
    }
    out
}

pub(crate) fn parse(text: &str) -> Result<Notes, Diagnostic> {
    let bad = |m: String| Diagnostic::new("bad-notes", m);
    let mut out = Notes {
        source: String::new(),
        title: String::new(),
        notes: Vec::new(),
    };
    for (head, body) in sections(text) {
        let joined = body.join("\n").trim().to_string();
        match head.split_once(' ') {
            _ if head == "SOURCE" => out.source = joined,
            _ if head == "TITLE" => out.title = joined,
            Some(("NOTE", anchor)) => out.notes.push(note(anchor, &body)),
            _ => return Err(bad(format!("unknown section `== {head}`"))),
        }
    }
    if out.source.is_empty() {
        return Err(bad(
            "a notes file needs `== SOURCE`, the line to draw".into()
        ));
    }
    Ok(out)
}

fn note(anchor: &str, body: &[&str]) -> Note {
    let (anchor, nth) = match anchor.rsplit_once(" #").map(|(a, n)| (a, n.parse())) {
        Some((a, Ok(n))) => (a, n),
        _ => (anchor, 1),
    };
    let mut lines = body.iter().map(|l| l.trim()).filter(|l| !l.is_empty());
    Note {
        anchor: anchor.to_string(),
        nth,
        title: lines.next().unwrap_or_default().to_string(),
        body: lines.collect::<Vec<_>>().join(" "),
    }
}
