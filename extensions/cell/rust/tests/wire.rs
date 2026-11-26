//! Wire (decision C15, step 18): gauges from data and the AWG formula; a
//! feeder station feeds a length out of its nozzle, still on the spool
//! until it cuts; an arm takes the free end; the wire is a part whose
//! halves are its pins, bent and soldered like any lead; enamel burns
//! off at the iron; a free end sags as a cantilever.

use xetal_ext_cell::command::{parse_line, parse_program};
use xetal_ext_cell::wire;
use xetal_ext_cell::world::{Outcome, World};

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    let out = w.apply_all(&parse_program(program).unwrap());
    for o in &out {
        assert!(o.ok, "{o:?}");
    }
    out
}

#[test]
fn gauges_from_the_table_and_the_formula() {
    let d = wire::data().unwrap();
    let g26 = d.gauge(26);
    assert!((g26.diameter - 0.405).abs() < 1e-9 && (g26.amps - 2.2).abs() < 1e-9);
    // worked out: AWG 28 between 26 and 30, its resistance from copper's
    let g28 = d.gauge(28);
    assert!((g28.diameter - 0.321).abs() < 0.001, "{}", g28.diameter);
    assert!((g28.ohm_per_m - 0.2128).abs() < 0.002, "{}", g28.ohm_per_m);
    assert!(g28.amps < g26.amps && g28.amps > d.gauge(30).amps);
    // the formula agrees with the table where both speak
    let f = |n: f64| 0.127 * 92f64.powf((36.0 - n) / 39.0);
    assert!((f(22.0) - 0.644).abs() < 0.001);
}

#[test]
fn a_thin_long_free_end_sags() {
    let d = wire::data().unwrap();
    // AWG 34, 40 mm held at one end: about 0.15 mm; AWG 22: far less
    let thin = d.sag(0.160, 40.0);
    assert!(thin > 0.1 && thin < 0.25, "{thin}");
    assert!(d.sag(0.644, 40.0) < thin / 10.0);
    // the fourth power of the length
    assert!((d.sag(0.4, 20.0) * 16.0 - d.sag(0.4, 40.0)).abs() < 1e-9);
}

/// The DIP in the vise, the arm clear, 40 mm of wire fed.
const FED: &str = "reset circuit=demo
tool A tweezers
pick A U1 grasp=ends
place A U1 at=0,0,-24 rpy=0,0,90
vise close
release A U1
lift A mm=60
wire S5 awg=26 mm=40";

#[test]
fn the_feeder_feeds_and_cuts() {
    let mut w = World::default();
    let out = run(&mut w, FED);
    assert!(
        out.last().unwrap().events[0].starts_with("S5 feeds W1: 40 mm of AWG 26"),
        "{:?}",
        out.last()
    );
    let w1 = w.part("W1").unwrap();
    assert_eq!(w1.holder.as_deref(), Some("S5"), "on the spool");
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    // a second feed before the cut: refused
    let o = w.apply(&parse_line("wire S5 awg=26 mm=10").unwrap());
    assert!(
        o.refused
            .as_deref()
            .unwrap()
            .contains("W1 is still on S5's spool"),
        "{o:?}"
    );
    // the arm takes it, but cannot carry it off before the cut
    let o = run(&mut w, "pick A W1 grasp=middle");
    assert!(
        o[0].events
            .iter()
            .any(|e| e.contains("still on S5's spool")),
        "{o:?}"
    );
    let o = w.apply(&parse_line("lift A mm=20").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("cut S5 first"),
        "{o:?}"
    );
    // cut: the arm has it, and carries it away
    run(&mut w, "cut S5\nlift A mm=60");
    assert_eq!(w.part("W1").unwrap().holder.as_deref(), Some("A"));
    // nothing more to cut
    let o = w.apply(&parse_line("cut S5").unwrap());
    assert!(
        o.refused.as_deref().unwrap().contains("no wire out"),
        "{o:?}"
    );
}

#[test]
fn a_wire_cut_with_nothing_holding_it_falls() {
    let mut w = World::default();
    run(&mut w, &format!("{FED}\ncut S5"));
    assert!(w.part("W1").unwrap().lost);
}

#[test]
fn the_wire_program_solders_and_bends_a_wire() {
    let mut w = World::default();
    let out = run(&mut w, include_str!("../../programs/wire.cell"));
    let j = w.joints.iter().find(|j| j.name == "U1.3+W1.1").unwrap();
    assert_eq!(j.result.as_deref(), Some("good"));
    assert!(out.iter().any(|o| {
        o.events
            .iter()
            .any(|e| e.starts_with("P2 bends W1.2 90 degrees"))
    }));
    assert_eq!(w.part("W1").unwrap().holder.as_deref(), Some("vise"));
    assert!(w.clashes().is_empty(), "{:?}", w.clashes());
    assert_eq!(World::from_json(&w.to_json()).unwrap(), w);
}

#[test]
fn enamel_burns_off_at_the_iron_before_the_wire_wets() {
    let mut w = World::default();
    let prog = include_str!("../../programs/wire.cell")
        .replace("wire S5 awg=26 mm=40", "wire S5 awg=26 mm=40 enamel=yes")
        // magnet wire wants a hotter iron: its enamel burns off at 330 C
        .replace(
            "touch S1 at=U1.3+W1.1 dwell=2",
            "touch S1 at=U1.3+W1.1 dwell=3 temp=400",
        );
    let out = run(&mut w, &prog);
    assert!(
        out.iter().any(|o| o
            .events
            .iter()
            .chain(&o.warnings)
            .any(|e| e.contains("W1.1's enamel burned off"))),
        "no stripping"
    );
    let j = w.joints.iter().find(|j| j.name == "U1.3+W1.1").unwrap();
    assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
    assert!(w.part("W1").unwrap().wire.as_ref().unwrap().stripped[0]);
}
