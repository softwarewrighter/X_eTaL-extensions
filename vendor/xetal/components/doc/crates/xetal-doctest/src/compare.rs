//! Comparing what an example printed with the output shown (S10): an
//! `error[code]` line matches an error with that code, other lines the
//! output exactly; warnings are not compared.

/// The lines of `text` without trailing blanks or trailing empty lines.
fn lines(text: &str) -> Vec<&str> {
    let mut all: Vec<&str> = text.lines().map(str::trim_end).collect();
    while all.last() == Some(&"") {
        all.pop();
    }
    all
}

/// The error lines of a run's stderr.
fn errors(err: &str) -> Vec<&str> {
    lines(err)
        .into_iter()
        .filter(|l| l.starts_with("error["))
        .collect()
}

/// Whether the run (`out`, `err`) prints what `expected` shows.
pub(crate) fn matches(expected: &str, out: &str, err: &str) -> bool {
    let (want_err, want_out): (Vec<&str>, Vec<&str>) = lines(expected)
        .into_iter()
        .partition(|l| l.starts_with("error["));
    let got = errors(err);
    let errors_match = match want_err.is_empty() {
        true => got.is_empty(),
        false => want_err
            .iter()
            .all(|w| got.iter().any(|g| g.starts_with(w))),
    };
    errors_match && lines(out) == want_out
}

/// What the run printed, for a report: its output, then its errors.
pub(crate) fn actual(out: &str, err: &str) -> String {
    let mut all = lines(out);
    all.extend(errors(err));
    all.join("\n")
}
