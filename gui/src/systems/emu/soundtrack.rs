use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Cursor, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use emu::engine::SoundManager;
use rodio::{Decoder, Source};
use tracing::warn;

use super::assets::FileIndex;
use super::sound::{effect_names, parse_caf, track_names, EVERYTHING, EVERY_EFFECT, EVERY_TRACK};

pub(super) const RATE: u32 = 48_000;
const FULL: i32 = 100;
pub(super) const GAME_FPS: u32 = 30;
const SAMPLES_PER_FRAME: usize = (RATE / GAME_FPS) as usize;
const CHANNELS: usize = 2;

#[derive(Clone, Copy, Debug)]
enum Heard {
    Music { id: i32, duck: i32 },
    Effect { id: i32, volume: i32 },
    Stop(i32),
    Duck(i32),
    Pause,
    MusicVolume(i32),
    EffectVolume(i32),
}

pub struct SoundLog {
    pub frame: u32,
    heard: Vec<(u32, Heard)>,
    opening: (i32, i32),
    music: i32,
    effects: i32,
}

impl SoundLog {
    pub fn new(music: i32, effects: i32) -> Self {
        Self { frame: 0, heard: Vec::new(), opening: (music, effects), music, effects }
    }

    fn push(&mut self, heard: Heard) {
        self.heard.push((self.frame, heard));
    }
}

pub type SharedLog = Rc<RefCell<SoundLog>>;

pub struct Listener {
    log: SharedLog,
    files: Rc<RefCell<FileIndex>>,
}

impl Listener {
    pub fn new(log: SharedLog, files: Rc<RefCell<FileIndex>>) -> Self {
        Self { log, files }
    }

    fn is_track(&self, sound_id: i32) -> bool {
        let files = self.files.borrow();

        track_names(sound_id).iter().any(|name| files.contains_key(name.as_str()))
    }
}

impl SoundManager for Listener {
    fn play_audio(&mut self, sound_id: i32, volume: Option<i32>, is_bgm: bool) {
        let heard = if is_bgm || self.is_track(sound_id) {
            Heard::Music { id: sound_id, duck: volume.unwrap_or(FULL) }
        } else {
            Heard::Effect { id: sound_id, volume: volume.unwrap_or(FULL) }
        };

        self.log.borrow_mut().push(heard);
    }

    fn stop_audio(&mut self, sound_id: i32) {
        self.log.borrow_mut().push(Heard::Stop(sound_id));
    }

    fn set_bgm_duck(&mut self, percent: i32) {
        self.log.borrow_mut().push(Heard::Duck(percent));
    }

    fn pause_all(&mut self) {
        self.log.borrow_mut().push(Heard::Pause);
    }

    fn set_channel(&mut self, _channel: i32, _value: i32) {}

    fn get_bgm_volume_setting(&mut self) -> i32 {
        self.log.borrow().music
    }

    fn get_se_volume_setting(&mut self) -> i32 {
        self.log.borrow().effects
    }

    fn set_bgm_volume_setting(&mut self, percent: i32) {
        let mut log = self.log.borrow_mut();

        log.music = percent;
        log.push(Heard::MusicVolume(percent));
    }

    fn set_se_volume_setting(&mut self, percent: i32) {
        let mut log = self.log.borrow_mut();

        log.effects = percent;
        log.push(Heard::EffectVolume(percent));
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

struct Library<'a> {
    files: &'a FileIndex,
    tracks: BTreeMap<i32, Option<Arc<[f32]>>>,
    effects: BTreeMap<i32, Option<Arc<[f32]>>>,
}

impl Library<'_> {
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let source = self.files.get(name)?;

        source.read().map(|bytes| bytes.to_vec()).inspect_err(|error| warn!("emu: {name} could not be read for the video: {error}")).ok()
    }

    fn track(&mut self, sound_id: i32) -> Option<Arc<[f32]>> {
        if let Some(held) = self.tracks.get(&sound_id) {
            return held.clone();
        }

        let decoded = track_names(sound_id).iter().find_map(|name| self.read(name)).and_then(|bytes| {
            let decoder = Decoder::new(Cursor::new(bytes)).inspect_err(|error| warn!("emu: track {sound_id} could not be decoded: {error}")).ok()?;
            let channels = usize::from(decoder.channels().get());
            let rate = decoder.sample_rate().get();
            let samples: Vec<f32> = decoder.collect();

            Some(Arc::from(to_stereo(&samples, channels, rate)))
        });

        self.tracks.insert(sound_id, decoded.clone());
        decoded
    }

    fn effect(&mut self, sound_id: i32) -> Option<Arc<[f32]>> {
        if let Some(held) = self.effects.get(&sound_id) {
            return held.clone();
        }

        let decoded = effect_names(sound_id)
            .iter()
            .find_map(|name| self.read(name))
            .and_then(|bytes| parse_caf(&bytes))
            .map(|effect| Arc::from(to_stereo(&effect.samples, usize::from(effect.channels), effect.rate)));

        self.effects.insert(sound_id, decoded.clone());
        decoded
    }
}

struct Voice {
    samples: Arc<[f32]>,
    at: usize,
    level: f32,
}

pub fn mix(log: &SoundLog, files: &FileIndex, frames: u32, out: &Path, emit: &dyn Fn(f32), abort: &AtomicBool) -> io::Result<()> {
    let mut library = Library { files, tracks: BTreeMap::new(), effects: BTreeMap::new() };
    let mut song: Option<(i32, Voice)> = None;
    let mut voices: BTreeMap<i32, Voice> = BTreeMap::new();
    let ((mut music, mut effects), mut duck) = (log.opening, FULL);
    let mut heard = log.heard.iter().peekable();
    let song_level = |duck: i32, music: i32| (duck * music) as f32 / (FULL * FULL) as f32;
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
            match *event {
                Heard::Music { id, duck: level } => {
                    duck = level;
                    song = library.track(id).map(|samples| (id, Voice { samples, at: 0, level: song_level(duck, music) }));
                }
                Heard::Effect { id, volume } => {
                    if let Some(samples) = library.effect(id) {
                        voices.insert(id, Voice { samples, at: 0, level: (volume * effects) as f32 / (FULL * FULL) as f32 });
                    }
                }
                Heard::Stop(id) => {
                    if matches!(id, EVERY_TRACK | EVERYTHING) || song.as_ref().is_some_and(|(held, _)| *held == id) {
                        song = None;
                    }

                    if matches!(id, EVERY_EFFECT | EVERYTHING) {
                        voices.clear();
                    } else {
                        voices.remove(&id);
                    }
                }
                Heard::Duck(level) => duck = level,
                Heard::Pause => song = None,
                Heard::MusicVolume(level) => music = level,
                Heard::EffectVolume(level) => effects = level,
            }

            if let Some((_, voice)) = song.as_mut() {
                voice.level = song_level(duck, music);
            }
        }

        block.fill(0.0);

        if let Some((_, voice)) = song.as_mut()
            && !voice.samples.is_empty()
        {
            let mut filled = 0;

            while filled < block.len() {
                let source = voice.samples.get(voice.at..).unwrap_or_default();
                let taken = source.len().min(block.len() - filled);

                for (slot, sample) in block[filled..filled + taken].iter_mut().zip(source) {
                    *slot += sample * voice.level;
                }

                filled += taken;
                voice.at = (voice.at + taken) % voice.samples.len();
            }
        }

        voices.retain(|_, voice| {
            let source = voice.samples.get(voice.at..).unwrap_or_default();

            for (slot, sample) in block.iter_mut().zip(source) {
                *slot += sample * voice.level;
            }

            voice.at += block.len();
            voice.at < voice.samples.len()
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
