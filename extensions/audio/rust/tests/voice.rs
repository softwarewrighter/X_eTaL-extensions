//! A voice with no device writes what it is given to a WAV, which the
//! decoder reads back.
#![allow(unsafe_code)]

use xetal_ext_audio::decode::Stream;
use xetal_ext_audio::voice::Voice;

#[test]
#[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
fn a_voice_writes_its_samples_to_a_wav() {
    let dir = tempfile::tempdir().unwrap();
    let wav = dir.path().join("out.wav");
    // SAFETY: the only test in this binary.
    unsafe {
        std::env::set_var("XETAL_AUDIO", "off");
        std::env::set_var("XETAL_AUDIO_WAV", &wav);
    }
    let mut v = Voice::open(8000).unwrap();
    let tone: Vec<f32> = (0..8000)
        .map(|i| (0.5 * (std::f64::consts::TAU * 500.0 * f64::from(i) / 8000.0).sin()) as f32)
        .collect();
    let quiet = vec![0.0f32; 8000];
    assert!(
        (v.queue(&tone, &quiet) - 0.0).abs() < 1e-12,
        "no device: nothing waits"
    );
    v.queue(&tone[..4000], &tone[..4000]);
    assert_eq!(v.played(), 12000);
    v.finish().unwrap();

    let mut s = Stream::open(&wav).unwrap();
    assert_eq!((s.rate(), s.channels(), s.frames()), (8000, 2, Some(12000)));
    let c = s.next(8000).unwrap();
    let crossings = c
        .left
        .windows(2)
        .filter(|w| (w[0] < 0.0) != (w[1] < 0.0))
        .count();
    assert!(
        (990..=1010).contains(&crossings),
        "500 Hz for a second: {crossings}"
    );
    assert!(
        c.right.iter().all(|&x| x == 0.0),
        "the right channel was silent"
    );
    assert!(Voice::open(100).is_err());
}
