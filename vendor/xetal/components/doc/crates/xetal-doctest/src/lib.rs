//! `xetal doc --test` (S10): run the `## >>` examples of a file, each
//! doc block as one session, and compare what each prints with the
//! output shown under it, as rustdoc runs doc tests.

mod blocks;
mod compare;
mod report;
mod session;

pub use report::report;

/// One example run: where it is, what it showed and what it printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Outcome {
    /// The documented item, or None for the file's own doc.
    pub item: Option<String>,
    pub code: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Run every example of the file `text` (named `name`).
pub fn run(name: &str, text: &str) -> Result<Vec<Outcome>, xetal_base::Diagnostic> {
    let file = xetal_doc::model(name, text)?.into_iter().next();
    let Some(file) = file else {
        return Ok(Vec::new());
    };
    let context = match file.kind {
        "program" => text,
        _ => "",
    };
    Ok(blocks::blocks(&file)
        .into_iter()
        .flat_map(|(item, examples)| session::run_block(context, item, &examples))
        .collect())
}

/// `xetal doc --test FILE`: the report when every example passes; on a
/// failure the report is printed and the error says how many failed.
pub fn test(name: &str, text: &str) -> Result<String, xetal_base::Diagnostic> {
    let outcomes = run(name, text)?;
    let (text, ok) = report(name, &outcomes);
    if ok {
        return Ok(text.trim_end().to_string());
    }
    print!("{text}");
    let failed = outcomes.iter().filter(|o| !o.passed).count();
    let message = format!("{failed} doc example(s) failed in {name}");
    Err(xetal_base::Diagnostic::new("doc-test-failed", message))
}
