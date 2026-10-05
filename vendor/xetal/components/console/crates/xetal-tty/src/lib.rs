//! The terminal a program runs on (QD6): where `[]E_RR` writes, where
//! `[]K_EY` reads one key from, and what `[]T_E` reports. The host
//! installs one (a raw-mode terminal at the CLI, the live demo's
//! worker); without one it is [`Plain`]: standard error, a key read
//! from standard input, 24 rows of 80 columns.

mod current;
mod forward;
mod plain;

pub use current::{error, facts, install, key};
pub use forward::Forward;
pub use plain::{Plain, key_name};

/// What a program can ask of its terminal.
pub trait Tty: Send + Sync {
    /// Write a line to standard error.
    fn error(&self, line: &str);
    /// One key, no Enter, by name (`key_name`).
    fn key(&self) -> Result<String, String>;
    /// Rows, columns, 1 when output is a terminal, 1 when it does
    /// screen control.
    fn facts(&self) -> [i64; 4];
}
