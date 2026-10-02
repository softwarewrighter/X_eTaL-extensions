//! Diagnostics reported where they were written.

use xetal_base::{Diagnostic, Severity};

use crate::Sources;

impl Sources {
    /// `text` with hidden namespaces written as file `file` writes them.
    pub fn as_written(&self, file: usize, text: &str) -> String {
        let mut names = self
            .files
            .get(file)
            .map(|f| f.written.clone())
            .unwrap_or_default();
        names.sort_by_key(|(hidden, _)| std::cmp::Reverse(hidden.len()));
        names.iter().fold(text.to_string(), |t, (hidden, written)| {
            let shown = if written.is_empty() {
                String::new()
            } else {
                format!("{written}:")
            };
            t.replace(&format!("{hidden}:"), &shown)
        })
    }

    /// `d` as printed: unchanged for a one-file program; otherwise its
    /// place is given as `FILE:LINE:COLUMN` in the file it came from.
    pub fn describe(&self, d: &Diagnostic) -> String {
        let (Some(span), true) = (d.span, self.file_count() > 1) else {
            return d.to_string();
        };
        let at = self.locate(span.start);
        let level = match d.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        let message = self.as_written(at.index, &d.message);
        let mut out = format!(
            "{level}[{}]: {message} at {}:{}:{}",
            d.code, at.file, at.line, at.col
        );
        for note in &d.notes {
            out.push_str(&format!("\n  note: {note}"));
        }
        out
    }
}
