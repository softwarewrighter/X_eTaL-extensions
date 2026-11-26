//! The component tester (decision C13, step 23): the probes (the
//! turntable's pincers) on each pair of a held part's leads; what it is
//! from the readings alone; verify against the netlist and the marking.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::world::{Outcome, World};

#[test]
#[ignore]
fn print_starts() {
    for c in ["tester", "tester-b", "demo"] {
        let mut w = World::default();
        w.apply_all(&parse_program(&format!("reset circuit={c}")).unwrap());
        for p in &w.parts {
            println!(
                "START {c} {} {:?}",
                p.id,
                p.at.map(|x| (x * 1000.0).round() / 1000.0)
            );
        }
    }
}

#[allow(dead_code)]
fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    w.apply_all(&parse_program(program).unwrap())
}

fn said(o: &Outcome) -> String {
    o.events
        .iter()
        .chain(&o.warnings)
        .cloned()
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The program's outcomes for `test X` and `verify X`, by part.
fn results(program: &str) -> std::collections::BTreeMap<String, (String, String)> {
    let mut w = World::default();
    let out = run(&mut w, program);
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    let mut r = std::collections::BTreeMap::new();
    for pair in out.windows(2) {
        if let Some(id) = pair[0].command.strip_prefix("test ") {
            r.insert(id.to_string(), (said(&pair[0]), said(&pair[1])));
        }
    }
    r
}

#[test]
fn every_kind_and_every_fault() {
    let r = results(include_str!("../../programs/tester.cell"));
    let has = |id: &str, test: &str, verify: &str| {
        let (t, v) = &r[id];
        assert!(t.contains(test), "{id} test: {t}");
        assert!(v.contains(verify), "{id} verify: {v}");
    };
    has("R1", "R1 is a resistor of 4.63 kilohm", "R1: good");
    has("R2", "R2 is a resistor of 2.26 kilohm", "R2: mislabeled");
    has("R3", "R3 is a resistor of 9.98 kilohm", "R3: wrong part");
    has(
        "C1",
        "C1 is a capacitor of 99.4 nF, ESR 0.1 ohm",
        "C1: good",
    );
    has("C2", "C2 is open", "C2: failed");
    has(
        "D1",
        "D1 is an LED, 1.91 V forward (lit red), its anode pin 1",
        "D1: good",
    );
    has(
        "D2",
        "D2 is a diode, 0.72 V forward, its anode pin 1",
        "the marking is reversed",
    );
    has("Q1", "Q1 is an NPN transistor, E 1 B 2 C 3", "Q1: good");
    let r = results(include_str!("../../programs/tester-b.cell"));
    let has = |id: &str, test: &str, verify: &str| {
        let (t, v) = &r[id];
        assert!(t.contains(test), "{id} test: {t}");
        assert!(v.contains(verify), "{id} verify: {v}");
    };
    has("Q2", "Q2 is a PNP transistor, E 1 B 2 C 3", "Q2: good");
    has("C3", "C3 is shorted", "C3: failed");
    has(
        "C4",
        "C4 is a capacitor of 10.3 uF, ESR 1.5 ohm",
        "C4: good",
    );
}

#[test]
fn the_tester_refuses_what_it_cannot_do() {
    let mut w = World::default();
    run(&mut w, "reset circuit=tester-b");
    // nothing holds it
    let o = &run(&mut w, "test Q2")[0];
    assert!(
        o.refused.as_deref().unwrap().contains("nothing holds Q2"),
        "{o:?}"
    );
    // not measured yet
    let o = &run(&mut w, "verify Q2")[0];
    assert!(
        o.refused.as_deref().unwrap().contains("test Q2 first"),
        "{o:?}"
    );
    // an IC, whoever holds it
    let o = &run(&mut w, "test U1")[0];
    assert!(
        o.refused.as_deref().unwrap().contains("an IC of 8 pins"),
        "{o:?}"
    );
    // held too high for the probes
    run(
        &mut w,
        "tool A tweezers\npick A Q2 grasp=body\nmove A to=0,0,120 rpy=180,0,90",
    );
    let o = &run(&mut w, "test Q2")[0];
    assert!(
        o.refused.as_deref().unwrap().contains("cannot reach"),
        "{o:?}"
    );
}

#[test]
fn a_component_testers_reasoning() {
    use xetal_ext_cell::tester::{Model, Seen, led_color, seen};
    // a diode conducts one way only
    let d = Model::Diode(0.7, "1".into(), "2".into(), None);
    assert_eq!(seen(&d, "1", "2"), Seen::Drop(0.7));
    assert_eq!(seen(&d, "2", "1"), Seen::Nothing);
    // a resistor beyond the tester's range is open to it
    assert_eq!(seen(&Model::Resistor(1e9), "1", "2"), Seen::Nothing);
    // an LED's color from its drop
    assert_eq!(
        (led_color(1.9), led_color(2.15), led_color(3.0)),
        ("red", "green", "blue or white")
    );
}
