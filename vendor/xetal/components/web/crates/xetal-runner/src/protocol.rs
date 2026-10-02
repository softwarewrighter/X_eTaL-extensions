//! The messages between the page and the worker.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    pub src: String,
    pub seed: u64,
    pub mode: Mode,
    /// Print every array result boxed (the Boxed toggle, `--box`).
    pub boxed: bool,
    pub files: Vec<(String, String)>,
}

/// How to run: the output alone, or as a notebook (each statement shown
/// before its output), the whole program or up to statement k.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Run,
    Notebook(Option<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    Out(String),
    Err(String),
    Picture(String),
    Wrote(String, String),
    Done,
    /// A statement's source, in a notebook, just before it runs.
    Source(String),
    /// The worker is listening: the page sends it the program now (a
    /// message sent before would be lost while its wasm loads).
    Ready,
}

/// Fields as `LEN:TEXT`, one after another (LEN in bytes), so any text
/// at all survives the trip.
fn frame(fields: &[&str]) -> String {
    fields.iter().map(|f| format!("{}:{f}", f.len())).collect()
}

/// The fields of a framed message, or None when it is not one.
fn fields(mut text: &str) -> Option<Vec<&str>> {
    let mut out = Vec::new();
    while !text.is_empty() {
        let (len, rest) = text.split_once(':')?;
        let len: usize = len.parse().ok()?;
        out.push(rest.get(..len)?);
        text = &rest[len..];
    }
    Some(out)
}

impl Request {
    pub fn encode(&self) -> String {
        let mode = match self.mode {
            Mode::Run => "r".into(),
            Mode::Notebook(None) => "n".into(),
            Mode::Notebook(Some(k)) => format!("n{k}"),
        };
        let seed = self.seed.to_string();
        let boxed = if self.boxed { "b" } else { "-" };
        let mut all = vec![self.src.as_str(), seed.as_str(), mode.as_str(), boxed];
        all.extend(
            self.files
                .iter()
                .flat_map(|(p, t)| [p.as_str(), t.as_str()]),
        );
        frame(&all)
    }

    pub fn decode(text: &str) -> Option<Request> {
        let f = fields(text)?;
        let (src, seed, files) = (f.first()?, f.get(1)?, f.get(4..)?);
        let boxed = *f.get(3)? == "b";
        let mode = match f.get(2)?.strip_prefix('n') {
            None => (*f.get(2)? == "r").then_some(Mode::Run)?,
            Some("") => Mode::Notebook(None),
            Some(k) => Mode::Notebook(Some(k.parse().ok()?)),
        };
        (files.len() % 2 == 0).then(|| Request {
            src: src.to_string(),
            seed: seed.parse().unwrap_or(0),
            mode,
            boxed,
            files: files
                .chunks(2)
                .map(|c| (c[0].into(), c[1].into()))
                .collect(),
        })
    }
}

impl Event {
    pub fn encode(&self) -> String {
        match self {
            Event::Out(t) => frame(&["o", t]),
            Event::Err(t) => frame(&["e", t]),
            Event::Picture(t) => frame(&["p", t]),
            Event::Wrote(p, t) => frame(&["w", p, t]),
            Event::Done => frame(&["d"]),
            Event::Ready => frame(&["r"]),
            Event::Source(t) => frame(&["s", t]),
        }
    }

    pub fn decode(text: &str) -> Option<Event> {
        match fields(text)?.as_slice() {
            ["o", t] => Some(Event::Out(t.to_string())),
            ["e", t] => Some(Event::Err(t.to_string())),
            ["p", t] => Some(Event::Picture(t.to_string())),
            ["w", p, t] => Some(Event::Wrote(p.to_string(), t.to_string())),
            ["d"] => Some(Event::Done),
            ["r"] => Some(Event::Ready),
            ["s", t] => Some(Event::Source(t.to_string())),
            _ => None,
        }
    }
}
