//! The hello page: what it links and the program it shows first.

use xetal_ext_shell::run::Linked;

/// The program the page opens with: hello's tour.
pub const PROGRAM: &str = include_str!("../../demos/tour.xtl");

/// The same program on the command line.
pub const COMMAND: &str = "xetal-x --ext extensions/hello run extensions/hello/demos/tour.xtl";

/// hello, linked statically, and its facade.
pub fn linked() -> Vec<Linked> {
    vec![Linked {
        descriptor: xetal_ext_hello::__xetal_extension::descriptor,
        facade: ("Hello.xtl", include_str!("../../lib/Hello.xtl")),
    }]
}
