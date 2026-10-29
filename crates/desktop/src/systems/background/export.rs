use std::borrow::Cow;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, PoisonError};
use std::thread;

use arboard::{Clipboard, ImageData};
use iced::futures::channel::mpsc::unbounded;
use iced::{task, Task};
use image::RgbaImage;
use tracing::{error, info, warn};

use kokoro::domains::utilities::background::FRAME_RATE;
use kokoro::systems::animation::export::{leader, process, EncoderMessage, EncoderStatus, ExportFormat, ExportMode, ExportRequest, FrameTiming, ShowcaseLengths};
use codex::graphics::backdrop::{Backdrop, Camera};

use crate::common::feedback::Slot;

use super::controls::Feedback;
use super::offscreen::Renderer;

const FRAME_DELAY_MS: u32 = 1000 / FRAME_RATE;
const QUEUED_FRAMES_MAX: usize = 64;
const QUEUED_FRAMES_MIN: usize = 2;
const QUEUED_BYTES: usize = 256 * 1024 * 1024;

pub struct Request {
    pub backdrop: Backdrop,
    pub seed: u32,
    pub start: i32,
    pub end: i32,
    pub stem: String,
    pub format: ExportFormat,
    pub quality: u32,
    pub compression: u32,
    pub camera: Camera,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub enum Event {
    Encoder(EncoderStatus),
    Rendered(Result<(), String>),
}

#[derive(Clone)]
pub enum Message {
    Export(Event),
    Copied(u64, Result<(), String>),
    ExportExpired,
    CopyExpired,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Export(event) => write!(f, "Export({event:?})"),
            Self::Copied(generation, result) => write!(f, "Copied({generation}, {})", result.is_ok()),
            Self::ExportExpired => write!(f, "ExportExpired"),
            Self::CopyExpired => write!(f, "CopyExpired"),
        }
    }
}

struct Job {
    abort: Arc<AtomicBool>,
    handle: task::Handle,
    aborting: bool,
    rendered: Option<bool>,
    encoded: bool,
}

#[derive(Default)]
pub struct State {
    job: Option<Job>,
    copying: Option<u64>,
    copies: u64,
    export_feedback: Slot<Feedback>,
    copy_feedback: Slot<Feedback>,
    clipboard: Arc<Mutex<Option<Clipboard>>>,
}

pub(super) fn is_forced_opaque(format: &ExportFormat) -> bool {
    matches!(format, ExportFormat::Mp4 | ExportFormat::Mkv | ExportFormat::Webm)
}

fn queue_depth(width: u32, height: u32) -> usize {
    let frame_bytes = (width as usize * height as usize * 4).max(1);

    (QUEUED_BYTES / frame_bytes).clamp(QUEUED_FRAMES_MIN, QUEUED_FRAMES_MAX)
}

fn place_on_clipboard(shared: &Mutex<Option<Clipboard>>, image: &RgbaImage) -> Result<(), String> {
    let mut held = shared.lock().unwrap_or_else(PoisonError::into_inner);

    if held.is_none() {
        *held = Some(Clipboard::new().map_err(|err| format!("Clipboard unavailable: {err}"))?);
    }

    let clipboard = held.as_mut().ok_or_else(|| "Clipboard unavailable".to_owned())?;
    let (width, height) = image.dimensions();
    let data = ImageData { width: width as usize, height: height as usize, bytes: Cow::Borrowed(image.as_raw()) };

    clipboard.set_image(data).map_err(|err| err.to_string())
}

fn seek(backdrop: &mut Backdrop, seed: u32, frame: i32) {
    backdrop.restart(seed);

    for _ in 0..frame.max(0) {
        backdrop.tick();
    }
}

fn flipped(pixels: Vec<u8>, width: u32, height: u32) -> Option<RgbaImage> {
    let row = (width * 4) as usize;
    let mut upright = Vec::with_capacity(pixels.len());

    for line in (0..height as usize).rev() {
        upright.extend_from_slice(pixels.get(line * row..(line + 1) * row)?);
    }

    RgbaImage::from_raw(width, height, upright)
}

fn straight(mut image: RgbaImage) -> RgbaImage {
    for pixel in image.pixels_mut() {
        let alpha = u32::from(pixel[3]);

        if alpha == 0 || alpha == 255 {
            continue;
        }

        for channel in &mut pixel.0[..3] {
            *channel = (u32::from(*channel) * 255 / alpha).min(255) as u8;
        }
    }

    image
}

impl State {
    pub fn busy(&self) -> bool {
        self.job.is_some() || self.copying.is_some()
    }

    pub fn export_feedback(&self) -> Feedback {
        if self.job.is_some() {
            return Feedback::Busy;
        }

        self.export_feedback.get().copied().unwrap_or(Feedback::Idle)
    }

    pub fn copy_feedback(&self) -> Feedback {
        if self.copying.is_some() {
            return Feedback::Busy;
        }

        self.copy_feedback.get().copied().unwrap_or(Feedback::Idle)
    }

    pub fn abort_export(&mut self) {
        if let Some(job) = self.job.as_mut()
            && !job.aborting
        {
            job.aborting = true;
            job.abort.store(true, Ordering::Relaxed);
        }
    }

    pub fn abort_copy(&mut self) -> Task<Message> {
        if self.copying.take().is_none() {
            return Task::none();
        }

        self.copy_feedback.set(Feedback::Aborted, Message::CopyExpired)
    }

    pub fn begin_export(&mut self, request: Request) -> Task<Message> {
        if self.job.is_some() {
            return Task::none();
        }

        let Request { mut backdrop, seed, start, end, stem, format, quality, compression, camera, width, height } = request;
        let config = process::build_config(&ExportRequest {
            timing: FrameTiming { mode: ExportMode::Manual, frame_start: start, frame_end: end, loop_supported: false },
            showcase: ShowcaseLengths { walk: 0, idle: 0, attack: 0, kb: 0 },
            file_name: (start == end).then(|| stem.clone()),
            name_prefix: stem,
            format: format.clone(),
            quality_percent: quality,
            compression_percent: compression,
            fps: FRAME_RATE,
            width,
            height,
            background: is_forced_opaque(&format),
        });
        let (first, last) = (config.start_frame, config.end_frame);
        let abort = Arc::new(AtomicBool::new(false));
        let (frame_tx, frame_rx) = mpsc::sync_channel(queue_depth(width, height));
        let (tx, rx) = unbounded();

        let encoder_tx = tx.clone();
        let encoder_abort = Arc::clone(&abort);

        thread::spawn(move || {
            leader::run(
                config,
                frame_rx,
                |status| {
                    let _ = encoder_tx.unbounded_send(Event::Encoder(status));
                },
                &encoder_abort,
            );
        });

        let render_abort = Arc::clone(&abort);

        thread::spawn(move || {
            let shot = Shot { seed, first, last, camera, size: (width, height) };
            let outcome = render(&mut backdrop, shot, &frame_tx, &render_abort);

            if let Err(reason) = &outcome
                && !render_abort.load(Ordering::Relaxed)
            {
                error!("Background export rendering failed: {reason}");
                render_abort.store(true, Ordering::Relaxed);
            }

            let _ = frame_tx.send(EncoderMessage::Finish);
            let _ = tx.unbounded_send(Event::Rendered(outcome));
        });

        let (events, handle) = Task::stream(rx).map(Message::Export).abortable();

        self.job = Some(Job { abort, handle: handle.abort_on_drop(), aborting: false, rendered: None, encoded: false });

        events
    }

    pub fn begin_copy(&mut self, mut backdrop: Backdrop, seed: u32, frame: i32, camera: Camera, size: (u32, u32)) -> Task<Message> {
        if self.copying.is_some() {
            return Task::none();
        }

        self.copies = self.copies.wrapping_add(1);

        let generation = self.copies;

        self.copying = Some(generation);

        let clipboard = Arc::clone(&self.clipboard);

        Task::perform(
            smol::unblock(move || {
                let mut renderer = Renderer::new(size.0, size.1)?;

                seek(&mut backdrop, seed, frame);

                let pixels = renderer.render(&mut backdrop, camera)?;
                let image = flipped(pixels, renderer.width(), renderer.height())
                    .map(straight)
                    .ok_or_else(|| "The rendered background frame had the wrong size".to_owned())?;

                place_on_clipboard(&clipboard, &image)
            }),
            move |outcome| Message::Copied(generation, outcome),
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Export(event) => {
                let Some(job) = self.job.as_mut() else {
                    return Task::none();
                };

                match event {
                    Event::Encoder(EncoderStatus::Progress(_)) => return Task::none(),
                    Event::Encoder(EncoderStatus::Finished) => job.encoded = true,
                    Event::Rendered(outcome) => job.rendered = Some(outcome.is_ok()),
                }

                let (Some(rendered), true) = (job.rendered, job.encoded) else {
                    return Task::none();
                };
                let aborted = job.aborting;
                let ok = rendered && !job.abort.load(Ordering::Relaxed);

                if let Some(job) = self.job.take() {
                    job.handle.abort();
                }

                let feedback = if aborted {
                    info!("Background export aborted");
                    Feedback::Aborted
                } else if ok {
                    info!("Background export finished");
                    Feedback::Done(true)
                } else {
                    warn!("Background export did not complete");
                    Feedback::Done(false)
                };

                self.export_feedback.set(feedback, Message::ExportExpired)
            }
            Message::Copied(generation, outcome) => {
                if self.copying != Some(generation) {
                    return Task::none();
                }

                self.copying = None;

                if let Err(err) = &outcome {
                    error!("Background copy failed: {err}");
                }

                self.copy_feedback.set(Feedback::Done(outcome.is_ok()), Message::CopyExpired)
            }
            Message::ExportExpired => {
                self.export_feedback.expire();
                Task::none()
            }
            Message::CopyExpired => {
                self.copy_feedback.expire();
                Task::none()
            }
        }
    }
}

struct Shot {
    seed: u32,
    first: i32,
    last: i32,
    camera: Camera,
    size: (u32, u32),
}

fn render(backdrop: &mut Backdrop, shot: Shot, frames: &mpsc::SyncSender<EncoderMessage>, abort: &AtomicBool) -> Result<(), String> {
    let Shot { seed, first, last, camera, size } = shot;
    let mut renderer = Renderer::new(size.0, size.1)?;
    let (low, high) = (first.min(last), first.max(last));

    seek(backdrop, seed, low);

    for frame in low..=high {
        if abort.load(Ordering::Relaxed) {
            return Err("the export was aborted".to_owned());
        }

        let pixels = renderer.render(backdrop, camera)?;

        frames
            .send(EncoderMessage::Frame(pixels, renderer.width(), renderer.height(), FRAME_DELAY_MS))
            .map_err(|_| "the encoder stopped accepting frames".to_owned())?;

        if frame < high {
            backdrop.tick();
        }
    }

    Ok(())
}
