//! A run waits for typed lines (D50): with an input queue, `[]R_EAD`
//! takes the next line, or, when none has been typed, the run stops as
//! Waiting and goes on from that very call when a line is fed, its
//! state (loops, recursion, bindings) intact.

use xetal_arith::Rng;
use xetal_step::{Machine, Status};

/// A tiny text adventure: a loop (by recursion) reading commands. A
/// top-level p_rint! prints its text, then its value, as at the CLI.
const ADVENTURE: &str = r#"
u:t_urn := { n ->
  cmd := []R_EAD @
  cmd m_atch "q" ? n
  p_rint! "you " c_at cmd
  u:t_urn n + 1
}
p_rint! "ready"
u:t_urn 0
"#;

#[test]
fn a_run_waits_for_each_line_and_resumes_where_it_was() {
    let program = xetal_core::lower(ADVENTURE).expect("lowers");
    let mut out = Vec::new();
    let mut seen = Vec::new();
    {
        let mut machine = Machine::new(&program, &mut out, Rng::seeded(1)).waiting_for_input();
        for line in ["look", "go north", "q"] {
            assert_eq!(machine.run(1_000_000).expect("runs"), Status::Waiting);
            machine.feed(line.to_string());
        }
        assert_eq!(machine.run(1_000_000).expect("runs"), Status::Done);
        seen.push(());
    }
    let printed = String::from_utf8(out).expect("utf-8");
    assert_eq!(printed, "ready\nready\nyou look\nyou go north\n2\n");
    assert_eq!(seen.len(), 1);
}

#[test]
fn waiting_stops_before_the_line_is_needed() {
    let program =
        xetal_core::lower("p_rint! \"name?\"; x := []R_EAD @; p_rint! x").expect("lowers");
    let mut out = Vec::new();
    let mut machine = Machine::new(&program, &mut out, Rng::seeded(1)).waiting_for_input();
    assert_eq!(machine.run(1_000_000).expect("runs"), Status::Waiting);
    assert_eq!(machine.run(1_000_000).expect("runs"), Status::Waiting);
    machine.feed("Ada".to_string());
    assert_eq!(machine.run(1_000_000).expect("runs"), Status::Done);
    drop(machine);
    assert_eq!(
        String::from_utf8(out).expect("utf-8"),
        "name?\nname?\nAda\nAda\n"
    );
}

#[test]
fn a_run_waits_for_one_key_and_gets_a_key() {
    let src = "k := []K_EY @; p_rint! k; k = []K_NAMED 1";
    let program = xetal_core::lower(src).expect("lowers");
    let mut out = Vec::new();
    let mut machine = Machine::new(&program, &mut out, Rng::seeded(1)).waiting_for_input();
    assert_eq!(machine.run(1_000_000).expect("runs"), Status::WaitingKey);
    machine.feed("Up".to_string());
    assert_eq!(machine.run(1_000_000).expect("runs"), Status::Done);
    drop(machine);
    assert_eq!(String::from_utf8(out).expect("utf-8"), "UP\nUP\n1\n");
}
