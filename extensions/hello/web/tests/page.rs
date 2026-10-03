//! The page's program, run as the page runs it (natively here), gives
//! the command line's golden.

use hello_web::{PROGRAM, linked};
use xetal_ext_shell::run::{install, run};

#[test]
fn the_tour_runs_as_on_the_command_line() {
    let functions = install(&linked());
    assert_eq!(functions.len(), 8);
    let o = run(PROGRAM);
    assert_eq!(o.err, "");
    assert_eq!(o.out, include_str!("../../tests/hello-demo-tour.out"));
}
