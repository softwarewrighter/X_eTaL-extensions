//! Sound out of arrays: a voice plays the samples a program queues --
//! on the device (a thread owns the output stream), or, with no device
//! (`XETAL_AUDIO=off` or `XETAL_HEADLESS=1`), written to the WAV file
//! `XETAL_AUDIO_WAV` names (tests and recordings).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::output::{Output, Queue};
use crate::player::virtual_audio;

enum Sink {
    Device {
        queue: Arc<Mutex<Queue>>,
        played: Arc<AtomicU64>,
        rate: u32,
        stop: Sender<()>,
    },
    Wav {
        path: Option<std::path::PathBuf>,
        samples: Vec<f32>,
        /// The virtual playhead: a 60th of a second per `tick`.
        clock: u64,
    },
}

pub struct Voice {
    rate: u32,
    queued: u64,
    sink: Sink,
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl Voice {
    /// A voice at `rate` frames a second.
    ///
    /// # Errors
    ///
    /// When the rate is not 8,000 to 192,000, or the device cannot open.
    pub fn open(rate: u32) -> Result<Self, String> {
        if !(8_000..=192_000).contains(&rate) {
            return Err(format!("rate {rate}: give 8000 to 192000 frames a second"));
        }
        let sink = if virtual_audio() {
            Sink::Wav {
                path: std::env::var_os("XETAL_AUDIO_WAV")
                    .filter(|v| !v.is_empty())
                    .map(Into::into),
                samples: Vec::new(),
                clock: 0,
            }
        } else {
            let (ready, opened) = mpsc::channel();
            let (stop, stopped) = mpsc::channel::<()>();
            std::thread::spawn(move || match Output::open(rate) {
                Ok(out) => {
                    let _ = ready.send(Ok((out.queue.clone(), out.played.clone(), out.rate)));
                    let _ = out.play();
                    let _ = stopped.recv(); // the stream lives until then
                }
                Err(e) => {
                    let _ = ready.send(Err(e));
                }
            });
            let (queue, played, device_rate) = opened
                .recv()
                .map_err(|_| "the voice failed to start".to_string())??;
            Sink::Device {
                queue,
                played,
                rate: device_rate,
                stop,
            }
        };
        Ok(Self {
            rate,
            queued: 0,
            sink,
        })
    }

    /// Queues stereo frames (-1 to 1); the seconds now queued ahead.
    pub fn queue(&mut self, left: &[f32], right: &[f32]) -> f64 {
        self.queued += left.len() as u64;
        match &mut self.sink {
            Sink::Device { queue, rate, .. } => {
                let mut q = lock(queue);
                let need = q.frames() + left.len() * (*rate as usize) / (self.rate as usize) + 16;
                q.grow(need);
                q.push(left, right, self.rate, *rate);
            }
            Sink::Wav { samples, .. } => {
                for (l, r) in left.iter().zip(right) {
                    samples.push(l.clamp(-1.0, 1.0));
                    samples.push(r.clamp(-1.0, 1.0));
                }
            }
        }
        self.ahead()
    }

    /// Seconds queued and not yet played (0 with no device).
    #[must_use]
    pub fn ahead(&self) -> f64 {
        match &self.sink {
            Sink::Device { queue, rate, .. } => lock(queue).frames() as f64 / f64::from(*rate),
            Sink::Wav { .. } => 0.0,
        }
    }

    /// Waits until at most `seconds` are queued.
    pub fn wait(&self, seconds: f64) {
        while self.ahead() > seconds.max(0.0) {
            std::thread::sleep(Duration::from_millis(5));
        }
    }

    #[must_use]
    pub const fn rate(&self) -> u32 {
        self.rate
    }

    /// Frames played so far: counted by the device, or, with no device,
    /// everything queued.
    #[must_use]
    pub fn played(&self) -> u64 {
        match &self.sink {
            Sink::Device { played, rate, .. } => {
                played.load(Ordering::Relaxed) * u64::from(self.rate) / u64::from((*rate).max(1))
            }
            Sink::Wav { .. } => self.queued,
        }
    }

    /// The playhead, for a program drawing what plays: the device's
    /// count, or, with no device, a virtual clock that advances a 60th
    /// of a second each time it is read (not past what is queued), so
    /// a headless program's frames see exactly the same sound each run.
    pub fn tick(&mut self) -> u64 {
        let rate = u64::from(self.rate);
        let queued = self.queued;
        match &mut self.sink {
            Sink::Wav { clock, .. } => {
                *clock = (*clock + rate / 60).min(queued);
                *clock
            }
            Sink::Device { .. } => self.played(),
        }
    }

    /// Finishes: the device plays out what is queued; a WAV is written.
    ///
    /// # Errors
    ///
    /// When the WAV cannot be written.
    pub fn finish(self) -> Result<(), String> {
        match self.sink {
            Sink::Device { queue, stop, .. } => {
                while lock(&queue).frames() > 0 {
                    std::thread::sleep(Duration::from_millis(5));
                }
                std::thread::sleep(Duration::from_millis(50));
                let _ = stop.send(());
                Ok(())
            }
            Sink::Wav {
                path: Some(p),
                samples,
                ..
            } => write_wav(&p, self.rate, &samples),
            Sink::Wav { path: None, .. } => Ok(()),
        }
    }
}

/// A 16-bit stereo PCM WAV file.
///
/// # Errors
///
/// When the file cannot be written.
#[allow(clippy::cast_possible_truncation)]
pub fn write_wav(path: &std::path::Path, rate: u32, interleaved: &[f32]) -> Result<(), String> {
    let data_len = (interleaved.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&2u16.to_le_bytes()); // stereo
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 4).to_le_bytes());
    b.extend_from_slice(&4u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for s in interleaved {
        b.extend_from_slice(&((s * 32767.0).round() as i16).to_le_bytes());
    }
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    std::fs::write(path, b).map_err(|e| format!("{}: {e}", path.display()))
}
