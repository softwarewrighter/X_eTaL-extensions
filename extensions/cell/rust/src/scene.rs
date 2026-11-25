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
//! attach('U1', 'A', t0, t1)          (an arm's hold; 'stage': the vise's)
//! place('U1', t, nums('x y z r p y', 6)[0])   (where a fall ended)
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
use crate::layout::fixtures;
use crate::rig::Rig;
use crate::world::{Part, World};
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
    for (id, model, pose) in fixtures() {
        let _ = writeln!(
            out,
            "fixture({}, {}, {})",
            quote(&id),
            quote(&model),
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
    for p in w.tools.iter().chain(&w.parts) {
        thing(&mut out, p);
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
    // and where every part and tool ends
    let end = w
        .timeline
        .iter()
        .filter(|e| !e.rewound)
        .map(|e| e.t1)
        .fold(w.t, f64::max);
    // after the last fall has landed too
    let end = w
        .parts
        .iter()
        .flat_map(|p| p.moves.iter().map(|m| m.t))
        .fold(end, f64::max);
    for p in w.parts.iter().chain(&w.tools) {
        let at = [p.at[0], p.at[1], p.at[2]].map(|x| kin::clean((x * 1000.0).round() / 1000.0));
        let _ = writeln!(out, "expect({}, {}, {})", quote(&p.id), num(end), row(&at));
    }
    Ok(out)
}

/// A part or a tool: where it starts, each hold (an arm's, or the
/// vise's, which turns with the stage) and each fall.
fn thing(out: &mut String, p: &Part) {
    let _ = writeln!(
        out,
        "part({}, {}, {})",
        quote(&p.id),
        quote(&p.kind),
        row(&p.start)
    );
    for h in &p.holds {
        let by = if h.by == "vise" {
            "stage"
        } else {
            h.by.as_str()
        };
        let t1 = h.t1.map_or_else(|| "Infinity".to_string(), num);
        let _ = writeln!(
            out,
            "attach({}, {}, {}, {t1})",
            quote(&p.id),
            quote(by),
            num(h.t0)
        );
    }
    for m in &p.moves {
        let _ = writeln!(out, "place({}, {}, {})", quote(&p.id), num(m.t), row(&m.at));
    }
}
