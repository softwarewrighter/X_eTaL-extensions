//! The parsed case file: section access, editing and rendering.

use crate::parse::{parse_status, split_sections, validate};
use crate::{CaseError, Section, Status};

/// A parsed case file. Sections keep their file order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaseFile {
    pub preamble: Vec<String>,
    pub sections: Vec<(Section, String)>,
}

impl CaseFile {
    pub fn parse(text: &str) -> Result<CaseFile, CaseError> {
        let case = split_sections(text)?;
        validate(&case)?;
        Ok(case)
    }

    pub fn get(&self, section: Section) -> Option<&str> {
        self.sections
            .iter()
            .find(|(s, _)| *s == section)
            .map(|(_, body)| body.as_str())
    }

    pub fn source(&self) -> &str {
        self.get(Section::Source).unwrap_or("")
    }

    pub fn status(&self) -> Status {
        parse_status(self.get(Section::Status)).unwrap_or(Status::Active)
    }

    /// Replace a section body, or insert the section in canonical order.
    pub fn set(&mut self, section: Section, body: String) {
        if let Some(slot) = self.sections.iter_mut().find(|(s, _)| *s == section) {
            slot.1 = body;
            return;
        }
        let at = self
            .sections
            .iter()
            .position(|(s, _)| s.rank() > section.rank())
            .unwrap_or(self.sections.len());
        self.sections.insert(at, (section, body));
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in &self.preamble {
            out.push_str(line);
            out.push('\n');
        }
        for (section, body) in &self.sections {
            out.push_str("== ");
            out.push_str(section.name());
            out.push('\n');
            if !body.is_empty() {
                out.push_str(body);
                out.push('\n');
            }
        }
        out
    }
}
