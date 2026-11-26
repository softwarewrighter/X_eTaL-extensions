//! The assembly cell, live: the session (lib.rs) handed to the page
//! (page/live.js calls these; trunk starts the module and says so).

use wasm_bindgen::prelude::*;

/// Registers the cell with X_eTaL; once, first.
#[wasm_bindgen]
#[must_use]
pub fn install() -> usize {
    cell_web::install()
}

/// A new cell for a circuit: the outcome (JSON).
#[wasm_bindgen]
#[must_use]
pub fn reset(circuit: &str) -> String {
    cell_web::reset(circuit)
}

/// An empty cell, nothing run on it yet.
#[wasm_bindgen]
pub fn clear() {
    cell_web::clear();
}

/// Commands run on the cell: their outcomes and times (JSON).
#[wasm_bindgen]
#[must_use]
pub fn command(text: &str) -> String {
    cell_web::command(text)
}

/// Bands read from the microscope's picture (JSON in, JSON out).
#[wasm_bindgen]
#[must_use]
pub fn read_bands(json: &str) -> String {
    cell_web::read_bands(json)
}

/// What a marking read from the picture means for a kind (JSON).
#[wasm_bindgen]
#[must_use]
pub fn decode_mark(kind: &str, text: &str) -> String {
    cell_web::decode_mark(kind, text)
}

/// The cell's timeline as a scene for the player.
#[wasm_bindgen]
#[must_use]
pub fn scene(name: &str) -> String {
    cell_web::scene(name)
}

/// The cell now, for the controls (JSON).
#[wasm_bindgen]
#[must_use]
pub fn state() -> String {
    cell_web::state()
}

/// A program the page offers, by name.
#[wasm_bindgen]
#[must_use]
pub fn program(name: &str) -> String {
    cell_web::program(name)
}

/// The X_eTaL program the page starts with.
#[wasm_bindgen]
#[must_use]
pub fn example() -> String {
    cell_web::EXAMPLE.to_string()
}

/// An X_eTaL program run in the page (JSON: out, err, millis).
#[wasm_bindgen]
#[must_use]
pub fn xetal(src: &str) -> String {
    cell_web::xetal(src)
}

/// X_eTaL source drawn decorated (HTML).
#[wasm_bindgen]
#[must_use]
pub fn rendered(src: &str) -> String {
    cell_web::rendered(src)
}

/// What a footer says (HTML).
#[wasm_bindgen]
#[must_use]
pub fn about() -> String {
    cell_web::about()
}

fn main() {}
