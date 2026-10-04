//! Bounded, incremental decoding of Ogg Vorbis, MP3 and WAV to stereo
//! PCM. Adapted from sw-ml-study/demo-extensions'
//! mlpl-native3d-window/src/audio.rs (`PcmStream`; same author, MIT).

use std::path::Path;

use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{Decoder, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::Time;

/// Up to this many frames per chunk.
pub const MAX_CHUNK: usize = 1 << 16;

/// Stereo PCM frames (a mono source is copied to both channels),
/// starting at frame `start` of the source.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chunk {
    pub left: Vec<f32>,
    pub right: Vec<f32>,
    pub start: u64,
}

pub struct Stream {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,
    rate: u32,
    channels: usize,
    frames: Option<u64>,
    left: Vec<f32>,
    right: Vec<f32>,
    offset: usize,
    emitted: u64,
}

impl Stream {
    /// Opens a file without reading all of it.
    ///
    /// # Errors
    ///
    /// When the file cannot be read, its format is not supported, or its
    /// track has no sample rate.
    pub fn open(path: &Path) -> Result<Self, String> {
        let file = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mss = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| format!("{}: {e}", path.display()))?;
        let track = probed
            .format
            .default_track()
            .ok_or_else(|| format!("{}: no audio track", path.display()))?;
        let rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| format!("{}: no sample rate", path.display()))?;
        let channels = track.codec_params.channels.map_or(2, |c| c.count());
        let frames = track.codec_params.n_frames;
        let id = track.id;
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| format!("{}: {e}", path.display()))?;
        Ok(Self {
            format: probed.format,
            decoder,
            track: id,
            rate,
            channels,
            frames,
            left: Vec::new(),
            right: Vec::new(),
            offset: 0,
            emitted: 0,
        })
    }

    #[must_use]
    pub const fn rate(&self) -> u32 {
        self.rate
    }

    #[must_use]
    pub const fn channels(&self) -> usize {
        self.channels
    }

    /// The source's length in frames, when its container says.
    #[must_use]
    pub const fn frames(&self) -> Option<u64> {
        self.frames
    }

    /// The next frames, at most `n` (and `MAX_CHUNK`); empty at the end.
    ///
    /// # Errors
    ///
    /// A malformed packet or a decoder failure.
    pub fn next(&mut self, n: usize) -> Result<Chunk, String> {
        let n = n.clamp(1, MAX_CHUNK);
        let mut out = Chunk {
            start: self.emitted,
            ..Chunk::default()
        };
        while out.left.len() < n {
            if self.offset >= self.left.len() && !self.decode_packet()? {
                break;
            }
            let take = (n - out.left.len()).min(self.left.len() - self.offset);
            out.left
                .extend_from_slice(&self.left[self.offset..self.offset + take]);
            out.right
                .extend_from_slice(&self.right[self.offset..self.offset + take]);
            self.offset += take;
        }
        self.emitted += out.left.len() as u64;
        Ok(out)
    }

    /// Seeks to `seconds`; the frame it landed on.
    ///
    /// # Errors
    ///
    /// When the time is negative or not finite, or the source cannot seek.
    pub fn seek(&mut self, seconds: f64) -> Result<u64, String> {
        if !seconds.is_finite() || seconds < 0.0 {
            return Err("a seek time is a nonnegative number of seconds".into());
        }
        let seeked = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time: Time::from(seconds),
                    track_id: Some(self.track),
                },
            )
            .map_err(|e| e.to_string())?;
        self.decoder.reset();
        self.left.clear();
        self.right.clear();
        self.offset = 0;
        self.emitted = seeked.actual_ts;
        Ok(self.emitted)
    }

    fn decode_packet(&mut self) -> Result<bool, String> {
        loop {
            let packet = match self.format.next_packet() {
                Ok(p) => p,
                Err(SymphoniaError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(SymphoniaError::IoError(e))
                    if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(false);
                }
                Err(e) => return Err(e.to_string()),
            };
            if packet.track_id() != self.track {
                continue;
            }
            let decoded = match self.decoder.decode(&packet) {
                Ok(d) => d,
                Err(SymphoniaError::DecodeError(_)) => continue, // skip a bad packet
                Err(e) => return Err(e.to_string()),
            };
            let channels = decoded.spec().channels.count();
            if channels == 0 {
                continue;
            }
            let mut samples = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            samples.copy_interleaved_ref(decoded);
            self.left.clear();
            self.right.clear();
            for frame in samples.samples().chunks(channels) {
                self.left.push(frame[0]);
                self.right
                    .push(if channels == 1 { frame[0] } else { frame[1] });
            }
            self.offset = 0;
            if !self.left.is_empty() {
                return Ok(true);
            }
        }
    }
}
