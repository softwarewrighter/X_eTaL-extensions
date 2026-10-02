//! Running for display: values as grids, in order with printed text.

use xetal_eval::{Event, Grid, eval_events};

fn events(src: &str) -> Vec<Event> {
    let mut program = xetal_core::lower(src).unwrap();
    xetal_types::check_program(&mut program).unwrap();
    let (_, events, result) = eval_events(&program, Some(1));
    result.unwrap();
    events
}

#[test]
fn values_come_as_grids_in_order_with_printed_text() {
    let got = events("p_rint! 5\n2 3 r_eshape r_ange 6\n\"hi\"");
    let grid = |kind: &str, shape: &[usize], items: &[&str]| {
        Event::Value(Grid {
            kind: kind.into(),
            shape: shape.to_vec(),
            items: items.iter().map(|s| s.to_string()).collect(),
        })
    };
    assert_eq!(
        got,
        [
            Event::Printed("5\n".into()),
            grid("Int", &[], &["5"]),
            grid("Int", &[2, 3], &["1", "2", "3", "4", "5", "6"]),
            grid("Char", &[2], &["h", "i"]),
        ]
    );
}

#[test]
fn an_error_ends_the_run_after_what_was_shown() {
    let mut program = xetal_core::lower("1 + 1\n1 / 0").unwrap();
    xetal_types::check_program(&mut program).unwrap();
    let (_, events, result) = eval_events(&program, None);
    assert_eq!(events.len(), 1);
    assert_eq!(result.unwrap_err().code, "division-by-zero");
}
