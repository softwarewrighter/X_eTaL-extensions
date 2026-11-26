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
//! track('stage', nums('t turn tilt x y z ...', 6))
//! pincer('P1', 'clip', nums('t housing x y z r p y, mount x y z r p y ...', 13))
//! lead('R1', '2', t, diameter, nums('x y z ...', 3))   a lead re-formed
//! scope(nums('t pan tilt view focus rotate dist ...', 7))   the microscope
//! snap(t, 'U1.3+R1.1', field)           a picture taken
//! feeder('S1', length)                   the iron's solder feeder
//! bender_shape(plate, size, shear, height)  the bending rig's shape
//! mark('R1', {marking, body, pin1})     a part's marking, for drawing
//! sleeve('R1', '1', on, shrunk, nums(...), loose, snug, color)  heat-shrink on a lead
//! bender(nums('t close former shear r ...', 5))  the bending rig
//! feeding(t0, t1, 'S1')                  solder fed into a joint
//! station('S1', 'iron', nums('housing x y z r p y', 6)[0], nums('t x y z r p y ...', 7))
//! part('U1', 'dip8', nums('x y z r p y', 6)[0])
//! attach('U1', 'A', t0, t1)          (an arm's hold; 'stage': the vise's)
//! place('U1', t, nums('x y z r p y', 6)[0])   (where a fall ended)
//! heat('U1', nums('t body pin1 pin2 ...', k))  (C, while warm)
//! joint('U1.3+R1.1', 'U1', nums('x y z', 3)[0], nums('t temp mm3 ...', 3), 'good', t_froze)
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
    let _ = writeln!(out, "track('stage', {})", table(&rows, 6));
    // the microscope: its pan, tilt, view and the pictures it took
    let rows: Vec<Vec<f64>> = w.scope.track.iter().map(|r| r.to_vec()).collect();
    let _ = writeln!(out, "scope({})", table(&rows, 7));
    for (t0, t1, st) in &w.feeding {
        let _ = writeln!(out, "feeding({}, {}, {})", num(*t0), num(*t1), quote(st));
    }
    // the bending rig: its shape, then its clamp, former and shear
    {
        use crate::bender::{PLATE, POSE, SHEAR};
        let _ = writeln!(
            out,
            "bender_shape({}, {}, {}, {})",
            row(&PLATE.0),
            row(&PLATE.1),
            row(&SHEAR),
            num(POSE[2] + 200.0)
        );
    }
    let rows: Vec<Vec<f64>> = w.bender.track.iter().map(|r| r.to_vec()).collect();
    if !rows.is_empty() {
        let _ = writeln!(out, "bender({})", table(&rows, 5));
    }
    for (t, what, field) in &w.scope.snaps {
        let _ = writeln!(out, "snap({}, {}, {})", num(*t), quote(what), num(*field));
    }
    // the pincers: their housing and jaws in the cell's frame, at their
    // own moves and the turntable's
    for p in &w.pincers {
        let mut rows: Vec<Vec<f64>> = pincer_times(w, p)
            .into_iter()
            .map(|t| {
                let (h, m) = pincer_at(w, p, t);
                std::iter::once(t)
                    .chain(
                        crate::layout::six(&h)
                            .into_iter()
                            .chain(crate::layout::six(&m))
                            .map(|x| kin::clean((x * 1000.0).round() / 1000.0)),
                    )
                    .collect()
            })
            .collect();
        unwrap_angles(&mut rows, &[4, 5, 6, 10, 11, 12]);
        let _ = writeln!(
            out,
            "pincer({}, {}, {})",
            quote(&p.name),
            quote(crate::pincers::TOOL),
            table(&rows, 13)
        );
    }
    // the tool stations: the housing, and the tool's mount over time
    let all = crate::stations::data().unwrap_or_default();
    for st in &w.stations {
        let Some(s) = all.iter().find(|s| s.name == st.name) else {
            continue;
        };
        let rows: Vec<Vec<f64>> = st
            .track
            .iter()
            .map(|r| {
                let m = crate::layout::six(&s.mount(r[1]));
                std::iter::once(r[0])
                    .chain(m.iter().map(|x| kin::clean((x * 1000.0).round() / 1000.0)))
                    .collect()
            })
            .collect();
        let housing =
            crate::layout::six(&s.housing()).map(|x| kin::clean((x * 1000.0).round() / 1000.0));
        let _ = writeln!(
            out,
            "station({}, {}, {}, {})",
            quote(&s.name),
            quote(&s.tool),
            row(&housing),
            table(&rows, 7)
        );
        if s.feeder {
            // its solder feeder: the tool's length, its tip from the mount
            let _ = writeln!(
                out,
                "feeder({}, {})",
                quote(&s.name),
                num(crate::layout::tool_length(&s.tool))
            );
        }
    }
    for p in w.tools.iter().chain(&w.parts) {
        thing(&mut out, p);
    }
    // how hot each warm part was, and each joint's solder
    for h in &w.heat {
        let k = h.rows.first().map_or(0, Vec::len);
        let _ = writeln!(out, "heat({}, {})", quote(&h.id), table(&h.rows, k));
    }
    for j in &w.joints {
        if j.track.is_empty() {
            continue;
        }
        let rows: Vec<Vec<f64>> = j.track.iter().map(|r| r.to_vec()).collect();
        let part = j.a.split('.').next().unwrap_or("");
        let at = j.at.map(|x| kin::clean((x * 1000.0).round() / 1000.0));
        let result = j.result.as_deref().unwrap_or("");
        let set = j.set_at.map_or_else(|| "Infinity".to_string(), num);
        let _ = writeln!(
            out,
            "joint({}, {}, {}, {}, {}, {set})",
            quote(&j.name),
            quote(part),
            row(&at),
            table(&rows, 3),
            quote(result)
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
    // each station's tool mount, at the same times
    let all = crate::stations::data().unwrap_or_default();
    for st in &w.stations {
        let Some(s) = all.iter().find(|s| s.name == st.name) else {
            continue;
        };
        for &t in times.iter().step_by(step) {
            let ext = ext_at(&st.track, t);
            let r = s
                .mount(ext)
                .p
                .map(|x| kin::clean((x * 1000.0).round() / 1000.0));
            let _ = writeln!(out, "expect({}, {}, {})", quote(&s.name), num(t), row(&r));
        }
    }
    // the scope's eye, at the same times (its pan and tilt drawn right)
    for &t in times.iter().step_by(step) {
        let r = lerp_rows(&w.scope.track, t);
        let e = crate::scope::eye(r[1], r[2])
            .p
            .map(|x| kin::clean((x * 1000.0).round() / 1000.0));
        let _ = writeln!(out, "expect('scope', {}, {})", num(t), row(&e));
    }
    // each pincer's jaws' mount, at the same times
    for p in &w.pincers {
        for &t in times.iter().step_by(step) {
            let (_, m) = pincer_at(w, p, t);
            let r = m.p.map(|x| kin::clean((x * 1000.0).round() / 1000.0));
            let _ = writeln!(out, "expect({}, {}, {})", quote(&p.name), num(t), row(&r));
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
    // its sleeves: the span of the lead they cover (the part's frame),
    // loose until shrunk, then snug
    if let (Ok(kind), Ok(d)) = (p.kind_of(), crate::sleeve::data()) {
        for s in &p.sleeves {
            let Some(pin) = kind.pin(&s.pin) else {
                continue;
            };
            let Some(size) = d.size.iter().find(|z| z.name == s.size) else {
                continue;
            };
            let (_, tail) = crate::leads::split(&pin.lead, s.from);
            let (span, _) = crate::leads::split(&tail, s.to - s.from);
            let pts: Vec<f64> = span
                .iter()
                .flat_map(|q| q.iter().map(|x| kin::clean((x * 1000.0).round() / 1000.0)))
                .collect();
            let loose = size.before / 2.0 + size.wall;
            let snug = pin.diameter.max(size.after) / 2.0 + size.wall;
            let _ = writeln!(
                out,
                "sleeve({}, {}, {}, {}, {}, {}, {}, {})",
                quote(&p.id),
                quote(&s.pin),
                num(s.on),
                s.shrunk.map_or_else(|| "null".to_string(), num),
                table(&[pts], 3),
                num(loose),
                num(snug),
                quote(&s.color)
            );
        }
    }
    // its marking and the body it is on, for drawing (and reading back)
    if let (Some(mark), Ok(kind)) = (&p.mark, p.kind_of()) {
        if let Ok(m) = crate::marks::marking(&p.kind, mark) {
            let pin1 = kind.pins.first().and_then(|x| x.lead.first().copied());
            let json = serde_json::json!({
                "marking": m,
                "body": {
                    "shape": kind.body.shape,
                    "size": kind.body.size,
                    "diameter": kind.body.diameter,
                    "length": kind.body.length,
                    "axis": kind.body.axis,
                    "flat": kind.body.flat,
                },
                "pin1": pin1,
                // a diode's band printed at its anode, not its cathode
                "reversed": p.fault.as_deref() == Some("reversed"),
            });
            let _ = writeln!(out, "mark({}, {})", quote(&p.id), json);
        }
    }
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
    // each lead re-formed: from then on, drawn as this polyline
    if !p.reformed.is_empty() {
        let kind = p.kind_of().ok();
        for (t, pin, lead) in &p.reformed {
            let d = kind
                .as_ref()
                .and_then(|k| k.pin(pin))
                .map_or(0.5, |x| x.diameter);
            let pts: Vec<f64> = lead
                .iter()
                .flat_map(|q| q.iter().map(|x| kin::clean((x * 1000.0).round() / 1000.0)))
                .collect();
            let _ = writeln!(
                out,
                "lead({}, {}, {}, {}, {})",
                quote(&p.id),
                quote(pin),
                num(*t),
                num(d),
                table(&[pts], 3)
            );
        }
    }
}

/// A station's extension at time `t`, from its track (linear between
/// rows, held at the ends).
fn ext_at(track: &[[f64; 2]], t: f64) -> f64 {
    match track.iter().position(|r| r[0] > t) {
        None => track.last().map_or(0.0, |r| r[1]),
        Some(0) => track[0][1],
        Some(k) => {
            let (a, b) = (track[k - 1], track[k]);
            a[1] + (b[1] - a[1]) * (t - a[0]) / (b[0] - a[0]).max(1e-12)
        }
    }
}

/// Linear between rows, held at the ends: row `k`'s columns at time `t`.
fn lerp_rows<const N: usize>(rows: &[[f64; N]], t: f64) -> [f64; N] {
    match rows.iter().position(|r| r[0] > t) {
        None => rows.last().copied().unwrap_or([0.0; N]),
        Some(0) => rows[0],
        Some(k) => {
            let (a, b) = (rows[k - 1], rows[k]);
            let f = (t - a[0]) / (b[0] - a[0]).max(1e-12);
            std::array::from_fn(|i| a[i] + (b[i] - a[i]) * f)
        }
    }
}

/// A pincer's housing and jaws' mount in the cell's frame at time `t`:
/// its own track on the turntable, the turntable's track under it.
fn pincer_at(w: &World, p: &crate::pincers::PincerState, t: f64) -> (kin::Pose, kin::Pose) {
    let s = lerp_rows(&w.stage_track, t);
    let frame = crate::layout::positioner(s[1], s[2], [s[3], s[4], s[5]]);
    let q = lerp_rows(&p.track, t);
    let (h, m) = crate::pincers::local(q[1], q[2], q[3], q[4]);
    (frame.then(&h), frame.then(&m))
}

/// The times a pincer's drawing needs: its track's and the turntable's.
fn pincer_times(w: &World, p: &crate::pincers::PincerState) -> Vec<f64> {
    let mut t: Vec<f64> = p
        .track
        .iter()
        .map(|r| r[0])
        .chain(w.stage_track.iter().map(|r| r[0]))
        .collect();
    t.sort_by(f64::total_cmp);
    t.dedup_by(|a, b| (*a - *b).abs() < 1e-9);
    // riding the turntable a pincer moves on an arc 130 mm out: four
    // points inside each short step of a motion keep the chords a player
    // draws within a few thousandths of a millimeter of it
    let mut out = Vec::with_capacity(t.len() * 5);
    for w in t.windows(2) {
        out.push(w[0]);
        if w[1] - w[0] < 0.2 {
            for k in 1..5 {
                out.push(w[0] + (w[1] - w[0]) * f64::from(k) / 5.0);
            }
        }
    }
    out.extend(t.last());
    out
}

/// Angle columns made continuous down the rows (a step past 180 degrees
/// taken the short way), so a player interpolating them turns the short
/// way too.
fn unwrap_angles(rows: &mut [Vec<f64>], cols: &[usize]) {
    for k in 1..rows.len() {
        for &c in cols {
            let prev = rows[k - 1][c];
            let mut x = rows[k][c];
            while x - prev > 180.0 {
                x -= 360.0;
            }
            while prev - x > 180.0 {
                x += 360.0;
            }
            rows[k][c] = x;
        }
    }
}
