//! Which `##` block documents what: the block directly above a
//! definition, the file's first block when a blank line follows it, and
//! the `###` section a line falls under.

use crate::Doc;

/// The text of a doc comment line without its `##` (and one space), or
/// None for any other line, a `###` section title included.
fn doc_line(line: &str) -> Option<&str> {
    let rest = line.trim_start().strip_prefix("##")?;
    match rest.chars().next() {
        None => Some(""),
        Some(' ') => Some(&rest[1..]),
        Some(_) => None,
    }
}

/// The doc of the definition starting on `line` (from 1): the block of
/// `##` lines directly above it.
pub fn doc_above(text: &str, line: usize) -> Option<Doc> {
    let lines: Vec<&str> = text.lines().collect();
    let end = line.checked_sub(1)?.min(lines.len());
    let start = (0..end)
        .rev()
        .take_while(|&i| doc_line(lines[i]).is_some())
        .last()?;
    let block: Vec<&str> = lines[start..end]
        .iter()
        .filter_map(|l| doc_line(l))
        .collect();
    Some(Doc::from_lines(&block))
}

/// The file's own doc: a `##` block at the top of the file (after any
/// `#!` line and blank lines), when a blank line (or the end of the
/// file) follows it rather than a definition.
pub fn file_doc(text: &str) -> Option<Doc> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .position(|l| !(l.trim().is_empty() || l.starts_with("#!")))?;
    doc_line(lines[start])?;
    let len = lines[start..]
        .iter()
        .take_while(|l| doc_line(l).is_some())
        .count();
    let after = lines.get(start + len).map_or("", |l| l.trim());
    let block: Vec<&str> = lines[start..start + len]
        .iter()
        .filter_map(|l| doc_line(l))
        .collect();
    after.is_empty().then(|| Doc::from_lines(&block))
}

/// The title of the `### Title` section line `line` (from 1) falls
/// under, if any.
pub fn section_at(text: &str, line: usize) -> Option<String> {
    text.lines()
        .take(line.saturating_sub(1))
        .filter_map(|l| l.trim_start().strip_prefix("### "))
        .last()
        .map(|t| t.trim().to_string())
}
