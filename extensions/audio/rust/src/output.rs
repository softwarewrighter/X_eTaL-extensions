//! The sound device: a bounded stereo queue the device drains, counting
//! the frames it actually played. Adapted from sw-ml-study/
//! demo-extensions' `PcmOutput` and `PlaybackBuffer` (same author, MIT).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// Interleaved stereo samples waiting for the device, at the device's
/// rate, bounded.
pub struct Queue {
    samples: VecDeque<f32>,
    capacity: usize,
}

impl Queue {
    #[must_use]
    pub fn new(frames: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(frames * 2),
            capacity: frames * 2,
        }
    }

    /// Adds frames at `from` Hz, resampled (linearly) to `to` Hz.
    pub fn push(&mut self, left: &[f32], right: &[f32], from: u32, to: u32) {
        if left.is_empty() || left.len() != right.len() || from == 0 || to == 0 {
            return;
        }
        let n = left.len();
        let out = n * to as usize / from as usize;
        for i in 0..out {
            let t = i as f64 * f64::from(from) / f64::from(to);
            let lo = (t.floor() as usize).min(n - 1);
            let hi = (lo + 1).min(n - 1);
            let f = (t - t.floor()) as f32;
            self.samples.push_back(left[lo] * (1.0 - f) + left[hi] * f);
            self.samples
                .push_back(right[lo] * (1.0 - f) + right[hi] * f);
        }
        while self.samples.len() > self.capacity {
            self.samples.pop_front();
        }
    }

    /// The next frame, if any.
    pub fn pop(&mut self) -> Option<[f32; 2]> {
        let l = self.samples.pop_front()?;
        let r = self.samples.pop_front().unwrap_or(0.0);
        Some([l, r])
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    /// Lets the queue hold at least `frames` (a voice queues a whole
    /// note at once).
    pub fn grow(&mut self, frames: usize) {
        self.capacity = self.capacity.max(frames * 2);
    }

    #[must_use]
    pub fn frames(&self) -> usize {
        self.samples.len() / 2
    }
}

/// The default output device playing from a queue.
pub struct Output {
    pub queue: Arc<Mutex<Queue>>,
    /// Frames the device has played (not silence for an empty queue).
    pub played: Arc<AtomicU64>,
    pub rate: u32,
    stream: cpal::Stream,
}

impl Output {
    /// Opens the default output, at `rate` when the device can.
    ///
    /// # Errors
    ///
    /// When there is no output device or no usable configuration.
    pub fn open(rate: u32) -> Result<Self, String> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or("no audio output device")?;
        let config = device
            .supported_output_configs()
            .map_err(|e| e.to_string())?
            .find(|r| {
                r.channels() >= 2
                    && r.min_sample_rate().0 <= rate
                    && r.max_sample_rate().0 >= rate
                    && r.sample_format() == cpal::SampleFormat::F32
            })
            .map(|r| r.with_sample_rate(cpal::SampleRate(rate)))
            .map_or_else(
                || device.default_output_config().map_err(|e| e.to_string()),
                Ok,
            )?;
        if config.sample_format() != cpal::SampleFormat::F32 {
            return Err(format!(
                "the output device wants {:?} samples; f32 is supported",
                config.sample_format()
            ));
        }
        let rate = config.sample_rate().0;
        let stream_config: cpal::StreamConfig = config.into();
        let channels = usize::from(stream_config.channels);
        let queue = Arc::new(Mutex::new(Queue::new(rate as usize)));
        let played = Arc::new(AtomicU64::new(0));
        let (q, p) = (queue.clone(), played.clone());
        let stream = device
            .build_output_stream(
                &stream_config,
                move |out: &mut [f32], _| {
                    let mut q = q.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    let mut n = 0;
                    for frame in out.chunks_mut(channels) {
                        let [l, r] = match q.pop() {
                            Some(s) => {
                                n += 1;
                                s
                            }
                            None => [0.0, 0.0],
                        };
                        for (c, s) in frame.iter_mut().enumerate() {
                            *s = if c == 0 { l } else { r };
                        }
                    }
                    p.fetch_add(n, Ordering::Relaxed);
                },
                |e| eprintln!("audio output: {e}"),
                None,
            )
            .map_err(|e| e.to_string())?;
        Ok(Self {
            queue,
            played,
            rate,
            stream,
        })
    }

    /// Starts or resumes the device.
    ///
    /// # Errors
    ///
    /// The device's error.
    pub fn play(&self) -> Result<(), String> {
        self.stream.play().map_err(|e| e.to_string())
    }

    /// Pauses the device.
    ///
    /// # Errors
    ///
    /// The device's error.
    pub fn pause(&self) -> Result<(), String> {
        self.stream.pause().map_err(|e| e.to_string())
    }
}
