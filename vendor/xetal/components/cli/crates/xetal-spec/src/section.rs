//! Case-file section names and case status.

/// The sections a case file may contain, in canonical order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Source,
    Tokens,
    Render,
    Surface,
    Canonical,
    Core,
    Type,
    Result,
    Error,
    Status,
}

impl Section {
    pub const ALL: [Section; 10] = [
        Section::Source,
        Section::Tokens,
        Section::Render,
        Section::Surface,
        Section::Canonical,
        Section::Core,
        Section::Type,
        Section::Result,
        Section::Error,
        Section::Status,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Section::Source => "SOURCE",
            Section::Tokens => "TOKENS",
            Section::Render => "RENDER",
            Section::Surface => "SURFACE",
            Section::Canonical => "CANONICAL",
            Section::Core => "CORE",
            Section::Type => "TYPE",
            Section::Result => "RESULT",
            Section::Error => "ERROR",
            Section::Status => "STATUS",
        }
    }

    pub fn from_name(name: &str) -> Option<Section> {
        Section::ALL.into_iter().find(|s| s.name() == name)
    }

    /// Position in canonical order.
    pub fn rank(self) -> usize {
        self as usize
    }

    /// The `xetal` subcommand whose output this section pins, if any.
    pub fn stage(self) -> Option<&'static str> {
        match self {
            Section::Tokens => Some("lex"),
            Section::Render => Some("render"),
            Section::Surface => Some("parse"),
            Section::Canonical => Some("fmt"),
            Section::Core => Some("core"),
            Section::Type => Some("type"),
            Section::Result | Section::Error => Some("eval"),
            Section::Source | Section::Status => None,
        }
    }
}

/// Whether a case is expected to pass (`active`) or still fail (`pending`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Active,
    Pending,
}
