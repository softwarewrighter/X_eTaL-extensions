//! A macro-phase error and the file it was found in.

use xetal_base::Diagnostic;

#[derive(Debug, Clone)]
pub struct MacroError {
    pub diagnostic: Diagnostic,
    /// The file the error was found in, and its text.
    pub file: String,
    pub text: String,
    /// True when that file is the program itself (not a library).
    pub main: bool,
}

impl MacroError {
    /// The error as printed: as usual in the program, and at
    /// `FILE:LINE:COLUMN` in a library.
    pub fn describe(&self) -> String {
        let d = &self.diagnostic;
        match (self.main, d.span) {
            (false, Some(span)) => {
                let before = &self.text[..span.start.min(self.text.len())];
                let line = before.matches('\n').count() + 1;
                let col = before[before.rfind('\n').map_or(0, |n| n + 1)..]
                    .chars()
                    .count()
                    + 1;
                format!(
                    "error[{}]: {} at {}:{line}:{col}",
                    d.code, d.message, self.file
                )
            }
            _ => d.to_string(),
        }
    }
}
