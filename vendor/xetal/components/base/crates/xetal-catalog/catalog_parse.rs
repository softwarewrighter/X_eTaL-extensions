//! The shape of `builtins.toml`.

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Catalog {
    pub group: Vec<Group>,
    /// Absent once every listed built-in is implemented.
    #[serde(default)]
    pub later: Vec<Later>,
}

/// Implemented built-ins: `[name, arity, signature]` rows.
#[derive(Deserialize)]
pub struct Group {
    pub rule: String,
    pub entries: Vec<(String, usize, String)>,
}

/// Built-ins that exist but arrive with a later saga.
#[derive(Deserialize)]
pub struct Later {
    pub rule: String,
    pub names: Vec<String>,
}
