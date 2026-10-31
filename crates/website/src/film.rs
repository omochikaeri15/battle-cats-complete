use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::io::{self, Write};
use std::rc::Rc;

use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::StreamExt;
use js_sys::{Float32Array, Function, Object, Promise, Reflect, Uint8Array};
use muxide::api::{AacProfile, AudioCodec, MuxerBuilder, VideoCodec};
use tracing::{info, warn};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{AudioBuffer, AudioBufferSourceNode, GainNode, OfflineAudioContext};

use omote::platform::battle::{AudioCommand, BattleCommand, BattleEvent, Filmed, Filming, Rolled};

use crate::battle::Darkroom;

const BATCH: u32 = 30;
const KEY_EVERY: usize = 60;
const QUEUE_LIMIT: u32 = 8;
const BUSY_MS: f64 = 16.0;
const FPS: f64 = 30.0;
const MICROS: f64 = 1_000_000.0;
const RATE: u32 = 48_000;
const CHANNELS: u32 = 2;
const SAMPLES_PER_FRAME: u32 = RATE / 30;
const SLICE: usize = 48_000;
const AAC: &str = "mp4a.40.2";
const AAC_BITRATE: u32 = 192_000;
const OPUS: &str = "opus";
const OPUS_BITRATE: u32 = 160_000;
const QUALITY_MODE: u32 = 50;
const LOWEST_BPP: f64 = 0.002;
const HIGHEST_BPP: f64 = 0.15;
const PROFILES: [&str; 4] = ["42e0", "4200", "4d40", "6400"];
const REORDERED: &str = "this browser only encodes H.264 with reordered frames, which cannot be packaged with sound";
const LEVELS: [(u8, u32, u32); 8] = [
    (0x1f, 3_600, 108_000),
    (0x20, 5_120, 216_000),
    (0x28, 8_192, 245_760),
    (0x2a, 8_704, 522_240),
    (0x32, 22_080, 589_824),
    (0x33, 36_864, 983_040),
    (0x34, 36_864, 2_073_600),
    (0x3c, 139_264, 4_177_920),
];
pub(crate) const FILE_NAME: &str = "Replay.mp4";
pub(crate) const MIME: &str = "video/mp4";

mod codecs {
    use js_sys::{Object, Promise, Uint8Array};
    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;

    #[wasm_bindgen]
    extern "C" {
        pub type VideoEncoder;

        #[wasm_bindgen(constructor, catch)]
        pub fn new(init: &Object) -> Result<VideoEncoder, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn configure(this: &VideoEncoder, config: &Object) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn encode(this: &VideoEncoder, frame: &VideoFrame, options: &Object) -> Result<(), JsValue>;

        #[wasm_bindgen(method)]
        pub fn flush(this: &VideoEncoder) -> Promise;

        #[wasm_bindgen(method, catch)]
        pub fn close(this: &VideoEncoder) -> Result<(), JsValue>;

        #[wasm_bindgen(method, getter, js_name = encodeQueueSize)]
        pub fn encode_queue_size(this: &VideoEncoder) -> u32;

        #[wasm_bindgen(static_method_of = VideoEncoder, js_name = isConfigSupported)]
        pub fn is_config_supported(config: &Object) -> Promise;

        pub type VideoFrame;

        #[wasm_bindgen(constructor, catch)]
        pub fn new(source: &HtmlCanvasElement, init: &Object) -> Result<VideoFrame, JsValue>;

        #[wasm_bindgen(method)]
        pub fn close(this: &VideoFrame);

        pub type AudioEncoder;

        #[wasm_bindgen(constructor, catch)]
        pub fn new(init: &Object) -> Result<AudioEncoder, JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn configure(this: &AudioEncoder, config: &Object) -> Result<(), JsValue>;

        #[wasm_bindgen(method, catch)]
        pub fn encode(this: &AudioEncoder, data: &AudioData) -> Result<(), JsValue>;

        #[wasm_bindgen(method)]
        pub fn flush(this: &AudioEncoder) -> Promise;

        #[wasm_bindgen(method, catch)]
        pub fn close(this: &AudioEncoder) -> Result<(), JsValue>;

        #[wasm_bindgen(static_method_of = AudioEncoder, js_name = isConfigSupported)]
        pub fn is_config_supported(config: &Object) -> Promise;

        pub type AudioData;

        #[wasm_bindgen(constructor, catch)]
        pub fn new(init: &Object) -> Result<AudioData, JsValue>;

        #[wasm_bindgen(method)]
        pub fn close(this: &AudioData);

        pub type EncodedChunk;

        #[wasm_bindgen(method, getter, js_name = byteLength)]
        pub fn byte_length(this: &EncodedChunk) -> u32;

        #[wasm_bindgen(method, getter)]
        pub fn timestamp(this: &EncodedChunk) -> f64;

        #[wasm_bindgen(method, getter, js_name = "type")]
        pub fn kind(this: &EncodedChunk) -> String;

        #[wasm_bindgen(method, catch, js_name = copyTo)]
        pub fn copy_to(this: &EncodedChunk, destination: &Uint8Array) -> Result<(), JsValue>;
    }
}

use codecs::{AudioData, AudioEncoder, EncodedChunk, VideoEncoder, VideoFrame};

pub(crate) struct Wiring {
    pub(crate) send: Box<dyn Fn(BattleCommand)>,
    pub(crate) tell: Box<dyn Fn(BattleEvent)>,
    pub(crate) save: Box<dyn Fn(Vec<u8>)>,
}

struct Reel {
    rolled: UnboundedSender<Box<Rolled>>,
    abort: Rc<Cell<bool>>,
}

thread_local! {
    static REEL: RefCell<Option<Reel>> = const { RefCell::new(None) };
}

struct Chunk {
    at: f64,
    key: bool,
    data: Vec<u8>,
}

type Chunks = Rc<RefCell<Vec<Chunk>>>;
type Broken = Rc<RefCell<Option<String>>>;
type Output = Closure<dyn FnMut(JsValue, JsValue)>;
type Failure = Closure<dyn FnMut(JsValue)>;

#[derive(Clone, Default)]
struct Sink(Rc<RefCell<Vec<u8>>>);

impl Write for Sink {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buf);

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

enum Sound {
    Aac,
    Opus,
}

fn describe(err: JsValue) -> String {
    err.as_string().or_else(|| Reflect::get(&err, &JsValue::from_str("message")).ok().and_then(|message| message.as_string())).unwrap_or_else(|| format!("{err:?}"))
}

fn object(fields: &[(&str, JsValue)]) -> Object {
    let held = Object::new();

    for (key, value) in fields {
        let _ = Reflect::set(&held, &JsValue::from_str(key), value);
    }

    held
}

fn global_has(name: &str) -> bool {
    Reflect::has(&js_sys::global(), &JsValue::from_str(name)).unwrap_or(false)
}

pub(crate) fn supported() -> bool {
    ["VideoEncoder", "VideoFrame", "AudioEncoder", "AudioData", "OfflineAudioContext"].iter().all(|name| global_has(name))
}

fn clock() -> f64 {
    web_sys::window().and_then(|window| window.performance()).map_or(0.0, |performance| performance.now())
}

async fn pause(millis: i32) {
    let promise = Promise::new(&mut |resolve: Function, _reject: Function| {
        let scheduled = web_sys::window().map(|window| window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis));

        if !matches!(scheduled, Some(Ok(_))) {
            let _ = resolve.call0(&JsValue::NULL);
        }
    });

    let _ = JsFuture::from(promise).await;
}

async fn accepted(check: Promise) -> bool {
    JsFuture::from(check).await.ok().and_then(|answer| Reflect::get(&answer, &JsValue::from_str("supported")).ok()).is_some_and(|supported| supported.is_truthy())
}

fn collector(chunks: &Chunks, broken: &Broken) -> (Output, Failure) {
    let held = Rc::clone(chunks);
    let output = Closure::<dyn FnMut(JsValue, JsValue)>::new(move |chunk: JsValue, _meta: JsValue| {
        let chunk: EncodedChunk = chunk.unchecked_into();
        let bytes = Uint8Array::new_with_length(chunk.byte_length());

        if chunk.copy_to(&bytes).is_ok() {
            held.borrow_mut().push(Chunk { at: chunk.timestamp(), key: chunk.kind() == "key", data: bytes.to_vec() });
        }
    });
    let failed = Rc::clone(broken);
    let error = Closure::<dyn FnMut(JsValue)>::new(move |err: JsValue| {
        let reason = describe(err);

        warn!("A video encoder failed: {reason}");
        failed.borrow_mut().get_or_insert(reason);
    });

    (output, error)
}

fn bitrate(width: u32, height: u32, quality: u32) -> f64 {
    let share = f64::from(quality.clamp(1, 100)) / 100.0;
    let bpp = LOWEST_BPP * (HIGHEST_BPP / LOWEST_BPP).powf(share);

    f64::from(width) * f64::from(height) * FPS * bpp
}

fn level(width: u32, height: u32) -> u8 {
    let blocks = width.div_ceil(16) * height.div_ceil(16);
    let rate = blocks * FPS as u32;

    LEVELS.iter().find(|(_, frame, second)| blocks <= *frame && rate <= *second).map_or(0x3c, |(level, _, _)| *level)
}

async fn video_config(width: u32, height: u32, filming: &Filming) -> Result<Object, String> {
    let level = level(width, height);
    let latency = if filming.compression >= QUALITY_MODE { "quality" } else { "realtime" };

    for profile in PROFILES {
        let config = object(&[
            ("codec", JsValue::from_str(&format!("avc1.{profile}{level:02x}"))),
            ("width", JsValue::from(width)),
            ("height", JsValue::from(height)),
            ("bitrate", JsValue::from(bitrate(width, height, filming.quality))),
            ("bitrateMode", JsValue::from_str("variable")),
            ("framerate", JsValue::from(FPS)),
            ("latencyMode", JsValue::from_str(latency)),
            ("avc", object(&[("format", JsValue::from_str("annexb"))]).into()),
        ]);

        if accepted(VideoEncoder::is_config_supported(&config)).await {
            return Ok(config);
        }
    }

    Err("this browser cannot encode H.264 video".to_owned())
}

async fn audio_config() -> Result<(Object, Sound), String> {
    let aac = object(&[
        ("codec", JsValue::from_str(AAC)),
        ("sampleRate", JsValue::from(RATE)),
        ("numberOfChannels", JsValue::from(CHANNELS)),
        ("bitrate", JsValue::from(AAC_BITRATE)),
        ("aac", object(&[("format", JsValue::from_str("adts"))]).into()),
    ]);

    if accepted(AudioEncoder::is_config_supported(&aac)).await {
        return Ok((aac, Sound::Aac));
    }

    let opus = object(&[
        ("codec", JsValue::from_str(OPUS)),
        ("sampleRate", JsValue::from(RATE)),
        ("numberOfChannels", JsValue::from(CHANNELS)),
        ("bitrate", JsValue::from(OPUS_BITRATE)),
    ]);

    if accepted(AudioEncoder::is_config_supported(&opus)).await {
        return Ok((opus, Sound::Opus));
    }

    Err("this browser cannot encode AAC or Opus audio".to_owned())
}

pub(crate) fn start(filming: Filming, wiring: Wiring) {
    let (sender, receiver) = unbounded();
    let abort = Rc::new(Cell::new(false));

    REEL.with(|slot| {
        if let Some(previous) = slot.borrow_mut().replace(Reel { rolled: sender, abort: Rc::clone(&abort) }) {
            previous.abort.set(true);
        }
    });

    wasm_bindgen_futures::spawn_local(async move {
        let outcome = shoot(&filming, &wiring, receiver, &abort).await;

        (wiring.send)(BattleCommand::Unroll);

        let filmed = match outcome {
            Ok(Some(bytes)) => {
                info!(bytes = bytes.len(), "A replay video was exported");
                (wiring.save)(bytes);
                Filmed::Done
            }
            Ok(None) => Filmed::Aborted,
            Err(reason) => {
                warn!("A replay video could not be exported: {reason}");
                Filmed::Failed(reason)
            }
        };

        REEL.with(|slot| {
            if slot.borrow().as_ref().is_some_and(|reel| Rc::ptr_eq(&reel.abort, &abort)) {
                *slot.borrow_mut() = None;
            }
        });
        (wiring.tell)(BattleEvent::Filmed(filmed));
    });
}

pub(crate) fn abort() {
    REEL.with(|slot| {
        if let Some(reel) = slot.borrow().as_ref() {
            reel.abort.set(true);
        }
    });
}

pub(crate) fn deliver(rolled: Box<Rolled>) {
    REEL.with(|slot| {
        if let Some(reel) = slot.borrow().as_ref()
            && reel.rolled.unbounded_send(rolled).is_err()
        {
            warn!("A batch of replay video frames arrived after the export had ended");
        }
    });
}

async fn next(receiver: &mut UnboundedReceiver<Box<Rolled>>) -> Result<Box<Rolled>, String> {
    let rolled = receiver.next().await.ok_or("the data worker stopped sending the replay")?;

    match &rolled.failure {
        Some(reason) => Err(reason.clone()),
        None => Ok(rolled),
    }
}

async fn shoot(filming: &Filming, wiring: &Wiring, mut receiver: UnboundedReceiver<Box<Rolled>>, abort: &Cell<bool>) -> Result<Option<Vec<u8>>, String> {
    if !supported() {
        return Err("this browser cannot encode video".to_owned());
    }

    (wiring.tell)(BattleEvent::Rendering(0.0));
    (wiring.send)(BattleCommand::Film { source: filming.source, priority: filming.priority.clone() });

    let opened = next(&mut receiver).await?;
    let (width, height) = opened.size;
    let video_config = video_config(width, height, filming).await?;
    let (audio_config, sound) = audio_config().await?;
    let mut room = Darkroom::open(width, height).await?;
    let video_chunks: Chunks = Rc::default();
    let broken: Broken = Rc::default();
    let (output, error) = collector(&video_chunks, &broken);
    let encoder = VideoEncoder::new(&object(&[("output", output.as_ref().clone()), ("error", error.as_ref().clone())])).map_err(describe)?;

    encoder.configure(&video_config).map_err(describe)?;

    let mut sounds: Vec<(u32, AudioCommand)> = Vec::new();
    let mut drawn = 0usize;
    let mut over = opened.over;
    let mut shown = 0;
    let mut yielded = clock();

    if !over {
        (wiring.send)(BattleCommand::Roll(BATCH));
    }

    while !over {
        let rolled = next(&mut receiver).await?;

        over = rolled.over;

        if !over {
            (wiring.send)(BattleCommand::Roll(BATCH));
        }

        let Rolled { frames, sheets, audio, total, .. } = *rolled;

        room.develop(sheets);
        sounds.extend(audio);

        for frame in &frames {
            if abort.get() {
                let _ = encoder.close();

                return Ok(None);
            }

            if let Some(reason) = broken.borrow().clone() {
                return Err(reason);
            }

            room.expose(frame)?;

            let init = object(&[("timestamp", JsValue::from((drawn as f64 * MICROS / FPS).round())), ("duration", JsValue::from((MICROS / FPS).round())), ("alpha", JsValue::from_str("discard"))]);
            let shot = VideoFrame::new(room.canvas(), &init).map_err(describe)?;
            let encoded = encoder.encode(&shot, &object(&[("keyFrame", JsValue::from(drawn.is_multiple_of(KEY_EVERY)))]));

            shot.close();
            encoded.map_err(describe)?;
            drawn += 1;

            while encoder.encode_queue_size() > QUEUE_LIMIT {
                pause(1).await;
                yielded = clock();
            }

            let percent = (drawn * 100 / total.max(1)).min(100);

            if percent != shown {
                shown = percent;
                (wiring.tell)(BattleEvent::Rendering(drawn as f32 / total.max(1) as f32));
            }

            if clock() - yielded > BUSY_MS {
                pause(0).await;
                yielded = clock();
            }
        }
    }

    JsFuture::from(encoder.flush()).await.map_err(describe)?;
    let _ = encoder.close();

    if let Some(reason) = broken.borrow().clone() {
        return Err(reason);
    }

    (wiring.tell)(BattleEvent::Sounding(0.0));

    let samples = mix(&sounds, drawn).await?;

    if abort.get() {
        return Ok(None);
    }

    let audio_chunks = encode_audio(&audio_config, &samples, wiring, abort).await?;

    if abort.get() {
        return Ok(None);
    }

    drop((output, error));

    let video = std::mem::take(&mut *video_chunks.borrow_mut());

    mux(width, height, &sound, video, audio_chunks).map(Some)
}

async fn mix(sounds: &[(u32, AudioCommand)], frames: usize) -> Result<[Vec<f32>; 2], String> {
    let length = (frames.max(1) as u32) * SAMPLES_PER_FRAME;
    let context = OfflineAudioContext::new_with_number_of_channels_and_length_and_sample_rate(CHANNELS, length, RATE as f32).map_err(describe)?;
    let mut buffers: HashMap<Box<str>, AudioBuffer> = HashMap::new();

    for (_, command) in sounds {
        match command {
            AudioCommand::Effect { clip, channels, rate, samples } => {
                let channels = u32::from((*channels).max(1));
                let count = (samples.len() as u32 / channels).max(1);
                let buffer = context.create_buffer(channels, count, (*rate).max(8000) as f32).map_err(describe)?;

                for channel in 0..channels {
                    let lane: Vec<f32> = samples.iter().skip(channel as usize).step_by(channels as usize).copied().collect();

                    buffer.copy_to_channel(&lane, channel as i32).map_err(describe)?;
                }

                buffers.insert(clip.clone(), buffer);
            }
            AudioCommand::Track { clip, bytes } => {
                let copied = Uint8Array::from(bytes.as_slice()).buffer();

                match context.decode_audio_data(&copied) {
                    Ok(decoding) => match JsFuture::from(decoding).await {
                        Ok(decoded) => {
                            buffers.insert(clip.clone(), decoded.unchecked_into());
                        }
                        Err(err) => warn!("{clip} could not be decoded for the video: {}", describe(err)),
                    },
                    Err(err) => warn!("{clip} could not be decoded for the video: {}", describe(err)),
                }
            }
            _ => {}
        }
    }

    let mut voices: HashMap<u32, (AudioBufferSourceNode, GainNode)> = HashMap::new();

    for (frame, command) in sounds {
        let at = f64::from(*frame) / FPS;

        match command {
            AudioCommand::Start { voice, clip, looped, level } => {
                let Some(buffer) = buffers.get(clip) else {
                    continue;
                };
                let source = context.create_buffer_source().map_err(describe)?;
                let gain = context.create_gain().map_err(describe)?;

                source.set_buffer(Some(buffer));
                source.set_loop(*looped);
                gain.gain().set_value_at_time(*level, at).map_err(describe)?;
                source.connect_with_audio_node(&gain).map_err(describe)?;
                gain.connect_with_audio_node(&context.destination()).map_err(describe)?;
                source.start_with_when(at).map_err(describe)?;
                voices.insert(*voice, (source, gain));
            }
            AudioCommand::Level { voice, level } => {
                if let Some((_, gain)) = voices.get(voice) {
                    gain.gain().set_value_at_time(*level, at).map_err(describe)?;
                }
            }
            AudioCommand::Stop { voice } => {
                if let Some((source, _)) = voices.remove(voice) {
                    let _ = web_sys::AudioScheduledSourceNode::stop_with_when(&source, at);
                }
            }
            AudioCommand::Effect { .. } | AudioCommand::Track { .. } => {}
        }
    }

    let rendered: AudioBuffer = JsFuture::from(context.start_rendering().map_err(describe)?).await.map_err(describe)?.unchecked_into();

    Ok([rendered.get_channel_data(0).map_err(describe)?, rendered.get_channel_data(1).map_err(describe)?])
}

async fn encode_audio(config: &Object, samples: &[Vec<f32>; 2], wiring: &Wiring, abort: &Cell<bool>) -> Result<Vec<Chunk>, String> {
    let chunks: Chunks = Rc::default();
    let broken: Broken = Rc::default();
    let (output, error) = collector(&chunks, &broken);
    let encoder = AudioEncoder::new(&object(&[("output", output.as_ref().clone()), ("error", error.as_ref().clone())])).map_err(describe)?;
    let [left, right] = samples;
    let total = left.len();

    encoder.configure(config).map_err(describe)?;

    for start in (0..total).step_by(SLICE) {
        if abort.get() {
            let _ = encoder.close();

            return Ok(Vec::new());
        }

        let end = (start + SLICE).min(total);
        let count = end - start;
        let planar = Float32Array::new_with_length((count * 2) as u32);

        planar.subarray(0, count as u32).copy_from(left.get(start..end).unwrap_or_default());
        planar.subarray(count as u32, (count * 2) as u32).copy_from(right.get(start..end).unwrap_or_default());

        let data = AudioData::new(&object(&[
            ("format", JsValue::from_str("f32-planar")),
            ("sampleRate", JsValue::from(RATE)),
            ("numberOfFrames", JsValue::from(count as u32)),
            ("numberOfChannels", JsValue::from(CHANNELS)),
            ("timestamp", JsValue::from((start as f64 * MICROS / f64::from(RATE)).round())),
            ("data", planar.into()),
        ]))
        .map_err(describe)?;
        let encoded = encoder.encode(&data);

        data.close();
        encoded.map_err(describe)?;
        (wiring.tell)(BattleEvent::Sounding(end as f32 / total.max(1) as f32));
        pause(0).await;
    }

    JsFuture::from(encoder.flush()).await.map_err(describe)?;
    let _ = encoder.close();

    if let Some(reason) = broken.borrow().clone() {
        return Err(reason);
    }

    drop((output, error));

    Ok(std::mem::take(&mut *chunks.borrow_mut()))
}

fn mux(width: u32, height: u32, sound: &Sound, video: Vec<Chunk>, audio: Vec<Chunk>) -> Result<Vec<u8>, String> {
    let sink = Sink::default();
    let codec = match sound {
        Sound::Aac => AudioCodec::Aac(AacProfile::Lc),
        Sound::Opus => AudioCodec::Opus,
    };
    let mut muxer = MuxerBuilder::new(sink.clone())
        .video(VideoCodec::H264, width, height, FPS)
        .audio(codec, RATE, CHANNELS as u16)
        .build()
        .map_err(|err| format!("the video could not be started: {err}"))?;
    if video.windows(2).any(|pair| pair[1].at < pair[0].at) {
        return Err(REORDERED.to_owned());
    }

    let step = MICROS / FPS;
    let delay = video.iter().enumerate().map(|(index, chunk)| index as f64 * step - chunk.at).fold(0.0, f64::max).ceil();
    let mut audio = audio.into_iter().peekable();

    for (index, chunk) in video.into_iter().enumerate() {
        let decoded = index as f64 * step;

        while let Some(sound) = audio.next_if(|sound| sound.at.max(0.0) + delay < decoded) {
            muxer.write_audio((sound.at.max(0.0) + delay) / MICROS, &sound.data).map_err(|err| format!("the sound could not be written: {err}"))?;
        }

        muxer
            .write_video_with_dts((chunk.at + delay) / MICROS, decoded / MICROS, &chunk.data, chunk.key)
            .map_err(|err| format!("a frame could not be written: {err}"))?;
    }

    for sound in audio {
        muxer.write_audio((sound.at.max(0.0) + delay) / MICROS, &sound.data).map_err(|err| format!("the sound could not be written: {err}"))?;
    }

    muxer.finish().map_err(|err| format!("the video could not be finished: {err}"))?;

    let bytes = std::mem::take(&mut *sink.0.borrow_mut());

    Ok(bytes)
}
