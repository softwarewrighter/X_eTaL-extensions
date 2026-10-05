//! A run can stop inside a higher-order built-in (D50): each call of
//! its operand is ordinary work of the machine, so a run taken one
//! transition at a time needs at least one slice per call.

use xetal_arith::Rng;
use xetal_step::{Machine, Status};

/// Slices of one transition needed to run `src`, and what it printed.
fn slices(src: &str) -> (usize, String) {
    let program = xetal_core::lower(src).expect("lowers");
    let mut out = Vec::new();
    let mut count = 0;
    {
        let mut machine = Machine::new(&program, &mut out, Rng::seeded(1));
        while machine.run(1).expect("runs") == Status::Running {
            count += 1;
        }
    }
    (count, String::from_utf8(out).expect("utf-8"))
}

#[test]
fn each_stops_between_items() {
    let (count, out) = slices("'+ r_/ '{ _r * 2 } e_ach r_ange 300");
    assert_eq!(out.trim(), "90300");
    assert!(count > 600, "{count} slices for 600 operand calls");
}

#[test]
fn power_stops_between_applications() {
    let (count, out) = slices("400 '{ _r + 1 } p_ower 0");
    assert_eq!(out.trim(), "400");
    assert!(count > 400, "{count} slices for 400 applications");
}
