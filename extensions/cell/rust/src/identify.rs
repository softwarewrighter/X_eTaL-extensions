//! `identify PART [mode=oracle|sensed]` (decision C12, step 22): the
//! microscope turns onto the part and zooms to it, then reads its marking.
//!
//! - oracle: the marking as the simulator knows it, and what it means.
//! - sensed: as the microscope's picture would give it. Here, in the
//!   simulator, the picture is modeled rather than rendered: how many
//!   pixels its finest feature gets (a band, a line of text), the angle it
//!   is seen at, its focus, and glare off a glossy top seen straight on
//!   set a confidence; under `GOOD` a reading may be wrong (one band or
//!   character taken for its neighbor) and is a warning; under `POOR` the
//!   part is unreadable. The live page also reads the bands from the
//!   rendered picture itself.
//!
//! A marked side facing away from the scope cannot be read at all: the
//! agent must turn the part.

use crate::kin::{self, V3};
use crate::marks::{self, Marking};
use crate::scope::{FIELD_AT_1, eye};
use crate::world::{Halt, World, num1};

/// A reading this sure is right.
pub const GOOD: f64 = 0.6;
/// Below this, nothing can be read.
pub const POOR: f64 = 0.3;
/// The scope's picture, pixels across.
pub const PIXELS: f64 = 320.0;
/// A marked side seen at more than this from straight on is too oblique,
/// degrees.
pub const OBLIQUE: f64 = 75.0;

/// Band colors a poor picture confuses, each with its neighbor.
const CONFUSED: [(&str, &str); 8] = [
    ("red", "orange"),
    ("orange", "red"),
    ("brown", "red"),
    ("violet", "blue"),
    ("blue", "violet"),
    ("gray", "silver"),
    ("gold", "yellow"),
    ("yellow", "gold"),
];

/// Characters a poor picture confuses.
fn confuse_char(c: char) -> char {
    match c {
        '0' => 'O',
        'O' => '0',
        '1' => 'I',
        '5' => 'S',
        '8' => 'B',
        '4' => 'A',
        x => x,
    }
}

/// The finest feature of a marking, mm: a band's width, a character's
/// height.
fn finest(m: &Marking, size: f64) -> f64 {
    if m.bands.len() > 1 {
        // the bands share the body's middle with the gaps between them
        size * 0.6 / (2.0 * m.bands.len() as f64)
    } else if !m.text.is_empty() {
        (size * 0.8 / m.text.iter().map(String::len).max().unwrap_or(1) as f64 * 1.4).min(1.6)
    } else {
        // a color: the whole body
        size
    }
}

/// How the scope sees a marking: a confidence, and why it is not higher.
#[derive(Clone, Debug, PartialEq)]
pub struct Seen {
    /// 0 to 1.
    pub confidence: f64,
    /// What held it back, in words.
    pub because: Vec<String>,
}

/// The confidence of a reading from the pixels the finest feature gets,
/// the angle (degrees from straight on), the focus error over the depth of
/// field, and glare.
#[must_use]
pub fn seen(pixels: f64, angle: f64, blur: f64, glare: bool) -> Seen {
    let mut because = Vec::new();
    // four pixels a feature reads well, under two not at all
    let res = ((pixels - 2.0) / 2.0).clamp(0.0, 1.0);
    if res < 1.0 {
        because.push(format!("{} pixels to its finest feature", num1(pixels)));
    }
    let tilt = (1.0 - (angle - 45.0).max(0.0) / (OBLIQUE - 45.0)).clamp(0.0, 1.0);
    if tilt < 1.0 {
        because.push(format!("seen {} degrees off straight on", num1(angle)));
    }
    let sharp = (1.0 - (blur - 1.0).max(0.0)).clamp(0.0, 1.0);
    if sharp < 1.0 {
        because.push("out of focus".into());
    }
    let shine = if glare {
        because.push("glare off its glossy top".into());
        0.5
    } else {
        1.0
    };
    Seen {
        confidence: res * tilt * sharp * shine,
        because,
    }
}

impl World {
    /// The marking of a part, its kind's size (its largest dimension, mm)
    /// and its pose.
    fn marking_of(&self, id: &str) -> Result<(Marking, f64), Halt> {
        let k = self
            .part_index(id)
            .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
        let p = &self.parts[k];
        let mark = p
            .mark
            .as_deref()
            .ok_or_else(|| Halt::Refused(format!("{id} carries no marking")))?;
        let m = marks::marking(&p.kind, mark).map_err(Halt::Refused)?;
        let kind = p.kind_of().map_err(Halt::Refused)?;
        let b = &kind.body;
        let size = match b.shape.as_str() {
            "box" => b.size.map_or(5.0, |s| s[0].max(s[1]).max(s[2])),
            _ => b.length.unwrap_or(5.0).max(b.diameter.unwrap_or(5.0)),
        };
        Ok((m, size))
    }

    /// `identify PART [mode=]`.
    pub(crate) fn identify(
        &mut self,
        id: &str,
        mode: &str,
        zoom: Option<f64>,
        did: &mut Vec<String>,
        events: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> Result<(), Halt> {
        let (m, size) = self.marking_of(id)?;
        if self.part(id).is_some_and(|p| p.lost) {
            return Err(Halt::Refused(format!("{id} is lost")));
        }
        // the scope onto it, its field three times the part
        let zoom = zoom.unwrap_or_else(|| (FIELD_AT_1 / (3.0 * size)).clamp(1.0, 200.0));
        self.move_scope(
            None,
            None,
            Some(zoom),
            Some(0.0),
            None,
            Some(id),
            did,
            events,
        )?;
        let pose = self
            .part(id)
            .map(|p| p.pose())
            .ok_or_else(|| Halt::Refused(format!("no part {id}")))?;
        let e = eye(self.scope.pan, self.scope.tilt).p;
        let to_eye = kin::sub(e, pose.p);
        let to_eye = kin::scale(to_eye, 1.0 / kin::norm(to_eye).max(1e-9));
        // the marked side's angle to the scope: none for a marking all round
        let angle = m.side.normal().map_or(0.0, |n| {
            let nw = kin::mv(&pose.r, n);
            dot(nw, to_eye).clamp(-1.0, 1.0).acos().to_degrees()
        });
        let what = describe(&m);
        if angle > OBLIQUE {
            warnings.push(format!(
                "{id}: unreadable -- its marking is on {}, {} degrees from the scope's line of sight: turn {id}",
                m.side.words(),
                num1(angle)
            ));
            return Ok(());
        }
        if mode != "sensed" {
            events.push(format!("{id}: {what}: {} (oracle)", m.value));
            return Ok(());
        }
        let field = FIELD_AT_1 / self.scope.zoom;
        let pixels = finest(&m, size) * PIXELS / field * angle.to_radians().cos().max(0.05);
        // a glossy top (an IC's epoxy) seen within 15 degrees of straight on
        // shines back the light above it
        let glare = m.side == marks::Side::Top && angle < 15.0;
        let s = seen(pixels, angle, 0.0, glare);
        let why = if s.because.is_empty() {
            String::new()
        } else {
            format!(" ({})", s.because.join(", "))
        };
        if s.confidence < POOR {
            warnings.push(format!(
                "{id}: unreadable, confidence {}{why}: zoom in, refocus or turn {id}",
                two(s.confidence)
            ));
        } else if s.confidence < GOOD {
            let (read, meaning) = misread(&m, id);
            warnings.push(format!(
                "{id}: read as {read} ({meaning}), confidence {}{why}: it may be wrong; zoom in or turn {id}",
                two(s.confidence)
            ));
        } else {
            events.push(format!(
                "{id}: {what}: {} (sensed, confidence {})",
                m.value,
                two(s.confidence)
            ));
        }
        Ok(())
    }
}

/// Two decimals: a confidence just under a threshold does not print as it.
fn two(x: f64) -> String {
    format!("{x:.2}")
}

fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The marking as it is seen: its bands, its text, its color.
#[must_use]
pub fn describe(m: &Marking) -> String {
    let mut parts = Vec::new();
    if m.bands.len() > 1 {
        parts.push(m.bands.join(" "));
    }
    if !m.text.is_empty() {
        parts.push(format!("\"{}\"", m.text.join(" ")));
    }
    if m.pin1_dot {
        parts.push("a pin-1 dot".into());
    }
    if m.bands.len() == 1 {
        parts.push(format!("a {} band at pin 1", m.bands[0]));
    }
    if let Some(c) = &m.color {
        parts.push(format!("{c} body"));
    }
    parts.join(", ")
}

/// A plausible misreading: one band or character taken for its neighbor
/// (which one from the part's id, so it is the same each time), and what
/// that reading would mean.
#[must_use]
pub fn misread(m: &Marking, id: &str) -> (String, String) {
    let pick = id.bytes().map(usize::from).sum::<usize>();
    if m.bands.len() > 1 {
        let mut b = m.bands.clone();
        let n = b.len();
        for k in 0..n {
            let i = (pick + k) % n;
            if let Some((_, to)) = CONFUSED.iter().find(|(from, _)| *from == b[i]) {
                b[i] = (*to).to_string();
                break;
            }
        }
        let meaning = marks::decode_bands(&b).map_or_else(
            |e| format!("not a resistor: {e}"),
            |(v, t)| format!("{}, {t}%", marks::ohms_words(v)),
        );
        return (b.join(" "), meaning);
    }
    if let Some(line) = m.text.first() {
        let chars: Vec<char> = line.chars().collect();
        let mut out = chars.clone();
        for k in 0..chars.len() {
            let i = (pick + k) % chars.len();
            let c = confuse_char(chars[i]);
            if c != chars[i] {
                out[i] = c;
                break;
            }
        }
        let read: String = out.into_iter().collect();
        return (format!("\"{read}\""), "not a known marking".into());
    }
    (describe(m), m.value.clone())
}
