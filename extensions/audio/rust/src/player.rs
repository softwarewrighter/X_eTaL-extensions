//! Playing a file while the program reads what is playing (plan M3).
//!
//! A device player decodes ahead on a thread of its own into the sound
//! device's queue, independent of the program's frame rate; a virtual
//! player (no device: `XETAL_HEADLESS=1` or `XETAL_AUDIO=off`) advances
//! one 60th of a second each time the program reads a window, so tests
//! are exact. Both keep the last few seconds decoded, so `window` gives
//! the frames ending at what is playing now.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::decode::{Chunk, Stream};
use crate::output::Output;

/// The last frames decoded, with the source frame the first one is.
#[derive(Debug)]
pub struct History {
    base: u64,
    left: VecDeque<f32>,
    right: VecDeque<f32>,
    cap: usize,
}

impl History {
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            base: 0,
            left: VecDeque::new(),
            right: VecDeque::new(),
            cap,
        }
    }

    pub fn reset(&mut self, base: u64) {
        self.base = base;
        self.left.clear();
        self.right.clear();
    }

    #[must_use]
    pub fn end(&self) -> u64 {
        self.base + self.left.len() as u64
    }

    pub fn push(&mut self, c: &Chunk) {
        if c.start != self.end() {
            self.reset(c.start);
        }
        self.left.extend(&c.left);
        self.right.extend(&c.right);
        while self.left.len() > self.cap {
            self.left.pop_front();
            self.right.pop_front();
            self.base += 1;
        }
    }

    /// The `n` frames ending at frame `end` (zero where none is kept).
    #[must_use]
    pub fn window(&self, end: u64, n: usize) -> (Vec<f64>, Vec<f64>) {
        let mut l = vec![0.0; n];
        let mut r = vec![0.0; n];
        for i in 0..n {
            let f = end as i128 - n as i128 + i as i128;
            if f >= self.base as i128 && f < self.end() as i128 {
                let j = (f - self.base as i128) as usize;
                l[i] = f64::from(self.left[j]);
                r[i] = f64::from(self.right[j]);
            }
        }
        (l, r)
    }
}

/// Whether there is no sound device to use (tests, servers).
#[must_use]
pub fn virtual_audio() -> bool {
    let on = |k: &str| std::env::var_os(k).is_some_and(|v| !v.is_empty() && v != "0");
    on("XETAL_HEADLESS") || std::env::var("XETAL_AUDIO").is_ok_and(|v| v == "off")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stopped = 0,
    Playing = 1,
    Paused = 2,
}

enum Cmd {
    Play,
    Pause,
    Seek(f64, Sender<Result<u64, String>>),
    Stop,
}

/// Where playback is, shared with the device thread.
#[derive(Default)]
struct Shared {
    state: Option<State>,
    ended: bool,
    seek_frame: u64,
    played_at_seek: u64,
}

enum Clock {
    Device {
        cmds: Sender<Cmd>,
        played: Arc<AtomicU64>,
        device_rate: u32,
        shared: Arc<Mutex<Shared>>,
    },
    Virtual {
        stream: Stream,
        pos: u64,
        state: State,
        ended: bool,
    },
}

pub struct Player {
    path: PathBuf,
    rate: u32,
    history: Arc<Mutex<History>>,
    clock: Option<Clock>,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Player {
    #[must_use]
    pub fn new(path: &Path, rate: u32) -> Self {
        Self {
            path: path.to_owned(),
            rate,
            history: Arc::new(Mutex::new(History::new(rate as usize * 4))),
            clock: None,
        }
    }

    /// Starts (or resumes) playing.
    ///
    /// # Errors
    ///
    /// When the file or the device cannot be opened.
    pub fn play(&mut self) -> Result<(), String> {
        match &mut self.clock {
            Some(Clock::Device { cmds, .. }) => cmds
                .send(Cmd::Play)
                .map_err(|_| "the player stopped".into()),
            Some(Clock::Virtual { state, ended, .. }) => {
                if !*ended {
                    *state = State::Playing;
                }
                Ok(())
            }
            None if virtual_audio() => {
                self.clock = Some(Clock::Virtual {
                    stream: Stream::open(&self.path)?,
                    pos: 0,
                    state: State::Playing,
                    ended: false,
                });
                Ok(())
            }
            None => {
                self.clock = Some(self.start_device()?);
                Ok(())
            }
        }
    }

    fn start_device(&self) -> Result<Clock, String> {
        let (cmds, rx) = mpsc::channel();
        let (ready, started) = mpsc::channel();
        let shared = Arc::new(Mutex::new(Shared::default()));
        let (path, history, sh) = (self.path.clone(), self.history.clone(), shared.clone());
        std::thread::spawn(move || device_thread(&path, &history, &sh, &rx, &ready));
        let (played, device_rate) = started
            .recv()
            .map_err(|_| "the player failed to start".to_string())??;
        Ok(Clock::Device {
            cmds,
            played,
            device_rate,
            shared,
        })
    }

    pub fn pause(&mut self) {
        match &mut self.clock {
            Some(Clock::Device { cmds, .. }) => {
                let _ = cmds.send(Cmd::Pause);
            }
            Some(Clock::Virtual { state, .. }) if *state == State::Playing => {
                *state = State::Paused
            }
            _ => {}
        }
    }

    /// Moves playback to `seconds`; the second it landed on.
    ///
    /// # Errors
    ///
    /// When nothing plays yet, or the source cannot seek there.
    pub fn seek(&mut self, seconds: f64) -> Result<f64, String> {
        let frame = match &mut self.clock {
            Some(Clock::Device { cmds, .. }) => {
                let (tx, rx) = mpsc::channel();
                cmds.send(Cmd::Seek(seconds, tx))
                    .map_err(|_| "the player stopped".to_string())?;
                rx.recv().map_err(|_| "the player stopped".to_string())??
            }
            Some(Clock::Virtual {
                stream, pos, ended, ..
            }) => {
                let f = stream.seek(seconds)?;
                *pos = f;
                *ended = false;
                lock(&self.history).reset(f);
                f
            }
            None => return Err("play first, then seek".into()),
        };
        Ok(frame as f64 / f64::from(self.rate))
    }

    /// The frame playing now.
    #[must_use]
    pub fn position(&self) -> u64 {
        match &self.clock {
            Some(Clock::Device {
                played,
                device_rate,
                shared,
                ..
            }) => {
                let s = lock(shared);
                let dev = played
                    .load(Ordering::Relaxed)
                    .saturating_sub(s.played_at_seek);
                s.seek_frame + dev * u64::from(self.rate) / u64::from((*device_rate).max(1))
            }
            Some(Clock::Virtual { pos, .. }) => *pos,
            None => 0,
        }
    }

    #[must_use]
    pub fn state(&self) -> State {
        match &self.clock {
            Some(Clock::Device { shared, .. }) => {
                let s = lock(shared);
                if s.ended {
                    State::Stopped
                } else {
                    s.state.unwrap_or(State::Stopped)
                }
            }
            Some(Clock::Virtual { state, ended, .. }) => {
                if *ended {
                    State::Stopped
                } else {
                    *state
                }
            }
            None => State::Stopped,
        }
    }

    /// The `n` frames ending at what plays now. A virtual player first
    /// advances one 60th of a second (when playing).
    ///
    /// # Errors
    ///
    /// A decoding failure.
    pub fn window(&mut self, n: usize) -> Result<(Vec<f64>, Vec<f64>), String> {
        if let Some(Clock::Virtual {
            stream,
            pos,
            state,
            ended,
        }) = &mut self.clock
        {
            if *state == State::Playing && !*ended {
                *pos += u64::from(self.rate) / 60;
                let mut h = lock(&self.history);
                while h.end() < *pos {
                    let c = stream.next(4096)?;
                    if c.left.is_empty() {
                        *ended = true;
                        break;
                    }
                    h.push(&c);
                }
            }
        }
        let end = self.position();
        Ok(lock(&self.history).window(end, n))
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        if let Some(Clock::Device { cmds, .. }) = &self.clock {
            let _ = cmds.send(Cmd::Stop);
        }
    }
}

type Started = Result<(Arc<AtomicU64>, u32), String>;

/// Owns the decoder and the device (the device stream stays on this
/// thread): fills the device's queue about a quarter second ahead.
fn device_thread(
    path: &Path,
    history: &Mutex<History>,
    shared: &Mutex<Shared>,
    cmds: &Receiver<Cmd>,
    ready: &Sender<Started>,
) {
    let opened = Stream::open(path).and_then(|s| Output::open(s.rate()).map(|o| (s, o)));
    let (mut stream, out) = match opened {
        Ok(x) => x,
        Err(e) => {
            let _ = ready.send(Err(e));
            return;
        }
    };
    if ready.send(Ok((out.played.clone(), out.rate))).is_err() {
        return;
    }
    let _ = out.play();
    lock(shared).state = Some(State::Playing);
    let mut exhausted = false;
    loop {
        while let Ok(cmd) = cmds.try_recv() {
            match cmd {
                Cmd::Play => {
                    let _ = out.play();
                    lock(shared).state = Some(State::Playing);
                }
                Cmd::Pause => {
                    let _ = out.pause();
                    lock(shared).state = Some(State::Paused);
                }
                Cmd::Seek(t, reply) => {
                    let r = stream.seek(t);
                    if let Ok(f) = r {
                        lock(&out.queue).clear();
                        lock(history).reset(f);
                        let mut s = lock(shared);
                        s.seek_frame = f;
                        s.played_at_seek = out.played.load(Ordering::Relaxed);
                        s.ended = false;
                        exhausted = false;
                    }
                    let _ = reply.send(r);
                }
                Cmd::Stop => return,
            }
        }
        if lock(shared).state == Some(State::Paused) {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let queued = lock(&out.queue).frames();
        if !exhausted && queued < out.rate as usize / 4 {
            match stream.next(2048) {
                Ok(c) if !c.left.is_empty() => {
                    lock(&out.queue).push(&c.left, &c.right, stream.rate(), out.rate);
                    lock(history).push(&c);
                }
                Ok(_) => exhausted = true,
                Err(e) => {
                    eprintln!("audio: {e}");
                    exhausted = true;
                }
            }
        } else {
            if exhausted && queued == 0 {
                lock(shared).ended = true;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}
