//! Parsing case-file text into a validated `CaseFile`.

use std::fmt;

use crate::{CaseFile, Section, Status};

/// A malformed case file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseError {
    /// 1-based line number, or 0 for whole-file problems.
    pub line: usize,
    pub message: String,
}

impl fmt::Display for CaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "{}", self.message)
        } else {
            write!(f, "line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for CaseError {}

pub(crate) fn case_err(line: usize, message: impl Into<String>) -> CaseError {
    CaseError {
        line,
        message: message.into(),
    }
}

/// Split `text` into preamble comment lines and `(section, body)` pairs.
pub(crate) fn split_sections(text: &str) -> Result<CaseFile, CaseError> {
    let mut case = CaseFile {
        preamble: Vec::new(),
        sections: Vec::new(),
    };
    let mut open: Option<(Section, Vec<&str>)> = None;
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        if let Some(rest) = line.strip_prefix("==") {
            close(&mut case, open.take());
            open = Some((header(&case, rest.trim(), idx + 1)?, Vec::new()));
        } else if let Some((_, body)) = open.as_mut() {
            body.push(line);
        } else if line.trim().is_empty() || line.starts_with('#') {
            case.preamble.push(line.to_string());
        } else {
            return Err(case_err(
                idx + 1,
                "text before the first `== SECTION` header must be a `#` comment",
            ));
        }
    }
    close(&mut case, open);
    while case.preamble.last().is_some_and(|l| l.trim().is_empty()) {
        case.preamble.pop();
    }
    Ok(case)
}

fn header(case: &CaseFile, name: &str, line: usize) -> Result<Section, CaseError> {
    let section = Section::from_name(name)
        .ok_or_else(|| case_err(line, format!("unknown section `{name}`")))?;
    if case.get(section).is_some() {
        return Err(case_err(
            line,
            format!("duplicate section `{}`", section.name()),
        ));
    }
    Ok(section)
}

fn close(case: &mut CaseFile, open: Option<(Section, Vec<&str>)>) {
    if let Some((section, lines)) = open {
        let end = lines
            .iter()
            .rposition(|l| !l.trim().is_empty())
            .map_or(0, |i| i + 1);
        case.sections.push((section, lines[..end].join("\n")));
    }
}

/// Whole-file rules: SOURCE present and non-empty, RESULT and ERROR
/// exclusive, STATUS well-formed.
pub(crate) fn validate(case: &CaseFile) -> Result<(), CaseError> {
    match case.get(Section::Source) {
        None => return Err(case_err(0, "missing `== SOURCE` section")),
        Some(src) if src.trim().is_empty() => {
            return Err(case_err(0, "empty `== SOURCE` section"));
        }
        Some(_) => {}
    }
    if case.get(Section::Result).is_some() && case.get(Section::Error).is_some() {
        return Err(case_err(0, "`RESULT` and `ERROR` are mutually exclusive"));
    }
    parse_status(case.get(Section::Status)).map(|_| ())
}

pub(crate) fn parse_status(text: Option<&str>) -> Result<Status, CaseError> {
    let Some(text) = text else {
        return Ok(Status::Active);
    };
    match text.split_whitespace().next() {
        Some("active") => Ok(Status::Active),
        Some("pending") => Ok(Status::Pending),
        other => Err(case_err(
            0,
            format!(
                "STATUS must start with `active` or `pending`, found `{}`",
                other.unwrap_or("")
            ),
        )),
    }
}
