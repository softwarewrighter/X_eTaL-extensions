//! three, live: the city of a seed, worked out by X_eTaL in the page
//! and drawn by the Three.js player (static/live.js calls `build`).

use wasm_bindgen::prelude::*;

/// Builds and shows the city of a seed; what to say of it.
///
/// # Errors
///
/// The program's error.
#[wasm_bindgen]
pub fn build(seed: u32) -> Result<String, JsError> {
    three_web::build(seed).map_err(|e| JsError::new(&e))
}

/// The X_eTaL program that builds the city of a seed, drawn decorated
/// (HTML).
#[wasm_bindgen]
#[must_use]
pub fn rendered(seed: u32) -> String {
    three_web::rendered(seed)
}

fn main() {
    three_web::install();
}
