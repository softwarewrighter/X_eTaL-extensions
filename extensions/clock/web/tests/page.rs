//! The page's program, run as the page runs it (natively here), with
//! two extensions linked statically: the command line's golden, its
//! timings masked as the golden masks them.

use clock_web::{PROGRAM, linked};
use xetal_ext_shell::run::{install, run};

fn masked(text: &str) -> String {
    let mut out = String::new();
    let mut in_number = false;
    for c in text.chars() {
        let digit = c.is_ascii_digit() || (in_number && (c == '.' || c == 'e' || c == '-'));
        if digit {
            if !in_number {
                out.push('N');
            }
        } else {
            out.push(c);
        }
        in_number = digit;
    }
    out
}

#[test]
fn the_cost_demo_runs_as_on_the_command_line() {
    let functions = install(&linked());
    assert_eq!(functions.len(), 4 + 8);
    let o = run(PROGRAM);
    assert_eq!(o.err, "");
    assert_eq!(
        masked(&o.out),
        include_str!("../../tests/clock-demo-bridge-cost.out")
    );
}
