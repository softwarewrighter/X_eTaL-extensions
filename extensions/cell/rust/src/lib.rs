//! cell: a robot assembly cell, simulated -- arms, tools and parts that
//! build circuits in free air (docs/cell-design.md).
//!
//! The simulator (`world`, `command`, `parts`, `rig`, `kin`, `motion`)
//! has no threads, files or
//! clocks, so this crate is the extension, the MCP server's engine and,
//! compiled to WebAssembly, the live page's. The functions below are the
//! extension's: what the facade `lib/Cell.xtl` calls.

pub mod bender;
pub mod collide;
pub mod command;
pub mod heat;
pub mod identify;
pub mod kin;
pub mod layout;
pub mod leads;
pub mod marks;
pub mod model;
pub mod motion;
pub mod parts;
pub mod pincers;
pub mod rig;
pub mod scene;
pub mod scope;
pub mod sleeve;
pub mod solder;
pub mod stations;
pub mod tester;
mod thermal;
pub mod weld;
pub mod wire;
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

/// A kind's pin tips in its own frame, mm: three numbers a pin, a line
/// each, in pin order -- for a planner written in X_eTaL (n_umbers).
fn tips(args: &[Value]) -> Result<Value, OwnedError> {
    let kind = parts::kind(text(&args[0])?.trim()).map_err(OwnedError::invalid_argument)?;
    let lines: Vec<String> = kind
        .pins
        .iter()
        .map(|p| {
            let t = p.lead.last().copied().unwrap_or([0.0; 3]);
            format!(
                "{} {} {}",
                command::num(t[0]),
                command::num(t[1]),
                command::num(t[2])
            )
        })
        .collect();
    Ok(Value::Text(lines.join("\n")))
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

/// A program run on a new cell: each command's outcome as JSON, one a
/// line, stopping after the first that is not ok.
fn run(args: &[Value]) -> Result<Value, OwnedError> {
    let program = command::parse_program(&text(&args[0])?).map_err(OwnedError::invalid_argument)?;
    let mut w = world::World::default();
    let out = w.apply_all(&program);
    Ok(Value::Text(
        out.iter().map(|o| world::outcome_json(o) + "\n").collect(),
    ))
}

/// A program run on a new cell: the cell's state after it, as JSON. A
/// command not ok is an error.
fn state(args: &[Value]) -> Result<Value, OwnedError> {
    let program = command::parse_program(&text(&args[0])?).map_err(OwnedError::invalid_argument)?;
    let mut w = world::World::default();
    if let Some(o) = w.apply_all(&program).into_iter().find(|o| !o.ok) {
        let why = o.refused.or(o.stopped).unwrap_or_default();
        return Err(OwnedError::failure(format!("{}: {why}", o.command)));
    }
    Ok(Value::Text(w.to_json()))
}

/// What a model of the arm must be, as JSON: its nodes (one a joint's
/// pivot), links and check poses, with the rig.
fn arm_model(_: &[Value]) -> Result<Value, OwnedError> {
    let rig = rig::Rig::arm().map_err(OwnedError::failure)?;
    Ok(Value::Text(model::arm(&rig).to_string()))
}

/// A program run on a new cell, written as a scene for the players
/// (the three extension's cell player): name, then the program.
fn scene(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    let program = command::parse_program(&text(&args[1])?).map_err(OwnedError::invalid_argument)?;
    let mut w = world::World::default();
    if let Some(o) = w.apply_all(&program).into_iter().find(|o| !o.ok) {
        let why = o.refused.or(o.stopped).unwrap_or_default();
        return Err(OwnedError::failure(format!("{}: {why}", o.command)));
    }
    Ok(Value::Text(
        scene::calls(&name, &w).map_err(OwnedError::failure)?,
    ))
}

/// The arm's rig as JSON.
fn rig(_: &[Value]) -> Result<Value, OwnedError> {
    Ok(Value::Text(
        rig::Rig::arm().map_err(OwnedError::failure)?.to_json(),
    ))
}

xetal_ext_sdk::xetal_extension! {
    name: "cell",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        about: 0, "Unit -> Char", "The extension's name and version.";
        parts: 0, "Unit -> Char", "The parts library, one line a kind: kind, pins, title, example.";
        part: 1, "Char -> Char", "One part kind, its pins worked out, as JSON.";
        tips: 1, "Char -> Char", "A kind's pin tips in its own frame: x y z a line, in pin order.";
        commands: 0, "Unit -> Char", "The command list, one line a command: its synopsis and what it does.";
        parse: 1, "Char -> Char", "A program (text, or JSON lines) checked against the command table: each command as JSON, one a line.";
        run: 1, "Char -> Char", "A program run on a new cell: each command's outcome as JSON, one a line, stopping after the first not ok.";
        state: 1, "Char -> Char", "A program run on a new cell: its state after, as JSON; a command not ok is an error.";
        arm_model: 0, "Unit -> Char", "What a model of the arm must be, as JSON: nodes (one a joint's pivot, local z its axis), links, check poses, the rig.";
        scene: 2, "Char -> Char -> Char", "name program: the program run on a new cell, written as a scene for the players (cell(...), rig, track, step, ...).";
        rig: 0, "Unit -> Char", "The arm's rig as JSON: its joints' DH parameters, limits, speeds and home.";
    }
}
