mod canvas;
mod controls;
mod export;
mod offscreen;
mod overlay;
mod pipeline;

use std::sync::Arc;
use std::time::Duration;

use iced::widget::{container, stack, text, Space};
use iced::{Background, Border, Color, Element, Length, Padding, Task, Theme};

use kore::domains::utilities::background::{self as builder, Loaded, STAGE_WIDTH};
use kore::systems::addons::paths::{self, Presence};
use nyanko::graphics::backdrop::{Backdrop, Camera, Layers, Quad, Screen};

use crate::app::state::BackgroundState;
use crate::editor;
use crate::widget::smooth_scroll;

pub const PLAYING_TICK: Duration = Duration::from_millis(33);
pub const RESTING_TICK: Duration = Duration::from_millis(200);

pub const DEFAULT_LOOP_END: i32 = 299;

const FRAME_BORDER_WIDTH: f32 = 4.0;
const FRAME_BORDER_RADIUS: f32 = 5.0;
const EMPTY_BACKGROUND_SHADE: f32 = 0.6;
const CONTROLS_INSET_LEFT: f32 = 7.0;
const ZOOM_SCROLL_STRENGTH: f32 = 2.5;
const PERCENT_MIN: i32 = 0;
const PERCENT_MAX: i32 = 100;
const DEFAULT_QUALITY: i32 = 80;
const DEFAULT_COMPRESSION: i32 = 30;
const REPLAY_CEILING: i32 = 100_000;

const EMPTY_NOTICE: &str = "Enter a Background ID to load a background";
const FFMPEG_MISSING_HINT: &str = "FFMPEG Addon missing, so file formats and export speeds may be limited";
const AVIFENC_MISSING_HINT: &str = "AVIFENC Addon missing, so AVIF file format is unavailable";
const ADDONS_HINT: &str = "Download and manage Addons under Settings > Addons";

fn frame_border<'a>() -> Element<'a, Message> {
    container(Space::new().width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|t: &Theme| container::Style {
            border: Border { color: t.palette().primary, width: FRAME_BORDER_WIDTH, radius: FRAME_BORDER_RADIUS.into() },
            ..container::Style::default()
        })
        .into()
}

struct Scene {
    backdrop: Backdrop,
    layers: Layers,
    seed: u32,
    animated: bool,
    id: u32,
}

#[derive(Default)]
pub struct State {
    scene: Option<Scene>,
    quads: Arc<Vec<Quad>>,
    stamp: u64,
    canvas: canvas::State,
    controls: controls::State,
    export: export::State,
    overlay: overlay::State,
    camera: Option<Camera>,
    formats: Vec<&'static str>,
    format_hint: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    Canvas(canvas::Message),
    Controls(controls::Message),
    Export(export::Message),
    Overlay(overlay::Message),
}

fn percentage(input: &str, empty: i32) -> Option<(i32, String)> {
    if !input.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }

    if input.trim().is_empty() {
        return Some((empty, String::new()));
    }

    let parsed = input.parse::<i32>().unwrap_or(PERCENT_MAX);
    let clamped = parsed.clamp(PERCENT_MIN, PERCENT_MAX);
    let text = if clamped == parsed { input.to_owned() } else { clamped.to_string() };

    Some((clamped, text))
}

fn available_formats(ffmpeg: bool, avif: bool) -> Vec<&'static str> {
    controls::FORMAT_OPTIONS
        .iter()
        .copied()
        .filter(|format| match *format {
            "AVIF" => avif,
            "MP4" | "MKV" | "WebM" | "PNG" => ffmpeg,
            _ => true,
        })
        .collect()
}

fn addon_hint(ffmpeg: bool, avif: bool) -> String {
    let mut lines = Vec::new();

    if !ffmpeg {
        lines.push(FFMPEG_MISSING_HINT);
    }

    if !avif {
        lines.push(AVIFENC_MISSING_HINT);
    }

    if !lines.is_empty() {
        lines.push(ADDONS_HINT);
    }

    lines.join("\n")
}

impl State {
    pub fn adopt(&mut self, loaded: Loaded, seed: u32, prefs: &BackgroundState) {
        let backdrop = Backdrop::new(loaded.assets, Screen::default(), STAGE_WIDTH, seed);
        let animated = backdrop.animated();

        self.scene = Some(Scene { backdrop, layers: Layers::default(), seed, animated, id: loaded.id });
        self.canvas.current_frame = 0.0;
        self.canvas.loop_start = None;
        self.canvas.loop_end = None;
        self.controls = controls::State::default();
        self.controls.set_quality_text(prefs.last_export_quality.map_or_else(String::new, |value| value.to_string()));
        self.controls.set_compression_text(prefs.last_export_compression.map_or_else(String::new, |value| value.to_string()));
        self.refresh_addons();
        self.refresh();
    }

    fn refresh_addons(&mut self) {
        let ffmpeg = paths::ffmpeg_status() == Presence::Installed;
        let avif = paths::avifenc_status() == Presence::Installed;

        self.formats = available_formats(ffmpeg, avif);
        self.format_hint = addon_hint(ffmpeg, avif);
    }

    pub fn clear(&mut self) {
        self.scene = None;
        self.quads = Arc::new(Vec::new());
        self.stamp = self.stamp.wrapping_add(1);
        self.canvas.current_frame = 0.0;
    }

    pub fn set_seed(&mut self, seed: u32) {
        let Some(scene) = self.scene.as_mut() else {
            return;
        };

        if scene.seed == seed {
            return;
        }

        scene.seed = seed;
        self.sync_frame(true);
    }

    pub fn pace(&self) -> Duration {
        let animated = self.scene.as_ref().is_some_and(|scene| scene.animated);
        let live = (animated && self.canvas.is_playing) || self.controls.holding() || self.export.busy();

        if live { PLAYING_TICK } else { RESTING_TICK }
    }

    fn animated(&self) -> bool {
        self.scene.as_ref().is_some_and(|scene| scene.animated)
    }

    pub fn tick(&mut self) {
        let animated = self.animated();

        self.canvas.update(canvas::Message::Tick, Screen::default(), animated);
        self.controls.tick(&mut self.canvas, animated);
        self.sync_frame(false);
    }

    fn refresh(&mut self) {
        if let Some(scene) = self.scene.as_mut() {
            scene.layers = scene.backdrop.layers();
        }

        self.recompose();
    }

    fn recompose(&mut self) {
        let camera = self.canvas.camera();
        let quads = self.scene.as_ref().map_or_else(Vec::new, |scene| scene.backdrop.compose(&scene.layers, camera));

        self.quads = Arc::new(quads);
        self.stamp = self.stamp.wrapping_add(1);
    }

    fn capture(&self) -> (Camera, u32, u32) {
        let even = |length: f32| ((length.round().max(2.0) as u32) / 2 * 2).max(2);
        let camera = self.camera.unwrap_or_else(|| self.canvas.camera().rescaled(1.0));

        (camera, even(camera.width), even(camera.height))
    }

    fn sync_frame(&mut self, force: bool) {
        let Some(scene) = self.scene.as_mut() else {
            return;
        };
        let target = (self.canvas.current_frame.floor().max(0.0) as i32).min(REPLAY_CEILING);
        let held = scene.backdrop.frame();

        if held == target && !force {
            return;
        }

        if force || target < held {
            scene.backdrop.restart(scene.seed);
        }

        while scene.backdrop.frame() < target {
            scene.backdrop.tick();
        }

        self.refresh();
    }

    fn export_range(&self) -> (i32, i32) {
        if !self.animated() {
            return (0, 0);
        }

        let (start, end) = controls::frame_bounds(&self.canvas, true);

        (start.floor() as i32, end.unwrap_or(DEFAULT_LOOP_END as f32).floor() as i32)
    }

    pub fn update(&mut self, message: Message, prefs: &mut BackgroundState) -> Task<Message> {
        match message {
            Message::Canvas(msg) => {
                let animated = self.animated();

                if self.canvas.update(msg, Screen::default(), animated) {
                    self.recompose();
                }

                Task::none()
            }
            Message::Controls(controls::Message::ToggleExpanded) => {
                prefs.controls_expanded = !prefs.controls_expanded;
                self.refresh_addons();
                Task::none()
            }
            Message::Controls(controls::Message::SetCamera) => {
                self.overlay.selecting = true;
                Task::none()
            }
            Message::Controls(controls::Message::ClearCamera) => {
                self.camera = None;
                Task::none()
            }
            Message::Overlay(msg) => {
                if let overlay::Message::Selected(camera) = msg {
                    self.camera = Some(camera);
                }

                self.overlay.selecting = false;
                Task::none()
            }
            Message::Controls(controls::Message::SetFormat(format)) => {
                prefs.last_export_format = format;
                Task::none()
            }
            Message::Controls(controls::Message::QualityChanged(value)) => {
                let Some((percent, text)) = percentage(&value, DEFAULT_QUALITY) else {
                    return Task::none();
                };

                prefs.last_export_quality = (!text.is_empty()).then_some(percent);
                self.controls.set_quality_text(text);
                Task::none()
            }
            Message::Controls(controls::Message::CompressionChanged(value)) => {
                let Some((percent, text)) = percentage(&value, DEFAULT_COMPRESSION) else {
                    return Task::none();
                };

                prefs.last_export_compression = (!text.is_empty()).then_some(percent);
                self.controls.set_compression_text(text);
                Task::none()
            }
            Message::Controls(controls::Message::Copy) => {
                if self.export.copy_feedback() == controls::Feedback::Busy {
                    return self.export.abort_copy().map(Message::Export);
                }

                let Some(scene) = &self.scene else {
                    return Task::none();
                };
                let frame = if scene.animated { self.canvas.current_frame.floor().max(0.0) as i32 } else { 0 };
                let (camera, width, height) = self.capture();

                self.export.begin_copy(scene.backdrop.clone(), scene.seed, frame, camera, (width, height)).map(Message::Export)
            }
            Message::Controls(controls::Message::Export) => {
                if self.export.export_feedback() == controls::Feedback::Busy {
                    self.export.abort_export();
                    return Task::none();
                }

                let Some(scene) = &self.scene else {
                    return Task::none();
                };
                let (start, end) = self.export_range();
                let (camera, width, height) = self.capture();
                let request = export::Request {
                    backdrop: scene.backdrop.clone(),
                    seed: scene.seed,
                    start,
                    end,
                    stem: builder::export_stem(scene.id),
                    format: prefs.last_export_format.clone(),
                    quality: prefs.last_export_quality.unwrap_or(DEFAULT_QUALITY) as u32,
                    compression: prefs.last_export_compression.unwrap_or(DEFAULT_COMPRESSION) as u32,
                    camera,
                    width,
                    height,
                };

                self.export.begin_export(request).map(Message::Export)
            }
            Message::Controls(msg) => {
                let animated = self.animated();

                self.controls.update(msg, &mut self.canvas, animated);
                self.sync_frame(false);
                Task::none()
            }
            Message::Export(msg) => self.export.update(msg).map(Message::Export),
        }
    }

    pub fn view<'a>(&'a self, prefs: &'a BackgroundState) -> Element<'a, Message> {
        let Some(scene) = &self.scene else {
            let notice = container(text(EMPTY_NOTICE))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .style(|t: &Theme| {
                    let palette = t.palette();
                    let shade = |c: f32| c * EMPTY_BACKGROUND_SHADE;

                    container::Style {
                        background: Some(Background::Color(Color {
                            r: shade(palette.background.r),
                            g: shade(palette.background.g),
                            b: shade(palette.background.b),
                            a: palette.background.a,
                        })),
                        ..container::Style::default()
                    }
                });

            return stack![notice, frame_border()].into();
        };

        let viewport = container(
            smooth_scroll(self.canvas.view(Some(&scene.backdrop), &self.quads, self.stamp).map(Message::Canvas))
                .strength(ZOOM_SCROLL_STRENGTH),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(FRAME_BORDER_WIDTH);
        let shown = self.camera.filter(|_| prefs.controls_expanded);
        let selection = container(self.overlay.view(self.canvas.camera(), shown).map(Message::Overlay))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(FRAME_BORDER_WIDTH);
        let panel = controls::Panel {
            canvas: &self.canvas,
            loaded: true,
            animated: scene.animated,
            export_end: DEFAULT_LOOP_END,
            format: &prefs.last_export_format,
            formats: &self.formats,
            format_hint: &self.format_hint,
            camera: self.camera.is_some(),
            copy: self.export.copy_feedback(),
            export: self.export.export_feedback(),
            expanded: prefs.controls_expanded,
        };
        let controls = container(self.controls.view(panel).map(Message::Controls))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(iced::alignment::Horizontal::Left)
            .align_y(iced::alignment::Vertical::Bottom)
            .padding(Padding { left: CONTROLS_INSET_LEFT, ..Padding::ZERO });

        let layers = stack![viewport, selection, self.overlay.hint_view(), controls, frame_border()];

        editor::suppress(container(layers).width(Length::Fill).height(Length::Fill), self.overlay.selecting)
    }
}


