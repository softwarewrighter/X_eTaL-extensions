//! cell: a robot assembly cell, simulated -- arms, tools and parts that
//! build circuits in free air (docs/cell-design.md).
//!
//! The simulator (`world`, `command`, `parts`, `rig`, `kin`, `motion`)
//! has no threads, files or
//! clocks, so this crate is the extension, the MCP server's engine and,
//! compiled to WebAssembly, the live page's. The functions below are the
//! extension's: what the facade `lib/Cell.xtl` calls.

pub mod bender;
pub mod check;
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
pub mod wrap;

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

thread_local! {
    /// The planner's session: one cell kept between calls (`begin`,
    /// `attempt`, `frame`), so a planner can try a step and keep it only
    /// if the simulator takes it.
    static SESSION: std::cell::RefCell<world::World> = std::cell::RefCell::new(world::World::default());
}

/// A new session cell for a circuit: the reset's outcome as JSON.
fn begin(args: &[Value]) -> Result<Value, OwnedError> {
    let circuit = text(&args[0])?;
    let program = command::parse_program(&format!("reset circuit={circuit}"))
        .map_err(OwnedError::invalid_argument)?;
    SESSION.with(|s| {
        let mut w = world::World::default();
        let out = w.apply_all(&program);
        let ok = out.iter().all(|o| o.ok);
        *s.borrow_mut() = w;
        if ok {
            Ok(Value::Text(
                out.iter().map(|o| world::outcome_json(o) + "\n").collect(),
            ))
        } else {
            Err(OwnedError::failure(format!("no circuit {circuit}")))
        }
    })
}

/// Commands tried on the session's cell: kept if every one is ok
/// (`ok`, then each outcome as JSON, a line each), else the cell as it
/// was and `no COMMAND: why`.
fn attempt(args: &[Value]) -> Result<Value, OwnedError> {
    let program = command::parse_program(&text(&args[0])?).map_err(OwnedError::invalid_argument)?;
    SESSION.with(|s| {
        let mut w = s.borrow().clone();
        let out = w.apply_all(&program);
        if let Some(o) = out.iter().find(|o| !o.ok) {
            let why = o
                .refused
                .clone()
                .or_else(|| o.stopped.clone())
                .unwrap_or_default();
            return Ok(Value::Text(format!("no {}: {why}", o.command)));
        }
        *s.borrow_mut() = w;
        Ok(Value::Text(
            std::iter::once("ok\n".to_string())
                .chain(out.iter().map(|o| world::outcome_json(o) + "\n"))
                .collect(),
        ))
    })
}

/// Where a frame of the session's cell is now (`U1.3.tip`, `R1.2.tip`,
/// `C1`): x y z in the cell's frame, mm.
fn frame(args: &[Value]) -> Result<Value, OwnedError> {
    let name = text(&args[0])?;
    SESSION.with(|s| {
        let p = s.borrow().frame(&name).map_err(OwnedError::failure)?.p;
        Ok(Value::Text(format!(
            "{} {} {}",
            kin::clean((p[0] * 1000.0).round() / 1000.0),
            kin::clean((p[1] * 1000.0).round() / 1000.0),
            kin::clean((p[2] * 1000.0).round() / 1000.0)
        )))
    })
}

/// A part of the session's cell: its kind, and whether it came kitted
/// with its leads cut (`R1` -> `axial-resistor`, `BT1` ->
/// `battery-snap kitted`).
fn kind(args: &[Value]) -> Result<Value, OwnedError> {
    let id = text(&args[0])?;
    SESSION.with(|s| {
        let w = s.borrow();
        let p = w
            .part(&id)
            .ok_or_else(|| OwnedError::failure(format!("no part {id}")))?;
        let kitted = !p.leads.is_empty() && p.reformed.is_empty() && p.wire.is_none();
        Ok(Value::Text(format!(
            "{}{}",
            p.kind,
            if kitted { " kitted" } else { "" }
        )))
    })
}

/// A part's pins that are one conductor (a ground plane's posts), space
/// between (empty: none).
fn common(args: &[Value]) -> Result<Value, OwnedError> {
    let id = text(&args[0])?;
    SESSION.with(|s| {
        let w = s.borrow();
        let p = w
            .part(&id)
            .ok_or_else(|| OwnedError::failure(format!("no part {id}")))?;
        let k = p.kind_of().map_err(OwnedError::failure)?;
        Ok(Value::Text(k.common.join(" ")))
    })
}

/// The id the session's next wire will have (`W3`).
fn next_wire(_: &[Value]) -> Result<Value, OwnedError> {
    SESSION.with(|s| {
        let w = s.borrow();
        let n = (1..)
            .find(|n| w.part(&format!("W{n}")).is_none())
            .unwrap_or(1);
        Ok(Value::Text(format!("W{n}")))
    })
}

/// The pins a pin of the session's cell is soldered to, space between
/// (empty: none yet).
fn joined(args: &[Value]) -> Result<Value, OwnedError> {
    let pin = text(&args[0])?;
    SESSION.with(|s| {
        let w = s.borrow();
        let others: Vec<String> = w
            .joints
            .iter()
            .filter(|j| j.result.is_some())
            .filter_map(|j| {
                if j.a == pin {
                    Some(j.b.clone())
                } else if j.b == pin {
                    Some(j.a.clone())
                } else {
                    None
                }
            })
            .collect();
        Ok(Value::Text(others.join(" ")))
    })
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
        begin: 1, "Char -> Char", "A new session cell for a circuit (the planner's): the reset's outcome.";
        attempt: 1, "Char -> Char", "Commands tried on the session's cell: kept if all are ok (ok, then the outcomes), else the cell as it was and: no COMMAND: why.";
        frame: 1, "Char -> Char", "Where a frame of the session's cell is now: x y z, mm.";
        kind: 1, "Char -> Char", "A part of the session's cell: its kind, then kitted if its leads came cut.";
        next_wire: 0, "Unit -> Char", "The id the session's next wire will have.";
        common: 1, "Char -> Char", "A part's pins that are one conductor (a ground plane's posts), space between.";
        joined: 1, "Char -> Char", "The pins a pin of the session's cell is soldered to, space between.";
    }
}
