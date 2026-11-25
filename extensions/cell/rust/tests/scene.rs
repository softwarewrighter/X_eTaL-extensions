//! A cell's timeline written as a scene for the players: its first call
//! is cell(...), every arm has a rig and a track, every command a step,
//! and every expect line is the simulator's flange at that time.

use xetal_ext_cell::command::parse_program;
use xetal_ext_cell::kin;
use xetal_ext_cell::rig::Rig;
use xetal_ext_cell::scene::{self, joints_at};
use xetal_ext_cell::world::World;

fn world(program: &str) -> World {
    let mut w = World::default();
    for o in w.apply_all(&parse_program(program).unwrap()) {
        assert!(o.ok, "{o:?}");
    }
    w
}

#[test]
fn a_scene_of_a_program() {
    let w = world("move A to=0,0,20 rpy=180,0,0\nstage turn=90\nvise close");
    let calls = scene::calls("demo", &w).unwrap();
    let lines: Vec<&str> = calls.lines().collect();
    assert_eq!(lines[0], "cell('demo', 1280, 720)");
    let count = |p: &str| lines.iter().filter(|l| l.starts_with(p)).count();
    assert_eq!(
        count("fixture("),
        4,
        "the stage, the scope, one rack, the tray"
    );
    assert_eq!(count("rig("), 1, "one arm");
    assert_eq!(count("track("), 2, "the arm and the stage");
    assert_eq!(count("step("), 3);
    assert!(count("expect(") >= 4);
    assert!(lines.contains(&"step(2, 'stage turn=90', 3.932754, 5.182754)"));
    assert!(lines.contains(&"rig('A', 'arm', nums('0 450 -200 0 0 -90', 6)[0])"));
    // the end of the move: the flange where it was sent
    assert!(
        lines.contains(&"expect('A', 3.932754, nums('0 0 20', 3)[0])"),
        "{calls}"
    );
}

#[test]
fn quotes_are_escaped() {
    let w = World::default();
    let calls = scene::calls("it's", &w).unwrap();
    assert!(calls.starts_with("cell('it\\'s', 1280, 720)"));
}

#[test]
fn joints_between_rows_are_interpolated() {
    let track = [
        [0.0, 0.0, 0.0, 0.0, 0.0, 90.0, 0.0],
        [2.0, 10.0, 20.0, 0.0, 0.0, 90.0, -4.0],
    ];
    assert_eq!(joints_at(&track, -1.0), [0.0, 0.0, 0.0, 0.0, 90.0, 0.0]);
    assert_eq!(joints_at(&track, 1.0), [5.0, 10.0, 0.0, 0.0, 90.0, -2.0]);
    assert_eq!(joints_at(&track, 3.0), [10.0, 20.0, 0.0, 0.0, 90.0, -4.0]);
}

#[test]
fn every_expect_is_the_simulators_flange() {
    let w = world("joints A -30 -20 10 0 60 0\nmove A to=120,40,30 rpy=180,0,0\nlift A mm=40");
    let rig = Rig::arm().unwrap();
    let calls = scene::calls("x", &w).unwrap();
    let mut n = 0;
    for line in calls.lines().filter(|l| l.starts_with("expect('A'")) {
        // expect('A', T, nums('X Y Z', 3)[0])
        let t: f64 = line.split(", ").nth(1).unwrap().parse().unwrap();
        let xyz: Vec<f64> = line
            .split('\'')
            .nth(3)
            .unwrap()
            .split(' ')
            .map(|v| v.parse().unwrap())
            .collect();
        let arm = w.arm("A").unwrap();
        let f = arm
            .base_pose()
            .then(&kin::forward(&rig, &joints_at(&arm.track, t)))
            .p;
        assert!(
            kin::norm(kin::sub(f, [xyz[0], xyz[1], xyz[2]])) < 0.001,
            "{line}"
        );
        n += 1;
    }
    assert!(n >= 3);
}
