//! cell: a robot assembly cell, simulated -- arms, tools and parts that
//! build circuits in free air (docs/cell-design.md).
//!
//! The simulator (`world`, `command`, `parts`) has no threads, files or
//! clocks, so this crate is the extension, the MCP server's engine and,
//! compiled to WebAssembly, the live page's. The functions below are the
//! extension's: what the facade `lib/Cell.xtl` calls.

pub mod command;
pub mod parts;
pub mod world;

use xetal_ext_sdk::{OwnedError, Value, text};

/// The extension's name and version, as text.
#[allow(clippy::unnecessary_wraps)] // every handler has the SDK's type
fn about(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(format!("cell {}", env!("CARGO_PKG_VERSION"))))
}

/// The parts library, one line a kind.
fn parts(_: &[Value]) -> Result<Value, OwnedError> {
    let lib = parts::library().map_err(OwnedError::failure)?;
    Ok(Value::Text(lib.iter().map(|k| k.line() + "\n").collect()))
}

/// One part kind, pins worked out, as JSON.
fn part(args: &[Value]) -> Result<Value, OwnedError> {
    let kind = parts::kind(text(&args[0])?.trim()).map_err(OwnedError::invalid_argument)?;
    Ok(Value::Text(kind.to_json()))
}

/// The command list, one line a command.
#[allow(clippy::unnecessary_wraps)]
fn commands(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(command::list()))
}

/// A program (text or JSON lines) checked against the command table:
/// each command as JSON, one a line.
fn parse(args: &[Value]) -> Result<Value, OwnedError> {
    let program = command::parse_program(&text(&args[0])?).map_err(OwnedError::invalid_argument)?;
    Ok(Value::Text(
        program
            .iter()
            .map(|c| c.to_json().to_string() + "\n")
            .collect(),
    ))
}

xetal_ext_sdk::xetal_extension! {
    name: "cell",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        about: 0, "Unit -> Char", "The extension's name and version.";
        parts: 0, "Unit -> Char", "The parts library, one line a kind: kind, pins, title, example.";
        part: 1, "Char -> Char", "One part kind, its pins worked out, as JSON.";
        commands: 0, "Unit -> Char", "The command list, one line a command: its synopsis and what it does.";
        parse: 1, "Char -> Char", "A program (text, or JSON lines) checked against the command table: each command as JSON, one a line.";
    }
}
