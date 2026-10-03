//! The clock page: what it links and the program it shows first.

use xetal_ext_shell::run::Linked;

/// The program the page opens with: the time, and the bridge's cost.
pub const PROGRAM: &str = include_str!("../../demos/bridge-cost.xtl");

/// The same program on the command line.
pub const COMMAND: &str = "xetal-x --ext extensions run extensions/clock/demos/bridge-cost.xtl";

/// clock and hello (the cost demo calls hello), linked statically,
/// and their facades.
pub fn linked() -> Vec<Linked> {
    vec![
        Linked {
            descriptor: xetal_ext_clock::__xetal_extension::descriptor,
            facade: ("Clock.xtl", include_str!("../../lib/Clock.xtl")),
        },
        Linked {
            descriptor: xetal_ext_hello::__xetal_extension::descriptor,
            facade: ("Hello.xtl", include_str!("../../../hello/lib/Hello.xtl")),
        },
    ]
}
