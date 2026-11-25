//! A cell's timeline written as a scene for the players: one call a
//! line, numbers as tables (the players' text, as the city's and the
//! board's). The three extension's cell player (static/cell.js) draws
//! it; its first call, cell(...), is how the page knows which player.
//!
//! ```text
//! cell('NAME', 1280, 720)
//! fixture('stage', 'stage', nums('0 0 -60 0 0 0', 6)[0])
//! rig('A', 'arm', nums('0 450 -200 0 0 -90', 6)[0])
//! track('A', nums('t j1 j2 j3 j4 j5 j6 ...', 7))
//! track('stage', nums('t angle ...', 2))
//! part('U1', 'dip8', nums('x y z r p y', 6)[0])
//! attach('U1', 'A', t0, t1)
//! ghost('U1', 'dip8', nums('x y z r p y', 6)[0])
//! step(1, 'move A to=0,0,20 rpy=180,0,0', t0, t1)
//! expect('A', t, nums('x y z', 3)[0])
//! ```
//!
//! Units are the simulator's (mm, degrees, seconds; Z up): the player
//! turns them to Three.js's Y up. `expect` gives where an arm's flange
//! is at a time, for a check of the drawing against the simulator.

use crate::command::num;
use crate::kin;
use crate::rig::Rig;
use crate::world::{FIXTURES, World};
use std::fmt::Write as _;

/// How many expect lines at most, over all arms.
pub const EXPECTS: usize = 48;

fn quote(t: &str) -> String {
    format!("'{}'", t.replace('\\', "\\\\").replace('\'', "\\'"))
}

fn table(rows: &[Vec<f64>], k: usize) -> String {
    let text: Vec<String> = rows
        .iter()
        .flat_map(|r| r.iter().map(|x| num(*x)))
        .collect();
    format!("nums('{}', {k})", text.join(" "))
}

fn row(v: &[f64]) -> String {
    format!("{}[0]", table(&[v.to_vec()], v.len()))
}

/// An arm's joints at time `t` as the player finds them: the track's
/// rows interpolated linearly.
#[must_use]
pub fn joints_at(track: &[[f64; 7]], t: f64) -> [f64; 6] {
    let i = track.partition_point(|r| r[0] <= t);
    if i == 0 {
        return std::array::from_fn(|j| track[0][j + 1]);
    }
    if i >= track.len() {
        return std::array::from_fn(|j| track[track.len() - 1][j + 1]);
    }
    let (a, b) = (track[i - 1], track[i]);
    let k = (t - a[0]) / (b[0] - a[0]);
    std::array::from_fn(|j| a[j + 1] + (b[j + 1] - a[j + 1]) * k)
}

/// The world's timeline as a scene called `name`.
///
/// # Errors
///
/// The rig compiled in is wrong.
pub fn calls(name: &str, w: &World) -> Result<String, String> {
    let rig = Rig::arm()?;
    let mut out = format!("cell({}, 1280, 720)\n", quote(name));
    for (id, model, pose) in FIXTURES {
        let _ = writeln!(
            out,
            "fixture({}, {}, {})",
            quote(id),
            quote(model),
            row(&pose)
        );
    }
    for a in &w.arms {
        let _ = writeln!(out, "rig({}, 'arm', {})", quote(&a.name), row(&a.base));
        let rows: Vec<Vec<f64>> = a.track.iter().map(|r| r.to_vec()).collect();
        let _ = writeln!(out, "track({}, {})", quote(&a.name), table(&rows, 7));
    }
    let rows: Vec<Vec<f64>> = w.stage_track.iter().map(|r| r.to_vec()).collect();
    let _ = writeln!(out, "track('stage', {})", table(&rows, 2));
    for p in &w.parts {
        let pose = [p.at[0], p.at[1], p.at[2], p.rpy[0], p.rpy[1], p.rpy[2]];
        let _ = writeln!(
            out,
            "part({}, {}, {})",
            quote(&p.id),
            quote(&p.kind),
            row(&pose)
        );
    }
    for (n, e) in w.timeline.iter().enumerate() {
        let _ = writeln!(
            out,
            "step({}, {}, {}, {})",
            n + 1,
            quote(&e.command),
            num(e.t0),
            num(e.t1)
        );
    }
    // expectations: each arm's flange at the middle and the end of each
    // step, while there are fewer than EXPECTS
    let mut times: Vec<f64> = w
        .timeline
        .iter()
        .flat_map(|e| [(e.t0 + e.t1) / 2.0, e.t1])
        .collect();
    times.dedup();
    let per = (EXPECTS / w.arms.len().max(1)).max(1);
    let step = times.len().div_ceil(per).max(1);
    for a in &w.arms {
        let base = a.base_pose();
        for &t in times.iter().step_by(step) {
            let f = base.then(&kin::forward(&rig, &joints_at(&a.track, t))).p;
            let r = f.map(|x| kin::clean((x * 1000.0).round() / 1000.0));
            let _ = writeln!(out, "expect({}, {}, {})", quote(&a.name), num(t), row(&r));
        }
    }
    Ok(out)
}
