use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Cursor, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rodio::{Decoder, Source};
use tracing::warn;

use super::sound::{Backend, Clip, Playing};

pub(super) const RATE: u32 = 48_000;
pub(super) const GAME_FPS: u32 = 30;
const SAMPLES_PER_FRAME: usize = (RATE / GAME_FPS) as usize;
const CHANNELS: usize = 2;

#[derive(Clone)]
enum Heard {
    Start { voice: u32, samples: Arc<[f32]>, looped: bool, level: f32 },
    Level { voice: u32, level: f32 },
    Stop { voice: u32 },
}

#[derive(Default)]
pub struct SoundLog {
    pub frame: u32,
    heard: Vec<(u32, Heard)>,
    next: u32,
}

impl SoundLog {
    pub fn new() -> Self {
        Self::default()
    }

    fn push(&mut self, heard: Heard) {
        self.heard.push((self.frame, heard));
    }
}

pub type SharedLog = Rc<RefCell<SoundLog>>;

pub struct SoundTape {
    log: SharedLog,
    decoded: RefCell<BTreeMap<String, Arc<[f32]>>>,
}

impl SoundTape {
    pub fn new(log: SharedLog) -> Self {
        Self { log, decoded: RefCell::new(BTreeMap::new()) }
    }

    fn samples(&self, clip: &Clip) -> Option<Arc<[f32]>> {
        let name = match clip {
            Clip::Effect { name, .. } | Clip::Track { name, .. } => name,
        };

        if let Some(held) = self.decoded.borrow().get(name) {
            return Some(Arc::clone(held));
        }

        let decoded: Arc<[f32]> = match clip {
            Clip::Effect { effect, .. } => Arc::from(to_stereo(&effect.samples, usize::from(effect.channels), effect.rate)),
            Clip::Track { bytes, .. } => {
                let decoder = Decoder::new(Cursor::new(Arc::clone(bytes)))
                    .inspect_err(|error| warn!("emu: {name} could not be decoded for the video: {error}"))
                    .ok()?;
                let channels = usize::from(decoder.channels().get());
                let rate = decoder.sample_rate().get();
                let samples: Vec<f32> = decoder.collect();

                Arc::from(to_stereo(&samples, channels, rate))
            }
        };

        self.decoded.borrow_mut().insert(name.clone(), Arc::clone(&decoded));

        Some(decoded)
    }
}

struct Taped {
    log: SharedLog,
    voice: u32,
    started: u32,
    frames: u32,
    looped: bool,
}

impl Playing for Taped {
    fn set_level(&self, level: f32) {
        self.log.borrow_mut().push(Heard::Level { voice: self.voice, level });
    }

    fn stop(&self) {
        self.log.borrow_mut().push(Heard::Stop { voice: self.voice });
    }

    fn busy(&self) -> bool {
        self.looped || self.log.borrow().frame.saturating_sub(self.started) < self.frames
    }
}

impl Backend for SoundTape {
    fn start(&self, clip: &Clip, looped: bool, level: f32) -> Option<Box<dyn Playing>> {
        let samples = self.samples(clip)?;
        let frames = samples.len().div_ceil(SAMPLES_PER_FRAME * CHANNELS) as u32;
        let mut log = self.log.borrow_mut();
        let voice = log.next;

        log.next += 1;
        log.push(Heard::Start { voice, samples, looped, level });

        Some(Box::new(Taped { log: Rc::clone(&self.log), voice, started: log.frame, frames, looped }))
    }
}

fn to_stereo(samples: &[f32], channels: usize, rate: u32) -> Vec<f32> {
    if channels == 0 || rate == 0 {
        return Vec::new();
    }

    let frames = samples.len() / channels;
    let length = (frames as u64 * u64::from(RATE) / u64::from(rate)) as usize;
    let mut out = Vec::with_capacity(length * CHANNELS);

    for index in 0..length {
        let position = index as f64 * f64::from(rate) / f64::from(RATE);
        let first = position.floor() as usize;
        let next = (first + 1).min(frames.saturating_sub(1));
        let weight = (position - first as f64) as f32;

        for channel in 0..CHANNELS {
            let source = channel.min(channels - 1);
            let at = |frame: usize| samples.get(frame * channels + source).copied().unwrap_or(0.0);

            out.push(at(first) * (1.0 - weight) + at(next) * weight);
        }
    }

    out
}

struct Voice {
    samples: Arc<[f32]>,
    at: usize,
    level: f32,
    looped: bool,
}

pub fn mix(log: &SoundLog, frames: u32, out: &Path, emit: &dyn Fn(f32), abort: &AtomicBool) -> io::Result<()> {
    let mut voices: BTreeMap<u32, Voice> = BTreeMap::new();
    let mut heard = log.heard.iter().peekable();
    let mut file = BufWriter::new(File::create(out)?);

    file.write_all(&header(frames as usize * SAMPLES_PER_FRAME * CHANNELS))?;

    let mut block = [0.0f32; SAMPLES_PER_FRAME * CHANNELS];
    let mut bytes: Vec<u8> = Vec::with_capacity(block.len() * 2);

    for frame in 0..frames {
        if abort.load(Ordering::Relaxed) {
            return Ok(());
        }

        if frame % GAME_FPS == 0 {
            emit(frame as f32 / frames.max(1) as f32);
        }

        while let Some((_, event)) = heard.next_if(|(at, _)| *at <= frame) {
            match event {
                Heard::Start { voice, samples, looped, level } => {
                    voices.insert(*voice, Voice { samples: Arc::clone(samples), at: 0, level: *level, looped: *looped });
                }
                Heard::Level { voice, level } => {
                    if let Some(held) = voices.get_mut(voice) {
                        held.level = *level;
                    }
                }
                Heard::Stop { voice } => {
                    voices.remove(voice);
                }
            }
        }

        block.fill(0.0);

        voices.retain(|_, voice| {
            if voice.samples.is_empty() {
                return false;
            }

            let mut filled = 0;

            while filled < block.len() {
                let source = voice.samples.get(voice.at..).unwrap_or_default();
                let taken = source.len().min(block.len() - filled);

                for (slot, sample) in block[filled..filled + taken].iter_mut().zip(source) {
                    *slot += sample * voice.level;
                }

                filled += taken;
                voice.at += taken;

                if voice.at < voice.samples.len() {
                    continue;
                }

                if !voice.looped {
                    return false;
                }

                voice.at = 0;
            }

            true
        });

        bytes.clear();

        for sample in &block {
            bytes.extend_from_slice(&((sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16).to_le_bytes());
        }

        file.write_all(&bytes)?;
    }

    file.flush()
}

fn header(samples: usize) -> Vec<u8> {
    let data = (samples * 2) as u32;
    let mut bytes = Vec::with_capacity(44);

    bytes.extend_from_slice(b"RIFF");
    bytes.extend_from_slice(&(36 + data).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt ");
    bytes.extend_from_slice(&16u32.to_le_bytes());
    bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&(CHANNELS as u16).to_le_bytes());
    bytes.extend_from_slice(&RATE.to_le_bytes());
    bytes.extend_from_slice(&(RATE * CHANNELS as u32 * 2).to_le_bytes());
    bytes.extend_from_slice(&((CHANNELS * 2) as u16).to_le_bytes());
    bytes.extend_from_slice(&16u16.to_le_bytes());
    bytes.extend_from_slice(b"data");
    bytes.extend_from_slice(&data.to_le_bytes());

    bytes
}
