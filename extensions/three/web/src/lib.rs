//! The three page: what it links, the libraries its program imports,
//! and the program that builds the city of a seed.

use xetal_ext_shell::chrome::footer_markup;
use xetal_ext_shell::run::{Linked, install_with, run};
use xetal_ext_shell::source::markup;

/// The same thing on the command line (the film's seed).
pub const COMMAND: &str = "just ext three city";

/// The X_eTaL libraries the program imports besides the facade: the
/// calls' vocabulary (blender's Cycles.xtl) and the city (NeonCity.xtl).
pub const LIBRARIES: &[(&str, &str)] = &[
    (
        "Cycles.xtl",
        include_str!("../../../blender/lib/Cycles.xtl"),
    ),
    (
        "NeonCity.xtl",
        include_str!("../../../blender/demos/NeonCity.xtl"),
    ),
];

/// three, linked statically, and its facade.
pub fn linked() -> Vec<Linked> {
    vec![Linked {
        descriptor: xetal_ext_three::__xetal_extension::descriptor,
        facade: ("Three.xtl", include_str!("../../lib/Three.xtl")),
    }]
}

/// Registers three and the libraries; once, before any [`build`].
pub fn install() -> usize {
    install_with(&linked(), LIBRARIES).len()
}

/// The program that works out the city of a seed and shows it.
pub fn program(seed: u32) -> String {
    format!(
        "\"th:\" u_se< \"Three\"\n\"nc:\" u_se< \"NeonCity\"\n\nth:s_how! ({seed}, \"media\", 1280, 720, 30, 24, \"GPU\") nc:c_ity 0\n"
    )
}

/// Runs the program of a seed: what to say of it (how many calls, how
/// long X_eTaL took), or the error.
///
/// # Errors
///
/// The program's error, as text.
pub fn build(seed: u32) -> Result<String, String> {
    let o = run(&program(seed));
    if o.err.is_empty() {
        Ok(format!(
            "seed {seed}: {} calls from X_eTaL in {:.0} ms",
            o.out.trim(),
            o.millis
        ))
    } else {
        Err(o.err)
    }
}

/// The program of a seed drawn decorated, as X_eTaL renders it: HTML
/// spans classed by the kind of token.
pub fn rendered(seed: u32) -> String {
    markup(&program(seed))
}

/// What a footer says, as HTML spans: copyright, license, repository,
/// the catalog, the X_eTaL commit and the build (host, commit, time).
pub fn about() -> String {
    footer_markup("../../")
}

/// A library the program imports, drawn decorated (HTML): "NeonCity"
/// is the city, "Three" the facade; nothing for another name.
pub fn library(name: &str) -> String {
    match name {
        "NeonCity" => markup(LIBRARIES[1].1),
        "Three" => markup(include_str!("../../lib/Three.xtl")),
        _ => String::new(),
    }
}
