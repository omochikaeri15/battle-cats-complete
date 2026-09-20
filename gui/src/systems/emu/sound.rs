use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Cursor;
use std::num::NonZero;
use std::rc::Rc;
use std::sync::Arc;

use emu::engine::SoundManager;
use rodio::buffer::SamplesBuffer;
use rodio::mixer::Mixer;
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player, Source};
use tracing::warn;

use super::assets::FileIndex;
#[cfg(target_os = "linux")]
use super::server_audio::ServerLink;

const FULL: i32 = 100;
const EVERY_TRACK: i32 = -1;
const CAF_HEADER: usize = 8;
const CHUNK_HEADER: usize = 12;
const DESCRIPTION: &[u8; 4] = b"desc";
const SAMPLES: &[u8; 4] = b"data";
const LITTLE_ENDIAN: u32 = 2;
const EDIT_COUNT: usize = 4;

#[derive(Clone)]
struct Effect {
    channels: u16,
    rate: u32,
    samples: Arc<[f32]>,
}

pub struct Volumes {
    pub music: i32,
    pub effects: i32,
}

pub type SharedVolumes = Rc<RefCell<Volumes>>;

enum Output {
    #[cfg(target_os = "linux")]
    Server(ServerLink),
    Device(MixerDeviceSink),
}

impl Output {
    fn open() -> Option<Self> {
        #[cfg(target_os = "linux")]
        if let Some(link) = ServerLink::open() {
            return Some(Self::Server(link));
        }

        DeviceSinkBuilder::open_default_sink()
            .inspect_err(|error| warn!("emu: no audio output: {error}"))
            .ok()
            .map(Self::Device)
    }

    fn mixer(&self) -> &Mixer {
        match self {
            #[cfg(target_os = "linux")]
            Self::Server(link) => link.mixer(),
            Self::Device(sink) => sink.mixer(),
        }
    }
}

pub struct Speaker {
    files: Rc<RefCell<FileIndex>>,
    volumes: SharedVolumes,
    stream: Option<Output>,
    music: Option<(i32, Player)>,
    effects: BTreeMap<i32, Effect>,
    duck: i32,
}

impl Speaker {
    pub fn new(files: Rc<RefCell<FileIndex>>, volumes: SharedVolumes) -> Self {
        let stream = Output::open();

        Self {
            files,
            volumes,
            stream,
            music: None,
            effects: BTreeMap::new(),
            duck: FULL,
        }
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.files.borrow().get(name).cloned()?;

        std::fs::read(path)
            .inspect_err(|error| warn!("emu: {name} could not be read: {error}"))
            .ok()
    }

    fn music_level(&self) -> f32 {
        (self.duck * self.volumes.borrow().music) as f32 / (FULL * FULL) as f32
    }

    fn effect(&mut self, sound_id: i32) -> Option<Effect> {
        if let Some(effect) = self.effects.get(&sound_id) {
            return Some(effect.clone());
        }

        let bytes = self.read(&format!("{sound_id:03}.caf"))?;
        let effect = parse_caf(&bytes)?;

        self.effects.insert(sound_id, effect.clone());

        Some(effect)
    }

    fn start_music(&mut self, sound_id: i32) {
        if let Some((held, sink)) = &self.music
            && *held == sound_id
        {
            sink.play();

            return;
        }

        let Some(stream) = &self.stream else {
            return;
        };
        let Some(bytes) = self.read(&format!("{sound_id:03}.ogg")) else {
            return;
        };
        let decoded = match Decoder::new_looped(Cursor::new(bytes)) {
            Ok(decoded) => decoded,
            Err(error) => {
                warn!("emu: track {sound_id} could not be decoded: {error}");

                return;
            }
        };
        let sink = Player::connect_new(stream.mixer());

        sink.set_volume(self.music_level());
        sink.append(decoded);
        self.music = Some((sound_id, sink));
    }

    pub fn refresh(&mut self) {
        let level = self.music_level();

        if let Some((_, sink)) = &self.music {
            sink.set_volume(level);
        }
    }
}

fn parse_caf(bytes: &[u8]) -> Option<Effect> {
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
                .chunks_exact(2)
                .map(|pair| {
                    let word = [pair[0], pair[1]];
                    let sample = if flags & LITTLE_ENDIAN != 0 {
                        i16::from_le_bytes(word)
                    } else {
                        i16::from_be_bytes(word)
                    };

                    f32::from(sample) / f32::from(i16::MAX)
                })
                .collect();

            return Some(Effect {
                channels: channels as u16,
                rate,
                samples: Arc::from(samples),
            });
        }

        at = body.checked_add(usize::try_from(size).ok()?)?;
    }

    None
}

impl SoundManager for Speaker {
    fn play_audio(&mut self, sound_id: i32, volume: Option<i32>, is_bgm: bool) {
        if is_bgm || self.files.borrow().contains_key(format!("{sound_id:03}.ogg").as_str()) {
            self.start_music(sound_id);

            return;
        }

        let Some(effect) = self.effect(sound_id) else {
            return;
        };
        let Some(stream) = &self.stream else {
            return;
        };
        let level = (volume.unwrap_or(FULL) * self.volumes.borrow().effects) as f32 / (FULL * FULL) as f32;
        let (Some(channels), Some(rate)) = (NonZero::new(effect.channels), NonZero::new(effect.rate)) else {
            return;
        };
        let source = SamplesBuffer::new(channels, rate, effect.samples.to_vec());

        stream.mixer().add(source.amplify(level));
    }

    fn stop_audio(&mut self, sound_id: i32) {
        if sound_id == EVERY_TRACK || self.music.as_ref().is_some_and(|(held, _)| *held == sound_id) {
            self.music = None;
        }
    }

    fn set_bgm_duck(&mut self, percent: i32) {
        self.duck = percent;
        self.refresh();
    }

    fn pause_all(&mut self) {
        if let Some((_, sink)) = &self.music {
            sink.pause();
        }
    }

    fn set_channel(&mut self, _channel: i32, _value: i32) {}
}
