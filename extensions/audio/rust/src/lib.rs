//! audio: audio files for X_eTaL -- decode Ogg Vorbis, MP3 and WAV
//! (Symphonia), play them on the sound device (CPAL), and read what is
//! playing as arrays, for analysis and pictures (the visualizer).
//! Audio state lives here under Int ids (plan M3).

pub mod decode;
pub mod output;
pub mod player;
pub mod voice;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Mutex, OnceLock};

use decode::Stream;
use player::Player;
use voice::Voice;
use xetal_ext_sdk::{Array, ArrayData, OwnedError, Value, float_vector, text};

struct Open {
    reader: Stream,
    player: Player,
    rate: u32,
    channels: usize,
    frames: Option<u64>,
}

fn opened() -> std::sync::MutexGuard<'static, HashMap<i64, Open>> {
    static O: OnceLock<Mutex<HashMap<i64, Open>>> = OnceLock::new();
    O.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[allow(clippy::cast_possible_truncation)]
fn num(v: &Value, what: &str) -> Result<f64, OwnedError> {
    let (_, x) = float_vector(v)?;
    x.first()
        .copied()
        .ok_or_else(|| OwnedError::invalid_argument(format!("{what}: a number")))
}

#[allow(clippy::cast_possible_truncation)]
fn id_of(v: &Value) -> Result<i64, OwnedError> {
    Ok(num(v, "an audio id")?.round() as i64)
}

fn with<R>(id: i64, f: impl FnOnce(&mut Open) -> Result<R, OwnedError>) -> Result<R, OwnedError> {
    let mut o = opened();
    let a = o
        .get_mut(&id)
        .ok_or_else(|| OwnedError::invalid_argument(format!("no audio {id} is open")))?;
    f(a)
}

fn stereo(left: Vec<f64>, right: Vec<f64>) -> Result<Value, OwnedError> {
    let n = left.len();
    let mut data = left;
    data.extend(right);
    Array::new(vec![2, n], ArrayData::Float(data))
        .map(Value::Array)
        .map_err(|e| OwnedError::failure(e.to_string()))
}

fn voices() -> std::sync::MutexGuard<'static, HashMap<i64, Voice>> {
    static V: OnceLock<Mutex<HashMap<i64, Voice>>> = OnceLock::new();
    V.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

static NEXT: AtomicI64 = AtomicI64::new(1);

/// output rate: a voice playing what the program queues; its id.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn output(args: &[Value]) -> Result<Value, OwnedError> {
    let rate = num(&args[0], "rate")?.round().max(0.0) as u32;
    let v = Voice::open(rate).map_err(OwnedError::failure)?;
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    voices().insert(id, v);
    Ok(Value::Int(id))
}

/// id queue samples: a vector (mono) or 2 by n (left, right), -1 to 1;
/// the seconds now queued ahead.
#[allow(clippy::cast_possible_truncation)]
fn queue(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    let (shape, x) = float_vector(&args[1])?;
    let (l, r): (Vec<f32>, Vec<f32>) = match shape[..] {
        [_] | [] => (
            x.iter().map(|&v| v as f32).collect(),
            x.iter().map(|&v| v as f32).collect(),
        ),
        [2, n] => (
            x[..n].iter().map(|&v| v as f32).collect(),
            x[n..].iter().map(|&v| v as f32).collect(),
        ),
        _ => {
            return Err(OwnedError::invalid_argument(format!(
                "samples are a vector or 2 by n, not {shape:?}"
            )));
        }
    };
    let mut v = voices();
    let voice = v
        .get_mut(&id)
        .ok_or_else(|| OwnedError::invalid_argument(format!("no voice {id} is open")))?;
    Ok(Value::Float(voice.queue(&l, &r)))
}

/// id wait seconds: block until at most that much is queued; the
/// seconds played so far.
#[allow(clippy::cast_precision_loss)]
fn wait(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    let s = num(&args[1], "seconds")?;
    // wait without holding the voices' lock
    loop {
        let (ahead, played, rate) = {
            let v = voices();
            let voice = v
                .get(&id)
                .ok_or_else(|| OwnedError::invalid_argument(format!("no voice {id} is open")))?;
            (voice.ahead(), voice.played(), voice.rate())
        };
        if ahead <= s.max(0.0) {
            return Ok(Value::Float(played as f64 / f64::from(rate)));
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

/// open path: an audio file (Ogg Vorbis, MP3 or WAV); its id.
fn open(args: &[Value]) -> Result<Value, OwnedError> {
    let path = PathBuf::from(text(&args[0])?);
    let reader = Stream::open(&path).map_err(OwnedError::failure)?;
    let (rate, channels, frames) = (reader.rate(), reader.channels(), reader.frames());
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    opened().insert(
        id,
        Open {
            reader,
            player: Player::new(&path, rate),
            rate,
            channels,
            frames,
        },
    );
    Ok(Value::Int(id))
}

/// info id: sample rate, channels, frames and seconds (-1 when the file
/// does not say).
#[allow(clippy::cast_precision_loss)]
fn info(args: &[Value]) -> Result<Value, OwnedError> {
    with(id_of(&args[0])?, |a| {
        let frames = a.frames.map_or(-1.0, |f| f as f64);
        let seconds = a.frames.map_or(-1.0, |f| f as f64 / f64::from(a.rate));
        Array::vector(ArrayData::Float(vec![
            f64::from(a.rate),
            a.channels as f64,
            frames,
            seconds,
        ]))
        .map(Value::Array)
        .map_err(|e| OwnedError::failure(e.to_string()))
    })
}

/// id chunk n: the next n frames read in order (apart from playback), a
/// 2 by m matrix (left, right), m less than n at the end, 0 after it.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn chunk(args: &[Value]) -> Result<Value, OwnedError> {
    let n = num(&args[1], "frames")?.max(1.0) as usize;
    with(id_of(&args[0])?, |a| {
        let c = a.reader.next(n).map_err(OwnedError::failure)?;
        stereo(
            c.left.iter().map(|&x| f64::from(x)).collect(),
            c.right.iter().map(|&x| f64::from(x)).collect(),
        )
    })
}

/// play id: start or resume playing; the id.
fn play(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    with(id, |a| a.player.play().map_err(OwnedError::failure))?;
    Ok(Value::Int(id))
}

/// pause id: pause; the id.
fn pause(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    with(id, |a| {
        a.player.pause();
        Ok(())
    })?;
    Ok(Value::Int(id))
}

/// id seek seconds: move playback there; the second it landed on.
fn seek(args: &[Value]) -> Result<Value, OwnedError> {
    let t = num(&args[1], "seconds")?;
    with(id_of(&args[0])?, |a| {
        a.player
            .seek(t.max(0.0))
            .map(Value::Float)
            .map_err(OwnedError::failure)
    })
}

/// position id: the second playing now.
#[allow(clippy::cast_precision_loss)]
fn position(args: &[Value]) -> Result<Value, OwnedError> {
    with(id_of(&args[0])?, |a| {
        Ok(Value::Float(a.player.position() as f64 / f64::from(a.rate)))
    })
}

/// id window n: the n frames ending at what plays now, a 2 by n matrix
/// (zeros before the start).
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn window(args: &[Value]) -> Result<Value, OwnedError> {
    let n = (num(&args[1], "frames")?.max(1.0) as usize).min(decode::MAX_CHUNK);
    with(id_of(&args[0])?, |a| {
        let (l, r) = a.player.window(n).map_err(OwnedError::failure)?;
        stereo(l, r)
    })
}

/// state id: 0 stopped (not started, or at the end), 1 playing, 2 paused.
fn state(args: &[Value]) -> Result<Value, OwnedError> {
    with(id_of(&args[0])?, |a| {
        Ok(Value::Int(a.player.state() as i64))
    })
}

/// close id: stop and forget a file; a voice plays out what is queued
/// (or writes its WAV) first; 1.
fn close(args: &[Value]) -> Result<Value, OwnedError> {
    let id = id_of(&args[0])?;
    if opened().remove(&id).is_some() {
        return Ok(Value::Int(1));
    }
    let voice = voices()
        .remove(&id)
        .ok_or_else(|| OwnedError::invalid_argument(format!("no audio {id} is open")))?;
    voice.finish().map_err(OwnedError::failure)?;
    Ok(Value::Int(1))
}

xetal_ext_sdk::xetal_extension! {
    name: "audio",
    version: env!("CARGO_PKG_VERSION"),
    functions: {
        open: 1, "Char -> Int", "An audio file (Ogg Vorbis, MP3, WAV); its id.";
        info: 1, "Num a => a -> Float", "Sample rate, channels, frames, seconds (-1 when unknown).";
        chunk: 2, "(Num a, Num b) => a -> b -> Float", "id chunk n: the next n frames in order, 2 by m.";
        play: 1, "Num a => a -> Int", "Start or resume playing.";
        pause: 1, "Num a => a -> Int", "Pause.";
        seek: 2, "(Num a, Num b) => a -> b -> Float", "id seek seconds: move playback; the second it landed on.";
        position: 1, "Num a => a -> Float", "The second playing now.";
        window: 2, "(Num a, Num b) => a -> b -> Float", "id window n: the n frames ending at what plays now, 2 by n.";
        state: 1, "Num a => a -> Int", "0 stopped, 1 playing, 2 paused.";
        close: 1, "Num a => a -> Int", "Stop and forget a file, or finish a voice.";
        output: 1, "Num a => a -> Int", "A voice at rate frames a second; its id.";
        queue: 2, "(Num a, Num b) => a -> b -> Float", "id queue samples (n, or 2 by n; -1 to 1): seconds queued ahead.";
        wait: 2, "(Num a, Num b) => a -> b -> Float", "id wait seconds: until at most that much is queued; seconds played.";
    }
}
