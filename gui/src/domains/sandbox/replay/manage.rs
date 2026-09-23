use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use iced::alignment::Vertical;
use iced::futures::channel::mpsc;
use iced::widget::{column, pick_list, progress_bar, row, text, text_input};
use iced::{Element, Length, Task};
use tracing::warn;

use kore::common::job::JobOutcome;
use kore::domains::sandbox::replay as tape;
use kore::domains::settings::Settings;
use kore::systems::addons::ffmpeg::{self, video::{Quality, VideoFormat}};
use kore::Vfs;

use crate::app::theme;
use crate::common::feedback::{Slot, FAILURE_LABEL};
use crate::systems::emu::{export_video, Reel, VideoEvent, VideoJob};
use crate::widget::hover_hint;

use super::Pick;

const WIDTH: f32 = 240.0;
const INPUT_WIDTH: f32 = 55.0;
const PERCENT_MIN: u32 = 1;
const PERCENT_MAX: u32 = 100;
const FORMAT_WIDTH: f32 = 84.0;
const SPACING: f32 = 6.0;
const STATUS_GAP: f32 = 8.0;
const STATUS_SIZE: f32 = 13.0;
const STATUS_HOLD: Duration = Duration::from_secs(3);
const QUALITY_HINT: &str = "80";
const COMPRESSION_HINT: &str = "30";
const READY_LABEL: &str = "Ready";
const DONE_LABEL: &str = "Done";
const NO_FFMPEG_HINT: &str = "Requires the FFMPEG Addon\nInstall under Settings > Addons > FFMPEG";
const UNSAVED_HINT: &str = "Can't delete a replay that isn't saved!";

#[derive(Clone, Debug)]
pub enum Update {
    Progress(&'static str, f32),
    Finished(JobOutcome),
}

#[derive(Clone, Debug)]
pub enum Message {
    Save,
    Copy,
    Export,
    Abort,
    Delete,
    Format(VideoFormat),
    Quality(String),
    Compression(String),
    Job(Pick, Update),
    ButtonExpired(Pick),
    StatusExpired(Pick),
    DeleteExpired,
}

pub enum Effect {
    None,
    Saved,
    Deleted(PathBuf),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Work {
    Save,
    Copy,
    Video,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ended {
    Completed,
    Aborted,
    Failed,
}

struct Status {
    ratio: f32,
    label: String,
}

impl Default for Status {
    fn default() -> Self {
        Self { ratio: 1.0, label: READY_LABEL.to_owned() }
    }
}

impl Status {
    fn at(ratio: f32, label: impl Into<String>) -> Self {
        Self { ratio, label: label.into() }
    }

    fn progress(stage: &str, ratio: f32) -> Self {
        let ratio = ratio.clamp(0.0, 1.0);

        Self::at(ratio, format!("{stage} | {}%", (ratio * 100.0) as i32))
    }
}

#[derive(Default)]
struct Job {
    running: Option<Work>,
    aborting: bool,
    abort: Arc<AtomicBool>,
    status: Status,
    settled: Slot<()>,
    ended: Slot<(Work, Ended)>,
}

impl Job {
    fn begin(&mut self, work: Work, status: Status) -> Arc<AtomicBool> {
        self.running = Some(work);
        self.aborting = false;
        self.abort = Arc::new(AtomicBool::new(false));
        self.status = status;
        self.settled.clear();
        self.ended.clear();

        Arc::clone(&self.abort)
    }

    fn ended(&self, work: Work) -> Option<Ended> {
        self.ended.get().filter(|(held, _)| *held == work).map(|(_, ended)| *ended)
    }
}

#[derive(Default)]
pub struct State {
    jobs: HashMap<Pick, Job>,
    deleting: Slot<()>,
    ffmpeg: bool,
    format: VideoFormat,
    quality: String,
    compression: String,
}

fn percentage(input: &str) -> Option<(Option<u32>, String)> {
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if input.is_empty() {
        return Some((None, String::new()));
    }

    let clamped = input.parse::<u32>().unwrap_or(PERCENT_MAX).clamp(PERCENT_MIN, PERCENT_MAX);

    Some((Some(clamped), clamped.to_string()))
}

fn field<'a>(label: &'a str, hint: &'a str, value: &'a str, on_input: fn(String) -> Message) -> Element<'a, Message> {
    row![
        text(label).size(STATUS_SIZE).width(Length::Fill),
        text_input(hint, value).on_input(on_input).width(Length::Fixed(INPUT_WIDTH)).style(theme::rounded_input),
    ]
        .spacing(SPACING)
        .align_y(Vertical::Center)
        .into()
}

fn reel_of(pick: &Pick) -> Reel {
    match pick {
        Pick::Latest => Reel::Latest,
        Pick::Bundle(path) => Reel::Bundle(path.clone()),
    }
}

fn save(dir: PathBuf, out: PathBuf, abort: Arc<AtomicBool>) -> Task<Message> {
    let (sender, receiver) = mpsc::unbounded();

    thread::spawn(move || {
        let progress = sender.clone();
        let outcome = tape::pack(
            &dir,
            &out,
            |ratio| {
                let _ = progress.unbounded_send(Update::Progress("Saving", ratio));
            },
            &abort,
        );

        let _ = sender.unbounded_send(Update::Finished(outcome));
    });

    Task::stream(receiver).map(|update| Message::Job(Pick::Latest, update))
}

fn copy(bundle: &Path) -> JobOutcome {
    match tape::export(bundle) {
        Ok(_) => JobOutcome::Completed,
        Err(reason) => {
            warn!("Replay could not be exported: {reason}");

            JobOutcome::Failed(reason)
        }
    }
}

impl State {
    pub fn refresh(&mut self, settings: &Settings, bundles: &[PathBuf]) {
        self.ffmpeg = ffmpeg::is_installed();
        self.format = settings.sandbox.video_format;
        self.quality = settings.sandbox.video_quality.map_or_else(String::new, |value| value.to_string());
        self.compression = settings.sandbox.video_compression.map_or_else(String::new, |value| value.to_string());
        self.jobs.retain(|pick, job| {
            job.running.is_some()
                || match pick {
                    Pick::Latest => true,
                    Pick::Bundle(path) => bundles.contains(path),
                }
        });
    }

    pub fn forget_confirm(&mut self) {
        self.deleting.clear();
    }

    pub fn rekey(&mut self, old: &Pick, fresh: Pick) {
        if let Some(job) = self.jobs.remove(old) {
            self.jobs.insert(fresh, job);
        }
    }

    pub fn busy(&self, pick: &Pick) -> bool {
        self.jobs.get(pick).is_some_and(|job| job.running.is_some())
    }

    pub fn update(&mut self, message: Message, picked: Option<&Pick>, title: &str, settings: &mut Settings, vfs: &Vfs) -> (Task<Message>, Effect) {
        match message {
            Message::Save => {
                if picked != Some(&Pick::Latest) || self.busy(&Pick::Latest) {
                    return (Task::none(), Effect::None);
                }

                let Some(dir) = tape::scratch() else {
                    warn!("Replay could not be saved: there is no state folder holding the latest battle");

                    return self.finish(Pick::Latest, JobOutcome::Failed("there is no state folder holding the latest battle".to_owned()));
                };
                let out = tape::next_bundle(&tape::library());
                let abort = self.jobs.entry(Pick::Latest).or_default().begin(Work::Save, Status::progress("Saving", 0.0));

                (save(dir, out, abort), Effect::None)
            }
            Message::Copy => {
                let Some(Pick::Bundle(bundle)) = picked.cloned() else {
                    return (Task::none(), Effect::None);
                };
                let key = Pick::Bundle(bundle.clone());

                if self.busy(&key) {
                    return (Task::none(), Effect::None);
                }

                self.jobs.entry(key.clone()).or_default().begin(Work::Copy, Status::at(0.0, "Exporting..."));

                let task = Task::perform(smol::unblock(move || copy(&bundle)), move |outcome| Message::Job(key.clone(), Update::Finished(outcome)));

                (task, Effect::None)
            }
            Message::Export => {
                let Some(key) = picked.cloned() else {
                    return (Task::none(), Effect::None);
                };

                if !self.ffmpeg || self.busy(&key) {
                    return (Task::none(), Effect::None);
                }

                let abort = self.jobs.entry(key.clone()).or_default().begin(Work::Video, Status::progress("Rendering", 0.0));
                let quality = Quality {
                    quality: settings.sandbox.video_quality.unwrap_or(Quality::DEFAULT_QUALITY),
                    compression: settings.sandbox.video_compression.unwrap_or(Quality::DEFAULT_COMPRESSION),
                };
                let job = VideoJob::new(reel_of(&key), settings.sandbox.replay_source, vfs, settings.sandbox.video_format, quality, title.to_owned(), abort);
                let task = Task::stream(export_video(job)).map(move |event| {
                    let update = match event {
                        VideoEvent::Rendered(done, total) => Update::Progress("Rendering", done as f32 / total.max(1) as f32),
                        VideoEvent::Sound(ratio) => Update::Progress("Adding Sound", ratio),
                        VideoEvent::Done(outcome) => Update::Finished(outcome),
                    };

                    Message::Job(key.clone(), update)
                });

                (task, Effect::None)
            }
            Message::Abort => {
                if let Some(job) = picked.and_then(|pick| self.jobs.get_mut(pick))
                    && matches!(job.running, Some(Work::Save | Work::Video))
                    && !job.aborting
                {
                    job.abort.store(true, Ordering::Relaxed);
                    job.aborting = true;
                    job.status = Status::at(job.status.ratio, "Aborting...");
                }

                (Task::none(), Effect::None)
            }
            Message::Delete => {
                let Some(Pick::Bundle(bundle)) = picked.cloned() else {
                    return (Task::none(), Effect::None);
                };
                let key = Pick::Bundle(bundle.clone());

                if self.busy(&key) {
                    return (Task::none(), Effect::None);
                }

                if !self.deleting.is_set() {
                    return (self.deleting.set((), Message::DeleteExpired), Effect::None);
                }

                self.deleting.clear();

                if let Err(reason) = tape::delete(&bundle) {
                    warn!("Replay could not be deleted: {reason}");

                    return self.finish(key, JobOutcome::Failed(reason));
                }

                self.jobs.remove(&key);

                (Task::none(), Effect::Deleted(bundle))
            }
            Message::Format(format) => {
                settings.sandbox.video_format = format;
                self.format = format;

                (Task::none(), Effect::None)
            }
            Message::Quality(input) => {
                if let Some((value, text)) = percentage(&input) {
                    settings.sandbox.video_quality = value;
                    self.quality = text;
                }

                (Task::none(), Effect::None)
            }
            Message::Compression(input) => {
                if let Some((value, text)) = percentage(&input) {
                    settings.sandbox.video_compression = value;
                    self.compression = text;
                }

                (Task::none(), Effect::None)
            }
            Message::Job(key, Update::Progress(stage, ratio)) => {
                if let Some(job) = self.jobs.get_mut(&key)
                    && job.running.is_some()
                    && !job.aborting
                {
                    job.status = Status::progress(stage, ratio);
                }

                (Task::none(), Effect::None)
            }
            Message::Job(key, Update::Finished(outcome)) => self.finish(key, outcome),
            Message::ButtonExpired(key) => {
                if let Some(job) = self.jobs.get_mut(&key) {
                    job.ended.expire();
                }

                (Task::none(), Effect::None)
            }
            Message::StatusExpired(key) => {
                if let Some(job) = self.jobs.get_mut(&key) {
                    job.settled.expire();

                    if job.running.is_none() {
                        job.status = Status::default();
                    }
                }

                (Task::none(), Effect::None)
            }
            Message::DeleteExpired => {
                self.deleting.expire();

                (Task::none(), Effect::None)
            }
        }
    }

    fn finish(&mut self, key: Pick, outcome: JobOutcome) -> (Task<Message>, Effect) {
        let job = self.jobs.entry(key.clone()).or_default();
        let work = job.running.take().unwrap_or(Work::Copy);
        let (ended, status) = match outcome {
            JobOutcome::Completed => (Ended::Completed, Status::at(1.0, DONE_LABEL)),
            JobOutcome::Aborted => (Ended::Aborted, Status::default()),
            JobOutcome::Failed(reason) => (Ended::Failed, Status::at(1.0, format!("Failed | {reason}"))),
        };

        job.aborting = false;
        job.status = status;

        let settled = job.settled.set_after((), Message::StatusExpired(key.clone()), STATUS_HOLD);
        let button = job.ended.set((work, ended), Message::ButtonExpired(key));
        let effect = if work == Work::Save && ended == Ended::Completed { Effect::Saved } else { Effect::None };

        (Task::batch([settled, button]), effect)
    }

    pub fn view(&self, picked: Option<&Pick>, ready: bool) -> Element<'_, Message> {
        let job = picked.and_then(|pick| self.jobs.get(pick));
        let running = job.and_then(|job| job.running);
        let aborting = job.is_some_and(|job| job.aborting);
        let idle = running.is_none();
        let ended = |work: Work| job.and_then(|job| job.ended(work));
        let latest = picked == Some(&Pick::Latest);

        let keep = if latest {
            match (running, ended(Work::Save)) {
                (Some(Work::Save), _) if aborting => theme::sized_button("Aborting...", WIDTH, theme::danger_button),
                (Some(Work::Save), _) => theme::sized_button("Abort Save", WIDTH, theme::danger_button).on_press(Message::Abort),
                (_, Some(Ended::Completed)) => theme::sized_button("Saved!", WIDTH, theme::success_status),
                (_, Some(Ended::Failed)) => theme::sized_button(FAILURE_LABEL, WIDTH, theme::danger_status),
                (_, Some(Ended::Aborted)) => theme::sized_button("Save Terminated!", WIDTH, theme::danger_status),
                _ => theme::sized_button("Save to Disk", WIDTH, theme::primary_button).on_press_maybe((ready && idle).then_some(Message::Save)),
            }
        } else {
            match (running, ended(Work::Copy)) {
                (Some(Work::Copy), _) => theme::sized_button("Exporting...", WIDTH, theme::warning_status),
                (_, Some(Ended::Completed)) => theme::sized_button("Exported!", WIDTH, theme::success_status),
                (_, Some(_)) => theme::sized_button(FAILURE_LABEL, WIDTH, theme::danger_status),
                _ => theme::sized_button("Export to BCV", WIDTH, theme::primary_button).on_press_maybe(idle.then_some(Message::Copy)),
            }
        };

        let video_width = WIDTH - FORMAT_WIDTH - SPACING;
        let video = match (running, ended(Work::Video)) {
            (Some(Work::Video), _) if aborting => theme::sized_button("Aborting...", video_width, theme::danger_button),
            (Some(Work::Video), _) => theme::sized_button("Abort Export", video_width, theme::danger_button).on_press(Message::Abort),
            (_, Some(Ended::Completed)) => theme::sized_button("Exported!", video_width, theme::success_status),
            (_, Some(Ended::Failed)) => theme::sized_button(FAILURE_LABEL, video_width, theme::danger_status),
            (_, Some(Ended::Aborted)) => theme::sized_button("Export Terminated!", video_width, theme::danger_status),
            _ => theme::sized_button("Export to Video", video_width, theme::primary_button)
                .on_press_maybe((ready && self.ffmpeg && idle).then_some(Message::Export)),
        };
        let video: Element<'_, Message> = if self.ffmpeg { video.into() } else { hover_hint(video, NO_FFMPEG_HINT) };
        let format = pick_list(VideoFormat::ALL, Some(self.format), Message::Format)
            .width(Length::Fixed(FORMAT_WIDTH))
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu);

        let delete = theme::sized_button(self.deleting.confirm_label("Delete from Disk"), WIDTH, theme::danger_button)
            .on_press_maybe((!latest && idle).then_some(Message::Delete));
        let delete: Element<'_, Message> = if latest { hover_hint(delete, UNSAVED_HINT) } else { delete.into() };

        let (ratio, label) = job.map_or((1.0, READY_LABEL), |job| (job.status.ratio, job.status.label.as_str()));

        column![
            column![
                keep,
                row![video, format].spacing(SPACING).align_y(Vertical::Center),
                field("Quality %", QUALITY_HINT, &self.quality, Message::Quality),
                field("Compression %", COMPRESSION_HINT, &self.compression, Message::Compression),
                delete,
            ]
                .spacing(SPACING),
            column![theme::centered_text(label).size(STATUS_SIZE).width(Length::Fill), progress_bar(0.0..=1.0, ratio)].spacing(SPACING),
        ]
            .spacing(STATUS_GAP)
            .width(Length::Fixed(WIDTH))
            .into()
    }
}
