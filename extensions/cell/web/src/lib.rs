//! The assembly cell, live: the session a page drives. One cell, kept
//! between calls; a command (or a program of them, the same text as
//! anywhere) runs on it and comes back as outcomes and a new scene for
//! the cell player. X_eTaL runs in the page too, with the cell
//! extension linked in (`ce:`): a program's scene can be shown.
//!
//! Plain Rust, so the tests run it natively; `main.rs` hands it to the
//! page (wasm-bindgen).

use serde_json::{Value as Json, json};
use std::cell::RefCell;
use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::scene;
use xetal_ext_cell::world::{World, outcome_json};
use xetal_ext_shell::chrome::footer_markup;
use xetal_ext_shell::run::{Linked, install_with, run};
use xetal_ext_shell::source::markup;

thread_local! {
    static CELL: RefCell<World> = RefCell::new(World::default());
}

/// The programs the page offers, as `programs/NAME.cell` has them.
pub const PROGRAMS: [(&str, &str); 6] = [
    ("solder", include_str!("../../programs/solder.cell")),
    (
        "solder-stations",
        include_str!("../../programs/solder-stations.cell"),
    ),
    ("weld", include_str!("../../programs/weld.cell")),
    (
        "weld-stations",
        include_str!("../../programs/weld-stations.cell"),
    ),
    ("pick", include_str!("../../programs/pick.cell")),
    ("arms", include_str!("../../programs/arms.cell")),
];

/// The X_eTaL program the page starts with: the cell from X_eTaL.
pub const EXAMPLE: &str = "\"ce:\" u_se< \"Cell\"\n\n# a scene written by the simulator, from X_eTaL in this page\n\"from X_eTaL\" ce:s_cene \"reset circuit=demo\\ntool D tweezers\\npick D C1 grasp=body\\nmove D to=120,20,60\\nrelease D C1\"\n";

/// cell, linked statically, and its facade.
#[must_use]
pub fn linked() -> Vec<Linked> {
    vec![Linked {
        descriptor: xetal_ext_cell::__xetal_extension::descriptor,
        facade: ("Cell.xtl", include_str!("../../lib/Cell.xtl")),
    }]
}

/// Registers cell with X_eTaL; once, before any [`xetal`]. How many
/// functions it brought.
pub fn install() -> usize {
    install_with(&linked(), &[]).len()
}

/// A new cell, its tray laid out for `circuit`: the outcome, as JSON.
#[must_use]
pub fn reset(circuit: &str) -> String {
    let text = format!("reset circuit={circuit}");
    CELL.with(|c| {
        *c.borrow_mut() = World::default();
        command_on(&mut c.borrow_mut(), &text)
    })
}

/// An empty cell, nothing run on it yet (for a program that begins
/// with its own reset).
pub fn clear() {
    CELL.with(|c| *c.borrow_mut() = World::default());
}

/// Runs commands (text: a line each, `;` between, `#` comments; or JSON
/// lines) on the cell, stopping after the first not ok: `{"outcomes":
/// [...], "t0": when the first began, "t1": the cell's time now}`, or
/// `{"error": ...}` when the text is not commands.
#[must_use]
pub fn command(text: &str) -> String {
    CELL.with(|c| command_on(&mut c.borrow_mut(), text))
}

fn command_on(w: &mut World, text: &str) -> String {
    let program = match parse_program(text) {
        Ok(p) => p,
        Err(e) => return json!({ "error": e }).to_string(),
    };
    let t0 = w.t;
    let outcomes: Vec<Json> = w
        .apply_all(&program)
        .iter()
        .map(|o| serde_json::from_str(&outcome_json(o)).unwrap_or(Json::Null))
        .collect();
    json!({ "outcomes": outcomes, "t0": t0, "t1": w.t }).to_string()
}

/// The cell's timeline as a scene for the cell player.
#[must_use]
pub fn scene(name: &str) -> String {
    CELL.with(|c| scene::calls(name, &c.borrow()).unwrap_or_else(|e| format!("// {e}\n")))
}

/// What the controls need: the time, each arm (joints, tool, what it
/// holds, its tip as x y z roll pitch yaw), each part (kind, where,
/// held by, lost, its grasp frames), the stage, the vise, the
/// timeline (each command, its times, rewound or not).
#[must_use]
pub fn state() -> String {
    CELL.with(|c| c.borrow().summary().to_string())
}

/// A program the page offers, by name (`pick`, `arms`); empty for
/// another name.
#[must_use]
pub fn program(name: &str) -> String {
    PROGRAMS
        .iter()
        .find(|(n, _)| *n == name)
        .map_or_else(String::new, |(_, t)| (*t).to_string())
}

/// Runs an X_eTaL program in the page (cell linked in): `{"out": ...,
/// "err": ..., "millis": ...}`. When what it printed is a scene (its
/// first call `cell(`), the page shows it.
#[must_use]
pub fn xetal(src: &str) -> String {
    let o = run(src);
    json!({ "out": o.out, "err": o.err, "millis": o.millis }).to_string()
}

/// X_eTaL source drawn decorated (HTML spans).
#[must_use]
pub fn rendered(src: &str) -> String {
    markup(src)
}

/// What a footer says (HTML): copyright, license, repository, build.
#[must_use]
pub fn about() -> String {
    footer_markup("../../")
}
