//! Lap joints (decision C22): a lead joined to another lead lies
//! alongside it, 3.2 mm (1/8 inch) or more, touching; a butt joint or a
//! shorter lap holds by the solder alone and is weak; a longer lap is
//! more metal to heat.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::world::{Outcome, World};

const LAP: &str = include_str!("../../programs/lap.cell");
const SOLDER: &str = include_str!("../../programs/solder.cell");

fn run(w: &mut World, program: &str) -> Vec<Outcome> {
    w.apply_all(&parse_program(program).unwrap())
}

fn all_ok(out: &[Outcome]) {
    assert!(out.iter().all(|o| o.ok), "{:?}", out.iter().find(|o| !o.ok));
}

fn joint<'a>(w: &'a World, name: &str) -> &'a xetal_ext_cell::solder::Joint {
    w.joints.iter().find(|j| j.name == name).unwrap()
}

#[test]
fn a_lap_joint_is_good() {
    let mut w = World::default();
    let out = run(&mut w, LAP);
    all_ok(&out);
    let j = joint(&w, "U1.3+R1.1");
    assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
    assert!(j.lap >= 3.2 && j.lap < 4.5, "{}", j.lap);
    // along the lap: more than one node of each lead
    assert!(j.nodes_a.len() >= 3 && j.nodes_b.len() >= 3, "{j:?}");
    let o = out
        .iter()
        .find(|o| o.command.starts_with("inspect"))
        .unwrap();
    assert!(
        o.events[0].contains("good; a 3.") && o.events[0].contains(" mm lap;"),
        "{o:?}"
    );
    let o = out.iter().find(|o| o.command.starts_with("bring")).unwrap();
    assert_eq!(o.events, ["R1.1's end alongside U1.3: a 3.2 mm lap"]);
}

#[test]
fn a_butt_joint_is_weak() {
    let mut w = World::default();
    let out = run(&mut w, SOLDER);
    all_ok(&out);
    let j = joint(&w, "U1.3+R1.1");
    assert_eq!(j.result.as_deref(), Some("weak"), "{j:?}");
    assert!(j.lap < 0.05 && j.holds(), "{j:?}");
    let o = run(&mut w, "inspect U1.3+R1.1");
    assert!(
        o[0].events[0].contains("weak; a butt joint (a lap of 3.2 mm is sound)"),
        "{o:?}"
    );
}

#[test]
fn a_short_lap_is_weak() {
    let mut w = World::default();
    all_ok(&run(&mut w, &LAP.replace("lap=3.2", "lap=1.5")));
    let j = joint(&w, "U1.3+R1.1");
    assert_eq!(j.result.as_deref(), Some("weak"), "{j:?}");
    assert!(j.lap > 1.0 && j.lap < 3.2, "{}", j.lap);
}

#[test]
fn a_straight_lead_is_not_laid_alongside() {
    let mut w = World::default();
    let program = LAP.split("insert A").next().unwrap().to_string()
        + "bring A R1.1 to=U1.3.tip along=-0.934,0.356,0 lap=3.2";
    let out = run(&mut w, &program);
    let o = out.iter().find(|o| o.command.starts_with("bring")).unwrap();
    assert!(
        o.refused
            .as_deref()
            .unwrap_or("")
            .contains("R1.1's lead is straight: bend its end first"),
        "{o:?}"
    );
}

#[test]
fn a_longer_lap_takes_more_heat() {
    // a short heat: the longer lap's solder touches more of each lead
    let wet = |lap: &str| {
        let mut w = World::default();
        let program = LAP
            .replace("lap=3.2", &format!("lap={lap}"))
            .replace("dwell=3 temp=400", "dwell=0.3 temp=330")
            .replace("feed S1 mm=2 rate=1", "feed S1 mm=2 rate=4");
        all_ok(&run(&mut w, &program));
        joint(&w, "U1.3+R1.1").wet_a
    };
    let (short, long) = (wet("1.5"), wet("3.2"));
    assert!(
        long < short,
        "wet {long} s with the longer lap, {short} s with the shorter"
    );
}

#[test]
fn a_jumper_is_formed_with_lap_ends() {
    // pin 4 across the chip's end to pin 5, each end down its leg
    let mut w = World::default();
    let prefix = LAP.split("pick A R1").next().unwrap();
    let out = run(
        &mut w,
        &(prefix.to_string()
            + "lift A mm=60
wire S5 awg=30 span=U1.4.tip+U1.5.tip lap=3.2 enamel=yes
pick A W1 grasp=middle
cut S5
lift A mm=60
lay A W1
pinch P1 on=W1.1 at=1.5
release A W1
lift A mm=60
present U1.4+W1.1 at=S1
touch S1 at=U1.4+W1.1 dwell=3 temp=400
feed S1 mm=2 rate=1
back S1
wait until=U1.4+W1.1 below=100
present U1.5+W1.2 at=S1
touch S1 at=U1.5+W1.2 dwell=3 temp=400
feed S1 mm=2 rate=1
back S1
wait until=U1.5+W1.2 below=100
unpinch P1
inspect shorts"),
    );
    all_ok(&out);
    let o = out.iter().find(|o| o.command.starts_with("lay")).unwrap();
    assert_eq!(
        o.events,
        [
            "W1.1 alongside U1.4: a 3.2 mm lap",
            "W1.2 alongside U1.5: a 3.2 mm lap"
        ]
    );
    for name in ["U1.4+W1.1", "U1.5+W1.2"] {
        let j = joint(&w, name);
        assert_eq!(j.result.as_deref(), Some("good"), "{j:?}");
        assert!(j.lap >= 3.2, "{j:?}");
    }
    assert_eq!(out.last().unwrap().events, ["shorts: none"]);
}

#[test]
fn inspect_says_a_lap_is_long() {
    // more overlap than a joint needs is more metal to heat (a DIP's leg
    // is too short for one: the lap is set to show what inspect says)
    let mut w = World::default();
    all_ok(&run(&mut w, LAP));
    let k = w.joints.iter().position(|j| j.name == "U1.3+R1.1").unwrap();
    w.joints[k].lap = 7.0;
    let o = run(&mut w, "inspect U1.3+R1.1");
    assert!(
        o[0].events[0].contains("a 7 mm lap, long: more metal to heat than it needs (over 6.4)"),
        "{o:?}"
    );
}

#[test]
fn the_blinker_matches_its_netlist() {
    // the whole 555 blinker, every joint a lap
    let mut w = World::default();
    let out = run(&mut w, include_str!("../../programs/blinker.cell"));
    all_ok(&out);
    let r = w.check_work();
    assert!(r.ok && r.made == (7, 7), "{r:?}");
    assert!(
        w.joints
            .iter()
            .all(|j| j.result.as_deref() == Some("good") && j.lap >= 3.2),
        "{:?}",
        w.joints
            .iter()
            .filter(|j| j.result.as_deref() != Some("good") || j.lap < 3.2)
            .collect::<Vec<_>>()
    );
    // a radial's straight lead laid alongside a leg, no bend: C1 on pin 2
    let o = out
        .iter()
        .find(|o| o.command.starts_with("bring A C1.1"))
        .unwrap();
    assert_eq!(o.events, ["C1.1's end alongside U1.2: a 3.2 mm lap"]);
    // a leg carrying three laps, heated for the third: one fillet, the
    // fresh solder's flux over all of it, none of them dull
    let on8 = w
        .joints
        .iter()
        .filter(|j| j.a == "U1.8" || j.b == "U1.8")
        .count();
    assert_eq!(on8, 3);
}
