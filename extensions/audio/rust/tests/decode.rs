//! Decoding the generated fixtures, and the virtual player, headless.
#![allow(unsafe_code)]

use std::path::Path;

use xetal_ext_audio::decode::{Chunk, Stream};
use xetal_ext_audio::player::{History, Player, State};

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../tests/data")
        .join(name)
}

/// Zero crossings per second: about twice the frequency.
#[allow(clippy::cast_precision_loss)]
fn crossings(x: &[f32], rate: u32) -> f64 {
    let n = x
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    n as f64 * f64::from(rate) / x.len() as f64
}

#[test]
fn each_format_decodes_to_its_tones() {
    for name in ["tone.ogg", "tone.mp3", "tone.wav"] {
        let mut s = Stream::open(&fixture(name)).unwrap();
        assert_eq!((s.rate(), s.channels()), (22050, 2), "{name}");
        let skip = s.next(4000).unwrap(); // past any encoder delay
        assert_eq!(skip.start, 0);
        let c = s.next(6000).unwrap();
        assert_eq!(c.start, 4000, "{name}");
        assert_eq!(c.left.len(), 6000, "{name}");
        let (l, r) = (crossings(&c.left, 22050), crossings(&c.right, 22050));
        assert!((l - 880.0).abs() < 40.0, "{name}: left {l}");
        assert!((r - 1760.0).abs() < 60.0, "{name}: right {r}");
    }
}

#[test]
fn reading_to_the_end_and_seeking() {
    let mut s = Stream::open(&fixture("tone.ogg")).unwrap();
    let mut total = 0;
    loop {
        let c = s.next(5000).unwrap();
        if c.left.is_empty() {
            break;
        }
        total += c.left.len();
    }
    assert!(
        (44000..=44400).contains(&total),
        "{total} frames in 2 s at 22,050 Hz (Vorbis pads to a block)"
    );
    let f = s.seek(1.0).unwrap();
    assert!((21000..=22100).contains(&f), "landed at {f}");
    assert_eq!(s.next(10).unwrap().start, f);
    assert!(s.seek(-1.0).is_err());
    assert!(Stream::open(&fixture("none.ogg")).is_err());
}

#[test]
fn history_windows() {
    let mut h = History::new(8);
    h.push(&Chunk {
        left: vec![1.0, 2.0, 3.0],
        right: vec![-1.0, -2.0, -3.0],
        start: 10,
    });
    assert_eq!(h.end(), 13);
    let (l, r) = h.window(12, 4);
    assert_eq!(l, [0.0, 0.0, 1.0, 2.0]);
    assert_eq!(r, [0.0, 0.0, -1.0, -2.0]);
    // a gap (after a seek) starts over; the cap keeps the newest
    h.push(&Chunk {
        left: (0..10).map(|i| i as f32).collect(),
        right: vec![0.0; 10],
        start: 100,
    });
    assert_eq!(h.end(), 110);
    assert_eq!(h.window(110, 2).0, [8.0, 9.0]);
    assert_eq!(h.window(101, 1).0, [0.0], "dropped by the cap");
}

#[test]
fn the_virtual_player_advances_a_frame_at_a_time() {
    // SAFETY: tests in this binary agree on this value.
    unsafe { std::env::set_var("XETAL_AUDIO", "off") };
    let mut p = Player::new(&fixture("tone.ogg"), 22050);
    assert_eq!(p.state(), State::Stopped);
    p.play().unwrap();
    assert_eq!(p.state(), State::Playing);
    let (l, _) = p.window(256).unwrap();
    assert_eq!(p.position(), 367);
    assert_eq!(l.len(), 256);
    for _ in 0..59 {
        p.window(256).unwrap();
    }
    assert_eq!(p.position(), 60 * 367, "about a second");
    p.pause();
    p.window(256).unwrap();
    assert_eq!((p.state(), p.position()), (State::Paused, 60 * 367));
    p.play().unwrap();
    let t = p.seek(1.9).unwrap();
    assert!((1.8..=1.95).contains(&t), "{t}");
    for _ in 0..30 {
        p.window(256).unwrap();
    }
    assert_eq!(p.state(), State::Stopped, "the end");
}
