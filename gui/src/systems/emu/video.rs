use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use iced::futures::channel::mpsc as async_mpsc;
use iced::futures::executor::block_on;
use iced::futures::Stream;
use iced::widget::shader::Pipeline as _;
use iced::{wgpu, Rectangle};
use tracing::{error, info, warn};

use kore::Vfs;
use kore::common::architecture;
use kore::common::job::JobOutcome;
use kore::domains::sandbox::replay as tape;
use kore::domains::settings::ReplaySource;
use kore::systems::addons::ffmpeg::video::{self, Quality, VideoFormat, VideoWriter};

use super::assets::{DiskAssets, FileIndex};
use super::driver::Driver;
use super::pipeline::{Pipeline, Run};
use super::session::{Reel as Source, CLOSED_FRAME};
use super::soundtrack::{self, SharedLog, SoundLog, GAME_FPS};
use super::viewport::{paint, Scene};

const EXPORTS: &str = "exports";
const PROGRESS_EVERY: Duration = Duration::from_millis(50);
const QUEUED_FRAMES: usize = 4;
const MIX_SHARE: f32 = 0.5;
const TEMP_VIDEO: &str = "video";
const TEMP_AUDIO: &str = "audio.wav";
const WORK_FOLDER: &str = "video";
const LATEST_WORK: &str = "latest";
const BUNDLE_WORK: &str = "bcv-";

pub struct VideoJob {
    reel: Source,
    index: Option<FileIndex>,
    format: VideoFormat,
    quality: Quality,
    title: String,
    abort: Arc<AtomicBool>,
}

impl VideoJob {
    pub fn new(reel: Source, source: ReplaySource, vfs: &Vfs, format: VideoFormat, quality: Quality, title: String, abort: Arc<AtomicBool>) -> Self {
        let index = match source {
            ReplaySource::Bcv => None,
            ReplaySource::Vfs => Some(DiskAssets::index(vfs)),
        };

        Self { reel, index, format, quality, title, abort }
    }
}

#[derive(Clone, Debug)]
pub enum VideoEvent {
    Rendered(u32, u32),
    Sound(f32),
    Done(JobOutcome),
}

pub fn spawn(job: VideoJob) -> impl Stream<Item = VideoEvent> {
    let (sender, receiver) = async_mpsc::unbounded();

    thread::spawn(move || {
        let progress = sender.clone();
        let outcome = match run(&job, &|event| {
            let _ = progress.unbounded_send(event);
        }) {
            Ok(true) => JobOutcome::Completed,
            Ok(false) => {
                info!("Replay video export of {} was aborted", job.title);

                JobOutcome::Aborted
            }
            Err(reason) => {
                error!("Replay video could not be exported: {reason}");

                JobOutcome::Failed(reason)
            }
        };

        let _ = sender.unbounded_send(VideoEvent::Done(outcome));
    });

    receiver
}

fn even(size: f32) -> u32 {
    ((size.max(2.0) as u32) / 2 * 2).max(2)
}

fn destination(title: &str, format: VideoFormat) -> PathBuf {
    Path::new(EXPORTS).join(format!("{title}.{}", format.extension()))
}

fn workspace(reel: &Source) -> PathBuf {
    let name = match reel {
        Source::Latest => LATEST_WORK.to_owned(),
        Source::Bundle(bundle) => format!("{BUNDLE_WORK}{}", bundle.file_stem().map_or_else(String::new, |stem| stem.to_string_lossy().into_owned())),
    };

    Path::new(architecture::WORK).join(WORK_FOLDER).join(name)
}

fn run(job: &VideoJob, emit: &dyn Fn(VideoEvent)) -> Result<bool, String> {
    let _work = architecture::Scratch::claim();
    let work = workspace(&job.reel);
    let outcome = film(job, &work, emit);

    if work.exists()
        && let Err(failure) = fs::remove_dir_all(&work)
    {
        warn!("{} could not be removed: {failure}", work.display());
    }

    outcome
}

fn film(job: &VideoJob, work: &Path, emit: &dyn Fn(VideoEvent)) -> Result<bool, String> {
    let dir = match &job.reel {
        Source::Latest => {
            fs::create_dir_all(work).map_err(|failure| format!("{} could not be created: {failure}", work.display()))?;

            tape::scratch().ok_or("there is no state folder holding the latest battle")?
        }
        Source::Bundle(bundle) => {
            tape::unpack(bundle, work)?;

            work.to_path_buf()
        }
    };
    let save = tape::read_save(&dir)?;
    let frames = tape::read_input(&dir)?;
    let index = match &job.index {
        Some(index) => index.clone(),
        None => tape::index(&dir).map_err(|failure| format!("the replay's asset list could not be read: {failure}"))?,
    };
    let total = u32::try_from(frames.len()).unwrap_or(u32::MAX);
    let (width, height) = (even(save.screen.width), even(save.screen.height));

    fs::create_dir_all(EXPORTS).map_err(|failure| format!("the exports folder could not be created: {failure}"))?;

    let log = Rc::new(RefCell::new(SoundLog::new(save.options.music, save.options.effects)));
    let mut driver = Driver::headless(Rc::clone(&log));

    driver.arm_playback(&save, frames, index.clone());

    if !driver.boot() {
        return Err("the replay's game data could not be loaded".to_owned());
    }

    if !driver.enter_battle() {
        return Err("the replay's battle could not be entered".to_owned());
    }

    let mut canvas = Canvas::new(width, height)?;
    let temp_video = work.join(format!("{TEMP_VIDEO}.{}", job.format.extension()));
    let encoder = Encoder::start(VideoWriter::start(width, height, GAME_FPS, job.format, job.quality, &temp_video)?);
    let mut reel = Reel { driver: &mut driver, log: &log, canvas: &mut canvas, encoder: &encoder, abort: &job.abort, drawn: 0, played: 0 };
    let filmed = reel.roll(total, emit);
    let drawn = reel.drawn;

    match (filmed, encoder.close()) {
        (Ok(false), closed) => {
            if let Ok(writer) = closed {
                writer.abort();
            }

            return Ok(false);
        }
        (_, Err(reason)) => return Err(reason),
        (Err(reason), Ok(writer)) => {
            writer.abort();

            return Err(reason);
        }
        (Ok(true), Ok(writer)) => writer.finish()?,
    }

    emit(VideoEvent::Sound(0.0));

    let temp_audio = work.join(TEMP_AUDIO);

    soundtrack::mix(&log.borrow(), &index, drawn, &temp_audio, &|ratio| emit(VideoEvent::Sound(ratio * MIX_SHARE)), &job.abort)
        .map_err(|failure| format!("the replay's sound could not be written: {failure}"))?;

    if job.abort.load(Ordering::Relaxed) {
        return Ok(false);
    }

    let out = destination(&job.title, job.format);

    let length = Duration::from_secs_f64(f64::from(drawn) / f64::from(GAME_FPS));
    let muxed = video::mux(&temp_video, &temp_audio, job.format, &out, length, |ratio| emit(VideoEvent::Sound(MIX_SHARE + ratio * (1.0 - MIX_SHARE))), &job.abort)?;

    if !muxed {
        return Ok(false);
    }

    emit(VideoEvent::Sound(1.0));
    info!("Replay video exported to {} ({drawn} frames)", out.display());

    Ok(true)
}

struct Reel<'a> {
    driver: &'a mut Driver,
    log: &'a SharedLog,
    canvas: &'a mut Canvas,
    encoder: &'a Encoder,
    abort: &'a AtomicBool,
    drawn: u32,
    played: u32,
}

impl Reel<'_> {
    fn shoot(&mut self) -> Result<(), String> {
        let bounds = Rectangle { x: 0.0, y: 0.0, width: self.canvas.width as f32, height: self.canvas.height as f32 };
        let scene = paint(&self.driver.frame().borrow(), &self.driver.sheets().borrow(), self.driver.design_width(), Some(self.driver.frame_aspect()), bounds);
        let mut pixels = self.encoder.spare();

        self.canvas.draw(scene, &mut pixels)?;
        self.encoder.send(pixels)?;
        self.drawn += 1;

        Ok(())
    }

    fn close(&mut self) -> Result<(), String> {
        let frozen = self.driver.frame().borrow().quads.len();

        for sweep in 1..=CLOSED_FRAME {
            self.driver.frame().borrow_mut().quads.truncate(frozen);
            self.driver.draw_curtain_over(sweep);
            self.shoot()?;
        }

        Ok(())
    }

    fn roll(&mut self, total: u32, emit: &dyn Fn(VideoEvent)) -> Result<bool, String> {
        let mut reported = Instant::now();

        self.driver.draw_curtain_aside(CLOSED_FRAME);
        self.shoot()?;

        loop {
            if self.abort.load(Ordering::Relaxed) {
                return Ok(false);
            }

            self.log.borrow_mut().frame = self.drawn;

            let advanced = self.driver.advance();

            self.played += 1;

            if let Err(reason) = advanced {
                warn!("Replay video closes at a fault on frame {}: {reason}", self.drawn);
                self.close()?;
                break;
            }

            if !self.driver.in_battle() {
                self.driver.draw_curtain_aside(CLOSED_FRAME);
                self.shoot()?;
                break;
            }

            if self.driver.reel_finished() || self.driver.exit_requested() {
                self.close()?;
                break;
            }

            self.shoot()?;

            if reported.elapsed() >= PROGRESS_EVERY {
                reported = Instant::now();
                emit(VideoEvent::Rendered(self.played.min(total), total));
            }
        }

        emit(VideoEvent::Rendered(total, total));

        Ok(true)
    }
}

struct Encoder {
    frames: SyncSender<Vec<u8>>,
    spares: Receiver<Vec<u8>>,
    worker: JoinHandle<Result<VideoWriter, String>>,
}

impl Encoder {
    fn start(writer: VideoWriter) -> Self {
        let (frames, inbox) = mpsc::sync_channel::<Vec<u8>>(QUEUED_FRAMES);
        let (recycle, spares) = mpsc::channel();
        let worker = thread::spawn(move || {
            let mut writer = writer;

            for frame in inbox {
                if let Err(reason) = writer.frame(&frame) {
                    error!("Replay video encoder stopped: {reason}");
                    writer.abort();

                    return Err(reason);
                }

                let _ = recycle.send(frame);
            }

            Ok(writer)
        });

        Self { frames, spares, worker }
    }

    fn spare(&self) -> Vec<u8> {
        self.spares.try_recv().unwrap_or_default()
    }

    fn send(&self, frame: Vec<u8>) -> Result<(), String> {
        self.frames.send(frame).map_err(|_| "the video encoder stopped".to_owned())
    }

    fn close(self) -> Result<VideoWriter, String> {
        drop(self.frames);

        self.worker.join().unwrap_or_else(|_| Err("the video encoder thread panicked".to_owned()))
    }
}

struct Canvas {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: Pipeline,
    target: wgpu::Texture,
    readback: wgpu::Buffer,
    width: u32,
    height: u32,
    padded: u32,
}

impl Canvas {
    fn new(width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .map_err(|failure| format!("no GPU adapter is available for the video: {failure}"))?;
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .map_err(|failure| format!("the video's GPU device could not be created: {failure}"))?;
        let format = wgpu::TextureFormat::Rgba8Unorm;
        let pipeline = Pipeline::new(&device, &queue, format);
        let target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("replay_video_target"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let padded = (width * 4).div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("replay_video_readback"),
            size: u64::from(padded) * u64::from(height),
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self { device, queue, pipeline, target, readback, width, height, padded })
    }

    fn draw(&mut self, scene: Scene, pixels: &mut Vec<u8>) -> Result<(), String> {
        for (name, sheet) in &scene.uploads {
            self.pipeline.ensure_sheet(&self.device, &self.queue, name, sheet);
        }

        let runs = scene.runs.iter().map(|(sheet, blend, start, end)| Run { sheet: sheet.clone(), blend: *blend, range: *start..*end }).collect();

        self.pipeline.write(&self.device, &self.queue, &scene.vertices, runs);

        let view = self.target.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("replay_video") });

        drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("replay_video_clear"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }));

        self.pipeline.draw(&mut encoder, &view, &Rectangle { x: 0, y: 0, width: self.width, height: self.height });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo { texture: &self.target, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo {
                buffer: &self.readback,
                layout: wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(self.padded), rows_per_image: Some(self.height) },
            },
            wgpu::Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
        );
        self.queue.submit([encoder.finish()]);

        let slice = self.readback.slice(..);
        let (sender, receiver) = mpsc::channel();

        slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = sender.send(result);
        });

        self.device.poll(wgpu::PollType::wait_indefinitely()).map_err(|failure| format!("the GPU stalled while reading a frame back: {failure}"))?;

        match receiver.recv() {
            Ok(Ok(())) => {}
            Ok(Err(failure)) => return Err(format!("a frame could not be read back: {failure}")),
            Err(_) => return Err("a frame was never read back".to_owned()),
        }

        let row = (self.width * 4) as usize;

        pixels.clear();
        pixels.reserve(row * self.height as usize);

        {
            let mapped = slice.get_mapped_range();

            for line in mapped.chunks_exact(self.padded as usize).take(self.height as usize) {
                pixels.extend_from_slice(&line[..row]);
            }
        }

        self.readback.unmap();

        Ok(())
    }
}
