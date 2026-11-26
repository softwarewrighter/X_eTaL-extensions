//! Markings (decision C12, step 22): what is printed or banded on a part,
//! and what it means. A circuit gives each part its mark (`mark = "4.7k"`,
//! `"104"`, `"NE555P"`, `"red"`), else its kind's default; this module
//! turns a mark into what is on the body (bands, text, a color), on which
//! side, and back: the resistor color code (4 and 5 bands, read from
//! either end) and EIA capacitor codes.

use serde::{Deserialize, Serialize};

/// The band colors, by digit.
pub const DIGITS: [&str; 10] = [
    "black", "brown", "red", "orange", "yellow", "green", "blue", "violet", "gray", "white",
];

/// Each band color and its sRGB, for drawing and for reading a picture.
pub const RGB: [(&str, [u8; 3]); 12] = [
    ("black", [20, 20, 20]),
    ("brown", [120, 66, 18]),
    ("red", [200, 30, 30]),
    ("orange", [240, 130, 20]),
    ("yellow", [240, 220, 40]),
    ("green", [30, 150, 50]),
    ("blue", [30, 70, 200]),
    ("violet", [140, 60, 190]),
    ("gray", [128, 128, 128]),
    ("white", [240, 240, 240]),
    ("gold", [205, 165, 50]),
    ("silver", [190, 190, 200]),
];

/// Where a marking is: all round the body, on its top (+z), on a face
/// (a disc's +y, a TO-92's flat at -y).
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// Seen from any side.
    Around,
    /// On the body's top, +z in the part's frame.
    Top,
    /// On the face toward +y in the part's frame.
    FacePlusY,
    /// On the face toward -y in the part's frame.
    FaceMinusY,
}

impl Side {
    /// The marked side's outward direction in the part's frame, if it
    /// has one.
    #[must_use]
    pub fn normal(self) -> Option<[f64; 3]> {
        match self {
            Self::Around => None,
            Self::Top => Some([0.0, 0.0, 1.0]),
            Self::FacePlusY => Some([0.0, 1.0, 0.0]),
            Self::FaceMinusY => Some([0.0, -1.0, 0.0]),
        }
    }

    /// In words.
    #[must_use]
    pub fn words(self) -> &'static str {
        match self {
            Self::Around => "all round",
            Self::Top => "its top",
            Self::FacePlusY | Self::FaceMinusY => "its marked face",
        }
    }
}

/// What a mark puts on a body.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct Marking {
    /// The mark as the circuit gives it: `4.7k`, `104`.
    pub mark: String,
    /// Color bands, body to the end of pin 2 (resistors), or a diode's
    /// cathode band (one band, at pin 1's end).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bands: Vec<String>,
    /// Printed text, lines.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text: Vec<String>,
    /// The body's color, if the mark is one (an LED).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A pin-1 dot on the top (an IC).
    #[serde(default)]
    pub pin1_dot: bool,
    /// Where it is.
    pub side: Side,
    /// What it means: `4.7 kilohm, 5%`.
    pub value: String,
}

/// A kind's mark when its circuit gives none.
#[must_use]
pub fn default_mark(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "axial-resistor" => "10k",
        "ceramic-disc" => "104",
        "radial-electrolytic" => "10uF 25V",
        "dip8" => "NE555P",
        "dip14" => "SN74HC00N",
        "dip16" => "SN74HC595N",
        "to92" => "2N3904",
        "led-5mm" => "red",
        "axial-diode" => "1N4148",
        _ => return None,
    })
}

/// A kind's marking from a mark.
///
/// # Errors
///
/// A mark that kind cannot carry: a resistor value that is not a number
/// with an SI prefix, a capacitor code that is not EIA, an LED color that
/// is not one.
pub fn marking(kind: &str, mark: &str) -> Result<Marking, String> {
    let mut m = Marking {
        mark: mark.to_string(),
        bands: Vec::new(),
        text: Vec::new(),
        color: None,
        pin1_dot: false,
        side: Side::Around,
        value: String::new(),
    };
    match kind {
        "axial-resistor" => {
            let ohms = parse_ohms(mark)?;
            // three significant digits need five bands
            let five = sig_digits(ohms) > 2;
            m.bands = resistor_bands(ohms, five)?;
            let (v, tol) = decode_bands(&m.bands)?;
            m.value = format!("{}, {tol}%", ohms_words(v));
        }
        "ceramic-disc" => {
            let pf = eia_pf(mark)?;
            m.text = vec![mark.to_string()];
            m.side = Side::FacePlusY;
            m.value = farads_words(pf * 1e-12);
        }
        "radial-electrolytic" => {
            let (uf, volts) = parse_electrolytic(mark)?;
            m.text = vec![format!("{}uF", trim(uf)), format!("{}V", trim(volts))];
            m.value = format!("{}, {} V", farads_words(uf * 1e-6), trim(volts));
        }
        "dip8" | "dip14" | "dip16" => {
            m.text = vec![mark.to_string()];
            m.pin1_dot = true;
            m.side = Side::Top;
            m.value = format!("the part number {mark}");
        }
        "to92" => {
            m.text = vec![mark.to_string()];
            m.side = Side::FaceMinusY;
            m.value = format!("the part number {mark}");
        }
        "axial-diode" => {
            m.text = vec![mark.to_string()];
            m.bands = vec!["black".into()];
            m.value = format!("the part number {mark}, its cathode at pin 1");
        }
        "led-5mm" => {
            let c = mark.to_lowercase();
            if !["red", "green", "yellow", "blue", "white", "orange"].contains(&c.as_str()) {
                return Err(format!("{mark} is not an LED color"));
            }
            m.color = Some(c.clone());
            m.value = format!("a {c} LED");
        }
        _ => return Err(format!("a {kind} carries no marking")),
    }
    Ok(m)
}

fn trim(x: f64) -> String {
    let s = format!("{x:.3}");
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// `4.7k`, `4k7`, `470`, `1M`, `2.2 kilohm`: ohms.
///
/// # Errors
///
/// Not a number with an optional R, k or M.
pub fn parse_ohms(text: &str) -> Result<f64, String> {
    let t = text
        .trim()
        .replace("kilohms", "k")
        .replace("kilohm", "k")
        .replace("megohms", "M")
        .replace("megohm", "M")
        .replace("ohms", "")
        .replace("ohm", "")
        .replace(' ', "");
    let bad = || format!("{text} is not a resistance (4.7k, 4k7, 470, 1M)");
    let (mult, letter) = if t.contains('k') {
        (1e3, 'k')
    } else if t.contains('M') {
        (1e6, 'M')
    } else if t.contains('R') {
        (1.0, 'R')
    } else {
        (1.0, ' ')
    };
    let num: f64 = if letter == ' ' {
        t.parse().map_err(|_| bad())?
    } else {
        let (a, b) = t.split_once(letter).ok_or_else(bad)?;
        if b.is_empty() {
            a.parse().map_err(|_| bad())?
        } else if a.contains('.') {
            return Err(bad());
        } else {
            // 4k7: the letter is the decimal point
            format!("{a}.{b}").parse().map_err(|_| bad())?
        }
    };
    if !(num.is_finite() && num > 0.0) {
        return Err(bad());
    }
    Ok(num * mult)
}

/// The significant digits a value needs (up to three).
fn sig_digits(ohms: f64) -> usize {
    let e = ohms.log10().floor();
    let m = ohms / 10f64.powf(e - 2.0); // three digits
    let m = m.round() as i64;
    if m % 100 == 0 {
        1
    } else if m % 10 == 0 {
        2
    } else {
        3
    }
}

/// The bands for a value: two digits, the multiplier and gold (5%), or
/// with `five` three digits, the multiplier and brown (1%).
///
/// # Errors
///
/// Below 1 ohm or above what a multiplier can say.
pub fn resistor_bands(ohms: f64, five: bool) -> Result<Vec<String>, String> {
    let digits = if five { 3 } else { 2 };
    let e = ohms.log10().floor() as i32;
    let mult = e - (digits - 1);
    let sig = (ohms / 10f64.powi(mult)).round() as i64;
    if !(-2..=9).contains(&mult) {
        return Err(format!("{} is beyond the color code", ohms_words(ohms)));
    }
    let mut out: Vec<String> = sig
        .to_string()
        .chars()
        .map(|c| DIGITS[c.to_digit(10).unwrap_or(0) as usize].to_string())
        .collect();
    out.push(match mult {
        -2 => "silver".to_string(),
        -1 => "gold".to_string(),
        m => DIGITS[m as usize].to_string(),
    });
    out.push(if five { "brown" } else { "gold" }.to_string());
    Ok(out)
}

fn tolerance(color: &str) -> Option<f64> {
    Some(match color {
        "brown" => 1.0,
        "red" => 2.0,
        "green" => 0.5,
        "blue" => 0.25,
        "violet" => 0.1,
        "gold" => 5.0,
        "silver" => 10.0,
        _ => return None,
    })
}

fn digit(color: &str) -> Option<i64> {
    DIGITS.iter().position(|d| *d == color).map(|d| d as i64)
}

fn multiplier(color: &str) -> Option<f64> {
    match color {
        "gold" => Some(0.1),
        "silver" => Some(0.01),
        c => digit(c).map(|d| 10f64.powi(d as i32)),
    }
}

/// Bands in the order given, digits first: ohms and tolerance (%).
fn decode_forward(bands: &[String]) -> Result<(f64, f64), String> {
    let n = bands.len();
    if !(4..=5).contains(&n) {
        return Err(format!("{n} bands: a resistor has 4 or 5"));
    }
    let tol = tolerance(&bands[n - 1])
        .ok_or_else(|| format!("{} is not a tolerance band", bands[n - 1]))?;
    let mult = multiplier(&bands[n - 2])
        .ok_or_else(|| format!("{} is not a multiplier band", bands[n - 2]))?;
    let mut sig = 0;
    for b in &bands[..n - 2] {
        sig = sig * 10 + digit(b).ok_or_else(|| format!("{b} is not a digit band"))?;
    }
    if sig == 0 {
        return Err("the digit bands read zero".into());
    }
    Ok((sig as f64 * mult, tol))
}

/// Bands as seen, from either end: ohms and tolerance (%). A part may lie
/// either way round: the end that makes sense is the one read; if both
/// do, the one whose tolerance band is gold or silver, else the one in
/// the E24 series.
///
/// # Errors
///
/// Neither way reads as a resistor.
pub fn decode_bands(bands: &[String]) -> Result<(f64, f64), String> {
    let fwd = decode_forward(bands);
    let rev: Vec<String> = bands.iter().rev().cloned().collect();
    let back = decode_forward(&rev);
    match (fwd, back) {
        (Ok(a), Err(_)) => Ok(a),
        (Err(_), Ok(b)) => Ok(b),
        (Ok(a), Ok(b)) => {
            let metallic = |c: &str| c == "gold" || c == "silver";
            let (fa, fb) = (
                metallic(&bands[bands.len() - 1]),
                metallic(&rev[rev.len() - 1]),
            );
            if fa != fb {
                return Ok(if fa { a } else { b });
            }
            Ok(if e24(a.0) || !e24(b.0) { a } else { b })
        }
        (Err(e), Err(_)) => Err(e),
    }
}

/// Whether a value is in the E24 series.
#[must_use]
pub fn e24(ohms: f64) -> bool {
    const E24: [i64; 24] = [
        10, 11, 12, 13, 15, 16, 18, 20, 22, 24, 27, 30, 33, 36, 39, 43, 47, 51, 56, 62, 68, 75, 82,
        91,
    ];
    let e = ohms.log10().floor();
    let m = (ohms / 10f64.powf(e - 1.0)).round() as i64;
    E24.contains(&m)
}

/// `4.7 kilohm`.
#[must_use]
pub fn ohms_words(ohms: f64) -> String {
    if ohms >= 1e6 {
        format!("{} megohm", trim(ohms / 1e6))
    } else if ohms >= 1e3 {
        format!("{} kilohm", trim(ohms / 1e3))
    } else {
        format!("{} ohm", trim(ohms))
    }
}

/// An EIA capacitor code: `104` (10 and four zeros, pF), `4R7` (4.7 pF),
/// `47` (47 pF), `0.1` (uF, as printed on larger parts): pF.
///
/// # Errors
///
/// Not such a code.
pub fn eia_pf(code: &str) -> Result<f64, String> {
    let c = code.trim();
    let bad = || format!("{code} is not an EIA capacitor code (104, 4R7, 47)");
    if let Some((a, b)) = c.split_once('R') {
        return format!("{a}.{b}").parse().map_err(|_| bad());
    }
    if c.contains('.') {
        let uf: f64 = c.parse().map_err(|_| bad())?;
        return Ok(uf * 1e6);
    }
    if !c.chars().all(|ch| ch.is_ascii_digit()) || c.is_empty() {
        return Err(bad());
    }
    if c.len() <= 2 {
        return c.parse().map_err(|_| bad());
    }
    if c.len() != 3 {
        return Err(bad());
    }
    let sig: f64 = c[..2].parse().map_err(|_| bad())?;
    let zeros = c[2..].parse::<i32>().map_err(|_| bad())?;
    if zeros > 6 {
        return Err(bad());
    }
    Ok(sig * 10f64.powi(zeros))
}

/// The EIA code for a value in pF: `104` for 100 nF.
#[must_use]
pub fn eia_code(pf: f64) -> String {
    if pf < 10.0 {
        return trim(pf).replace('.', "R");
    }
    if pf < 100.0 {
        return format!("{}", pf.round());
    }
    let e = pf.log10().floor() as i32;
    let sig = (pf / 10f64.powi(e - 1)).round() as i64;
    format!("{sig}{}", e - 1)
}

/// `10uF 25V`: microfarads and volts.
///
/// # Errors
///
/// Not a value in uF and a voltage.
pub fn parse_electrolytic(text: &str) -> Result<(f64, f64), String> {
    let bad = || format!("{text} is not a value and a voltage (10uF 25V)");
    let t = text.replace(' ', "");
    let (a, b) = t.split_once("uF").ok_or_else(bad)?;
    let uf: f64 = a.parse().map_err(|_| bad())?;
    let volts: f64 = b.trim_end_matches('V').parse().map_err(|_| bad())?;
    Ok((uf, volts))
}

/// `100 nF`.
#[must_use]
pub fn farads_words(f: f64) -> String {
    if f >= 1e-6 {
        format!("{} uF", trim(f * 1e6))
    } else if f >= 1e-9 {
        format!("{} nF", trim(f * 1e9))
    } else {
        format!("{} pF", trim(f * 1e12))
    }
}

/// The band color nearest an observed sRGB, and how near (0 far, 1 the
/// same): what reading a band in a picture comes to.
#[must_use]
pub fn nearest_color(rgb: [f64; 3]) -> (&'static str, f64) {
    let mut best = ("black", f64::INFINITY);
    for (name, c) in RGB {
        let d = (0..3)
            .map(|k| (rgb[k] - f64::from(c[k])).powi(2))
            .sum::<f64>()
            .sqrt();
        if d < best.1 {
            best = (name, d);
        }
    }
    // 441 is black to white
    (best.0, (1.0 - best.1 / 120.0).clamp(0.0, 1.0))
}

/// Bands read from a picture: each band's sampled color, the body's
/// sampled color between them and the body's true color (sRGB, 0 to
/// 255). The samples are white-balanced against the body (the light and
/// the camera tint every pixel alike), each taken for its nearest band
/// color, and the bands decoded from either end.
#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct BandReading {
    /// The colors read, in the order sampled.
    pub bands: Vec<String>,
    /// How near each sample came to its color, the least of them: 0 to 1.
    pub confidence: f64,
    /// What they mean, or why they mean nothing.
    pub value: Result<String, String>,
}

/// Reads sampled band colors (see [`BandReading`]).
#[must_use]
pub fn read_bands(samples: &[[f64; 3]], body_seen: [f64; 3], body_true: [f64; 3]) -> BandReading {
    let gain: Vec<f64> = (0..3)
        .map(|k| (body_true[k] / body_seen[k].max(1.0)).clamp(0.2, 5.0))
        .collect();
    let mut bands = Vec::new();
    let mut confidence: f64 = 1.0;
    for s in samples {
        let c = [
            (s[0] * gain[0]).min(255.0),
            (s[1] * gain[1]).min(255.0),
            (s[2] * gain[2]).min(255.0),
        ];
        let (name, q) = nearest_color(c);
        bands.push(name.to_string());
        confidence = confidence.min(q);
    }
    let value = decode_bands(&bands).map(|(v, t)| format!("{}, {t}%", ohms_words(v)));
    BandReading {
        bands,
        confidence,
        value,
    }
}
