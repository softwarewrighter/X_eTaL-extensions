//! What the page shows of a run as its events arrive: output appended
//! line by line, pictures and errors as they come, running until done
//! or stopped.

use std::rc::Rc;

use xetal_runner::{Action, Event, Output};
use yew::Reducible;

fn after(actions: Vec<Action>) -> Output {
    let mut state = Rc::new(Output::default());
    for a in actions {
        state = state.reduce(a);
    }
    (*state).clone()
}

#[test]
fn events_are_appended_as_they_arrive() {
    let o = after(vec![
        Action::Start,
        Action::Event(Event::Out("1750 559".into())),
        Action::Event(Event::Out("1500 600".into())),
        Action::Event(Event::Picture("<svg/>".into())),
    ]);
    assert!(o.running);
    let run = o.run.unwrap();
    assert_eq!(run.out, "1750 559\n1500 600\n");
    assert_eq!(run.pictures, ["<svg/>"]);
}

/// The worker saying it is ready (to be sent the program) changes
/// nothing shown.
#[test]
fn ready_changes_nothing() {
    let o = after(vec![Action::Start, Action::Event(Event::Ready)]);
    assert!(o.running);
    assert_eq!(o.run.unwrap(), xetal_play::Run::default());
}

#[test]
fn done_ends_the_run_and_errors_are_kept() {
    let o = after(vec![
        Action::Start,
        Action::Event(Event::Err("error[domain]: x".into())),
        Action::Event(Event::Done),
    ]);
    assert!(!o.running);
    assert_eq!(o.run.unwrap().err, "error[domain]: x\n");
}

#[test]
fn stopping_keeps_what_was_shown_and_says_so() {
    let o = after(vec![
        Action::Start,
        Action::Event(Event::Out("a".into())),
        Action::Stop,
    ]);
    assert!(!o.running);
    let run = o.run.unwrap();
    assert_eq!(run.out, "a\n");
    assert_eq!(run.err, "stopped\n");
}

#[test]
fn a_new_run_starts_empty_and_clear_shows_the_types_again() {
    let o = after(vec![
        Action::Start,
        Action::Event(Event::Out("a".into())),
        Action::Event(Event::Done),
        Action::Start,
    ]);
    assert_eq!(o.run.unwrap().out, "");
    assert_eq!(after(vec![Action::Start, Action::Clear]), Output::default());
}

/// In a notebook, each statement's source starts a cell, and what it
/// prints and draws goes under it.
#[test]
fn a_notebook_groups_output_under_its_statement() {
    let o = after(vec![
        Action::Start,
        Action::Event(Event::Source("x := 1".into())),
        Action::Event(Event::Source("x + 1".into())),
        Action::Event(Event::Out("2".into())),
        Action::Event(Event::Picture("<svg/>".into())),
        Action::Event(Event::Source("x + 2".into())),
        Action::Event(Event::Out("3".into())),
    ]);
    let run = o.run.clone().unwrap();
    let cells: Vec<(String, String, usize)> = o
        .cells
        .iter()
        .map(|c| {
            (
                c.source.clone(),
                o.out_of(c).to_string(),
                o.pictures_of(c).len(),
            )
        })
        .collect();
    assert_eq!(
        cells,
        [
            ("x := 1".into(), String::new(), 0),
            ("x + 1".into(), "2\n".into(), 1),
            ("x + 2".into(), "3\n".into(), 0),
        ]
    );
    assert_eq!(run.out, "2\n3\n");
}
