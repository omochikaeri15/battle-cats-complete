use std::any::Any;
use std::cell::{Cell, RefCell};
use std::rc::{Rc, Weak};
use std::sync::Arc;

use tracing::{debug, warn};

use emulator::engine::{AudioHost, AudioSource, SoundData};

use super::assets::{Files, SharedLedger};

const BUFFER_VOICES: i32 = 24;
const STREAM_VOICES: i32 = 2;
const BUFFER_PRIORITY: i32 = 0;
const STREAM_PRIORITY: i32 = -1;
const FILES_DIR: &[u8] = b"files";
const QUIETEST: f32 = 0.001;
const CAF_HEADER: usize = 8;
const CHUNK_HEADER: usize = 12;
const DESCRIPTION: &[u8; 4] = b"desc";
const SAMPLES: &[u8; 4] = b"data";
const LITTLE_ENDIAN: u32 = 2;
const EDIT_COUNT: usize = 4;

#[derive(Clone)]
pub struct Effect {
    pub channels: u16,
    pub rate: u32,
    pub samples: Arc<[f32]>,
}

struct EffectData {
    name: String,
    effect: Effect,
}

impl SoundData for EffectData {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub enum Clip {
    Effect { name: String, effect: Effect },
    Track { name: String, bytes: Arc<[u8]> },
}

pub trait Playing {
    fn set_level(&self, level: f32);
    fn stop(&self);
    fn busy(&self) -> bool;
}

pub trait Backend {
    fn start(&self, clip: &Clip, looped: bool, level: f32) -> Option<Box<dyn Playing>>;
}

struct Voice {
    clip: Clip,
    stream: bool,
    looping: Cell<bool>,
    level: Cell<f32>,
    playing: RefCell<Option<Box<dyn Playing>>>,
    backend: Rc<dyn Backend>,
    claims: Rc<Cell<i32>>,
}

impl AudioSource for Voice {
    fn is_playing(&self) -> bool {
        self.playing.borrow().as_ref().is_some_and(|playing| playing.busy())
    }

    fn is_stopped(&self) -> bool {
        !self.is_playing()
    }

    fn play(&self) {
        if self.stream && self.is_playing() {
            return;
        }

        self.stop();
        *self.playing.borrow_mut() = self.backend.start(&self.clip, self.looping.get(), self.level.get());
    }

    fn set_loop(&self, looping: bool) {
        self.looping.set(looping);
    }

    fn set_volume(&self, volume: f32) {
        let level = if volume > QUIETEST { volume.min(1.0) } else { 0.0 };

        self.level.set(level);

        if let Some(playing) = self.playing.borrow().as_ref() {
            playing.set_level(level);
        }
    }

    fn stop(&self) {
        if let Some(playing) = self.playing.borrow_mut().take() {
            playing.stop();
        }
    }

    fn priority(&self) -> i32 {
        if self.stream { STREAM_PRIORITY } else { BUFFER_PRIORITY }
    }
}

impl Drop for Voice {
    fn drop(&mut self) {
        self.stop();
        self.claims.set(self.claims.get() - 1);
    }
}

pub struct Pool {
    files: Rc<dyn Files>,
    ledger: SharedLedger,
    backend: Rc<dyn Backend>,
    buffers: Rc<Cell<i32>>,
    streams: Rc<Cell<i32>>,
    voices: Vec<Weak<Voice>>,
}

impl Pool {
    pub fn new(files: Rc<dyn Files>, ledger: SharedLedger, backend: Rc<dyn Backend>) -> Self {
        Self { files, ledger, backend, buffers: Rc::new(Cell::new(0)), streams: Rc::new(Cell::new(0)), voices: Vec::new() }
    }

    fn name(path: &[u8]) -> String {
        let base = path.rsplit(|byte| *byte == b'/').next().unwrap_or(path);

        String::from_utf8_lossy(base).into_owned()
    }

    fn read(&self, path: &[u8]) -> Option<(String, Vec<u8>)> {
        let name = Self::name(path);
        let bytes = self.files.read(&name)?;

        self.ledger.borrow_mut().note(&name, &name);

        Some((name, bytes))
    }

    fn claim(&mut self, clip: Clip, stream: bool) -> Option<Rc<dyn AudioSource>> {
        let (claims, limit) = if stream { (&self.streams, STREAM_VOICES) } else { (&self.buffers, BUFFER_VOICES) };

        if claims.get() >= limit {
            return None;
        }

        claims.set(claims.get() + 1);

        let voice = Rc::new(Voice {
            clip,
            stream,
            looping: Cell::new(false),
            level: Cell::new(1.0),
            playing: RefCell::new(None),
            backend: Rc::clone(&self.backend),
            claims: Rc::clone(claims),
        });

        self.voices.retain(|held| held.strong_count() > 0);
        self.voices.push(Rc::downgrade(&voice));

        Some(voice)
    }
}

impl AudioHost for Pool {
    fn active_sources(&mut self) -> i32 {
        self.voices.iter().filter_map(Weak::upgrade).filter(|voice| voice.is_playing()).count() as i32
    }

    fn source_capacity(&mut self) -> i32 {
        BUFFER_VOICES + STREAM_VOICES
    }

    fn new_buffer_source(&mut self, data: &Rc<dyn SoundData>) -> Option<Rc<dyn AudioSource>> {
        let data = data.as_any().downcast_ref::<EffectData>()?;
        let clip = Clip::Effect { name: data.name.clone(), effect: data.effect.clone() };

        self.claim(clip, false)
    }

    fn new_stream_source(&mut self, path: &[u8], _source_type: i32) -> Option<Rc<dyn AudioSource>> {
        let (name, bytes) = self.read(path)?;

        self.claim(Clip::Track { name, bytes: Arc::from(bytes) }, true)
    }

    fn file_exists(&mut self, path: &[u8]) -> bool {
        self.files.contains(&Self::name(path))
    }

    fn files_dir(&mut self) -> Vec<u8> {
        FILES_DIR.to_vec()
    }

    fn remove_file(&mut self, path: &[u8]) -> bool {
        debug!("emu: the game asked to delete {}, which is left in place", String::from_utf8_lossy(path));

        false
    }

    fn load_sound_data(&mut self, path: &[u8]) -> Option<Rc<dyn SoundData>> {
        let (name, bytes) = self.read(path)?;
        let Some(effect) = parse_caf(&bytes) else {
            warn!("emu: {name} is not a readable caf");

            return None;
        };

        Some(Rc::new(EffectData { name, effect }))
    }
}

pub fn parse_caf(bytes: &[u8]) -> Option<Effect> {
    let mut at = CAF_HEADER;
    let mut format: Option<(u32, u32, u32)> = None;

    while let Some(header) = bytes.get(at..at + CHUNK_HEADER) {
        let kind = header.get(..4)?;
        let size = i64::from_be_bytes(header.get(4..)?.try_into().ok()?);
        let body = at + CHUNK_HEADER;

        if kind == DESCRIPTION {
            let rate = f64::from_be_bytes(bytes.get(body..body + 8)?.try_into().ok()?);
            let flags = u32::from_be_bytes(bytes.get(body + 12..body + 16)?.try_into().ok()?);
            let channels = u32::from_be_bytes(bytes.get(body + 24..body + 28)?.try_into().ok()?);

            format = Some((rate as u32, flags, channels));
        }

        if kind == SAMPLES {
            let (rate, flags, channels) = format?;
            let end = if size < 0 { bytes.len() } else { (body + size as usize).min(bytes.len()) };
            let pcm = bytes.get(body + EDIT_COUNT..end)?;
            let samples: Vec<f32> = pcm
                .as_chunks::<2>()
                .0
                .iter()
                .map(|&word| {
                    let sample = if flags & LITTLE_ENDIAN != 0 { i16::from_le_bytes(word) } else { i16::from_be_bytes(word) };

                    f32::from(sample) / f32::from(i16::MAX)
                })
                .collect();

            return Some(Effect { channels: channels as u16, rate, samples: Arc::from(samples) });
        }

        at = body.checked_add(usize::try_from(size).ok()?)?;
    }

    None
}
