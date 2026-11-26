//! The microscope (decision C3; step 20): its head on the gantry pans
//! about the vertical and tilts, its eye 120 mm below the tilt; `aim`
//! turns it so the eye's axis passes through a pin or joint; zoom gives
//! the field of view (120 mm over the zoom), focus moves the focal plane,
//! rotate turns the picture. Its view is what the player draws in its
//! picture beside the cell; `snap` records a picture's moment.

use crate::command::num;
use crate::kin::{self, Pose, V3};
use crate::world::{Halt, World, micro, num1};
use serde::{Deserialize, Serialize};

/// The field of view at zoom 1, mm across.
pub const FIELD_AT_1: f64 = 120.0;
/// How fast the head pans and tilts, degrees a second.
pub const SPEED: f64 = 60.0;
/// How long a zoom or focus change takes, s.
pub const ZOOM_TIME: f64 = 0.5;

/// The scope's state.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Scope {
    /// About the vertical, degrees.
    pub pan: f64,
    /// From straight down, degrees.
    pub tilt: f64,
    /// The magnification.
    pub zoom: f64,
    /// The focal plane's offset, mm.
    pub focus: f64,
    /// The picture about its axis, degrees.
    pub rotate: f64,
    /// What it is aimed at, if anything.
    pub aim: Option<String>,
    /// Rows of the time, pan, tilt, the view's angle (degrees), focus,
    /// rotate and the distance to what it looks at (mm), for players.
    pub track: Vec<[f64; 7]>,
    /// Each picture taken: the time, what it showed, its field (mm).
    pub snaps: Vec<(f64, String, f64)>,
}

impl Default for Scope {
    fn default() -> Self {
        Self {
            pan: 0.0,
            tilt: 0.0,
            zoom: 1.0,
            focus: 0.0,
            rotate: 0.0,
            aim: None,
            track: vec![[0.0, 0.0, 0.0, 30.0, 0.0, 0.0, 330.0]],
            snaps: Vec::new(),
        }
    }
}

/// The scope's fixture frame (the gantry), from the layout.
fn frame() -> Pose {
    let at = crate::layout::fixtures()
        .into_iter()
        .find(|f| f.0 == "scope")
        .map_or([0.0, 0.0, 92.0, 0.0, 0.0, 135.0], |f| f.2);
    Pose::from_xyz_rpy([at[0], at[1], at[2]], [at[3], at[4], at[5]])
}

/// The tilt's pivot in the cell's frame (the pan turns about it).
#[must_use]
pub fn pivot() -> V3 {
    frame()
        .then(&Pose::from_xyz_rpy([0.0, 0.0, 388.0 - 30.0], [0.0; 3]))
        .p
}

/// The eye's pose in the cell's frame at a pan and tilt: its z the way
/// it looks (down at 0, 0).
#[must_use]
pub fn eye(pan: f64, tilt: f64) -> Pose {
    frame()
        .then(&Pose::from_xyz_rpy(
            [0.0, 0.0, 388.0 - 30.0],
            [0.0, 0.0, pan],
        ))
        .then(&Pose::from_xyz_rpy([0.0; 3], [tilt, 0.0, 0.0]))
        .then(&Pose::from_xyz_rpy([0.0, 0.0, -120.0], [180.0, 0.0, 0.0]))
}

/// The pan and tilt that point the eye's axis through `at`.
#[must_use]
pub fn aim_at(at: V3, pan_now: f64) -> (f64, f64) {
    let d = kin::sub(at, pivot());
    let ds = kin::mv(&kin::transpose(&frame().r), d);
    let n = kin::norm(ds).max(1e-9);
    let ds = kin::scale(ds, 1.0 / n);
    // R_z(pan) R_x(tilt) (0, 0, -1) = (-sin pan sin tilt, cos pan sin tilt, -cos tilt)
    let tilt = (-ds[2]).clamp(-1.0, 1.0).acos().to_degrees();
    if tilt.abs() < 1e-6 {
        return (pan_now, 0.0);
    }
    let pan = (-ds[0]).atan2(ds[1]).to_degrees();
    (pan, tilt)
}

/// The view's angle across at a zoom, looking at something `dist` mm
/// away: degrees.
#[must_use]
pub fn view_angle(zoom: f64, dist: f64) -> f64 {
    let field = FIELD_AT_1 / zoom.max(1e-9);
    2.0 * (field / 2.0 / dist.max(1.0)).atan().to_degrees()
}

impl World {
    /// Where the scope looks: what it is aimed at, else the work zone
    /// straight below its eye.
    fn scope_target(&self) -> V3 {
        if let Some(t) = self.scope.aim.as_deref() {
            if let Ok(p) = self.spot_point(t) {
                return p;
            }
        }
        let e = eye(self.scope.pan, self.scope.tilt);
        kin::add(e.p, kin::scale(e.z(), e.p[2].abs().max(50.0)))
    }

    /// A pin's frame, a joint's place or a part's origin, for aiming.
    fn spot_point(&self, target: &str) -> Result<V3, String> {
        if !target.contains('.') && !target.contains('+') {
            return self
                .part(target)
                .map(|p| p.pose().p)
                .ok_or_else(|| format!("no part {target}"));
        }
        if target.contains('+') {
            let (a, b) = target.split_once('+').ok_or("not a joint")?;
            let j = self
                .joints
                .iter()
                .find(|j| (j.a == a && j.b == b) || (j.a == b && j.b == a))
                .ok_or_else(|| format!("{target} is not a joint yet"))?;
            return self
                .joint_point(j)
                .ok_or_else(|| format!("{target}: its part is gone"));
        }
        Ok(self.frame(target)?.p)
    }

    /// `scope [pan=] [tilt=] [zoom=] [focus=] [rotate=] [aim=]`: the head
    /// turns (aim works out pan and tilt), the zoom and focus change.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn move_scope(
        &mut self,
        pan: Option<f64>,
        tilt: Option<f64>,
        zoom: Option<f64>,
        focus: Option<f64>,
        rotate: Option<f64>,
        aim: Option<&str>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let s = self.scope.clone();
        let (mut p, mut t) = (pan.unwrap_or(s.pan), tilt.unwrap_or(s.tilt));
        if let Some(target) = aim {
            let at = self.spot_point(target).map_err(Halt::Refused)?;
            (p, t) = aim_at(at, s.pan);
            if t.abs() > 90.0 {
                return Err(Halt::Refused(format!(
                    "{target} is beyond the scope's tilt (90 degrees)"
                )));
            }
        }
        let z = zoom.unwrap_or(s.zoom);
        let f = focus.unwrap_or(s.focus);
        let r = rotate.unwrap_or(s.rotate);
        let dp = kin::wrap(p - s.pan);
        let turn = dp.abs().max((t - s.tilt).abs()) / SPEED;
        let lens = if (z - s.zoom).abs() > 1e-9 || (f - s.focus).abs() > 1e-9 {
            ZOOM_TIME
        } else {
            0.0
        };
        let dt = turn.max(lens).max(0.05);
        let t0 = self.t;
        // where it looks before, then after, the angle from the eye's distance
        self.scope.aim = aim.map(str::to_string).or(s.aim.clone());
        let dist = |w: &World, pan: f64, tilt: f64| {
            kin::norm(kin::sub(w.scope_target(), eye(pan, tilt).p))
        };
        let d0 = s.track.last().map_or(330.0, |r| r[6]);
        let d1 = dist(self, s.pan + dp, t);
        let a0 = s.track.last().map_or(30.0, |r| r[3]);
        let a1 = view_angle(z, d1);
        let n = 10;
        for i in 0..=n {
            let k = f64::from(i) / f64::from(n);
            self.scope.track.push([
                micro(t0 + dt * k),
                s.pan + dp * k,
                s.tilt + (t - s.tilt) * k,
                a0 + (a1 - a0) * k,
                s.focus + (f - s.focus) * k,
                s.rotate + (r - s.rotate) * k,
                d0 + (d1 - d0) * k,
            ]);
        }
        (
            self.scope.pan,
            self.scope.tilt,
            self.scope.zoom,
            self.scope.focus,
            self.scope.rotate,
        ) = (s.pan + dp, t, z, f, r);
        self.advance(dt, None);
        did.push(format!(
            "scope to pan {} tilt {} zoom {} in {} s",
            num1(self.scope.pan),
            num1(t),
            num(z),
            num1(dt)
        ));
        let field = FIELD_AT_1 / z;
        events.push(match aim {
            Some(target) => format!("the scope looks at {target}, {} mm across", num1(field)),
            None => format!("the scope's field is {} mm across", num1(field)),
        });
        Ok(())
    }

    /// After the work moved from `t0` over `dt`, a scope with an aim
    /// turns back onto it over the same span (its view drifts in between).
    pub(crate) fn follow_aim(&mut self, t0: f64, dt: f64) {
        let Some(target) = self.scope.aim.clone() else {
            return;
        };
        let Ok(at) = self.spot_point(&target) else {
            return;
        };
        let s = self.scope.clone();
        let (p, t) = aim_at(at, s.pan);
        if t.abs() > 90.0 {
            return;
        }
        let dp = kin::wrap(p - s.pan);
        let d = kin::norm(kin::sub(at, eye(s.pan + dp, t).p));
        let a0 = s.track.last().map_or(30.0, |r| r[3]);
        let d0 = s.track.last().map_or(330.0, |r| r[6]);
        let a1 = view_angle(s.zoom, d);
        // hold the last row until the move starts
        if let Some(last) = s.track.last() {
            if last[0] < t0 {
                let mut hold = *last;
                hold[0] = micro(t0);
                self.scope.track.push(hold);
            }
        }
        let n = 10;
        for i in 1..=n {
            let k = f64::from(i) / f64::from(n);
            self.scope.track.push([
                micro(t0 + dt * k),
                s.pan + dp * k,
                s.tilt + (t - s.tilt) * k,
                a0 + (a1 - a0) * k,
                s.focus,
                s.rotate,
                d0 + (d - d0) * k,
            ]);
        }
        (self.scope.pan, self.scope.tilt) = (s.pan + dp, t);
    }

    /// `snap`: the scope's picture now -- recorded (its time, what it
    /// shows, its field); the player draws it beside the cell.
    pub(crate) fn snap(&mut self, events: &mut Vec<String>) {
        let what = self
            .scope
            .aim
            .clone()
            .unwrap_or_else(|| "the work zone".into());
        let field = FIELD_AT_1 / self.scope.zoom;
        self.scope.snaps.push((self.t, what.clone(), field));
        events.push(format!("a picture of {what}, {} mm across", num1(field)));
    }
}
