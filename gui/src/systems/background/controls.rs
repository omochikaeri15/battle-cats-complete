use iced::border::Radius;
use iced::widget::{button, column, container, mouse_area, opaque, pick_list, row, rule, text, text_input, Button, Container, TextInput};
use iced::{alignment, border, font, Background, Border, Color, Element, Font, Length, Padding, Theme};

use kore::systems::animation::export::ExportFormat;

use crate::app::theme;
use crate::common::feedback::FAILURE_LABEL;
use crate::common::fonts;
use crate::widget::{hover_hint, slide, Slide};

use super::canvas;

pub(crate) const HOLD_DELAY_SECS: f32 = 0.2;
const TICK_SECS: f32 = 1.0 / 30.0;

const TILE_HEIGHT: f32 = 28.0;
const GAP: f32 = 4.0;
const ROW_GAP: f32 = 3.0;
const ICON_W: f32 = 72.0;
const NAV_W: f32 = 30.0;
const FRAME_INPUT_W: f32 = 80.0;
const RANGE_W: f32 = 60.0;
const TILDE_W: f32 = 20.0;
const COL2_W: f32 = NAV_W * 2.0 + FRAME_INPUT_W + GAP * 2.0;
const COL3_W: f32 = 64.0;
const FORMAT_PADDING: [f32; 2] = [4.0, 6.0];
const INPUT_PADDING: [f32; 2] = [5.0, 6.0];
const DIVIDER_W: f32 = 10.0;
const PANEL_WIDTH: f32 = ICON_W + COL2_W + COL3_W + DIVIDER_W * 2.0 + GAP * 4.0;

const ACTION_W: f32 = 120.0;
const ACTION_H: f32 = 25.0;
const ACTION_GAP: f32 = 8.0;
const GRID_PAD: f32 = 2.0;

const TOGGLE_HEIGHT: f32 = 18.0;
const TOGGLE_TEXT_SIZE: f32 = 14.0;
const PLAY_TEXT_SIZE: f32 = 16.0;
const TILE_TEXT_SIZE: f32 = 14.0;

const PANEL_PAD: f32 = 8.0;
const PANEL_ALPHA: f32 = 160.0 / 255.0;
const PANEL_SHADE: f32 = 0.15;
const DISABLED_ALPHA: f32 = 0.5;

const PLAY_GLYPH: &str = "\u{25B6}";
const PAUSE_GLYPH: &str = "\u{25AE}\u{25AE}";

pub(super) const FORMAT_OPTIONS: [&str; 8] = ["GIF", "WebP", "AVIF", "PNG", "MP4", "MKV", "WebM", "ZIP"];

pub(super) const COPY_IDLE: &str = "Copy";
pub(super) const COPY_BUSY: &str = "Copying...";
pub(super) const COPY_DONE: &str = "Copied!";
pub(super) const EXPORT_IDLE: &str = "Export";
pub(super) const EXPORT_BUSY: &str = "Exporting...";
pub(super) const EXPORT_DONE: &str = "Exported!";
const ABORTED_LABEL: &str = "Aborted!";
const ENDLESS_LABEL: &str = "???";
const QUALITY_HINT: &str = "Export quality %";
const COMPRESSION_HINT: &str = "Export compression %";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feedback {
    Idle,
    Busy,
    Done(bool),
    Aborted,
}

pub(super) struct Panel<'a> {
    pub canvas: &'a canvas::State,
    pub loaded: bool,
    pub animated: bool,
    pub export_end: i32,
    pub format: &'a ExportFormat,
    pub formats: Vec<&'static str>,
    pub copy: Feedback,
    pub export: Feedback,
    pub expanded: bool,
}

#[derive(Default)]
pub struct State {
    range_start_input: String,
    range_end_input: String,
    quality_input: String,
    compression_input: String,
    hold_dir: i8,
    hold_timer: f32,
}

#[derive(Debug, Clone)]
pub enum Message {
    TogglePlay,
    ToggleExpanded,
    HoldStart(i8),
    HoldEnd,
    HoldCancel,
    FrameInputChanged(String),
    RangeStartChanged(String),
    RangeEndChanged(String),
    SetFormat(ExportFormat),
    QualityChanged(String),
    CompressionChanged(String),
    Copy,
    Export,
}

pub(super) fn format_label(format: &ExportFormat) -> &'static str {
    match format {
        ExportFormat::Gif => "GIF",
        ExportFormat::WebP => "WebP",
        ExportFormat::Avif => "AVIF",
        ExportFormat::Png => "PNG",
        ExportFormat::Mp4 => "MP4",
        ExportFormat::Mkv => "MKV",
        ExportFormat::Webm => "WebM",
        ExportFormat::Zip => "ZIP",
    }
}

pub(super) fn format_of(label: &str) -> ExportFormat {
    match label {
        "WebP" => ExportFormat::WebP,
        "AVIF" => ExportFormat::Avif,
        "PNG" => ExportFormat::Png,
        "MP4" => ExportFormat::Mp4,
        "MKV" => ExportFormat::Mkv,
        "WebM" => ExportFormat::Webm,
        "ZIP" => ExportFormat::Zip,
        _ => ExportFormat::Gif,
    }
}

fn frame_text(canvas: &canvas::State) -> String {
    (canvas.current_frame.trunc() as i32).to_string()
}

pub(super) fn frame_bounds(canvas: &canvas::State, animated: bool) -> (f32, Option<f32>) {
    if !animated {
        return (0.0, Some(0.0));
    }

    let min = canvas.loop_start.unwrap_or(0.0).max(0.0);
    let max = canvas.loop_end.map(|end| end.max(min));

    (min, max)
}

pub(super) fn clamp_frame(canvas: &mut canvas::State, animated: bool) {
    let (min, max) = frame_bounds(canvas, animated);

    canvas.current_frame = max.map_or(canvas.current_frame.max(min), |max| canvas.current_frame.clamp(min, max));
}

fn bold() -> font::Font {
    font::Font { weight: font::Weight::Bold, ..font::Font::DEFAULT }
}

fn panel_style(t: &Theme) -> container::Style {
    let palette = t.palette();
    let shade = |c: f32| c * PANEL_SHADE;

    container::Style {
        background: Some(Background::Color(Color {
            r: shade(palette.background.r),
            g: shade(palette.background.g),
            b: shade(palette.background.b),
            a: PANEL_ALPHA,
        })),
        border: Border {
            color: t.extended_palette().background.strong.color,
            width: 1.0,
            radius: Radius {
                top_left: theme::RADIUS_LG,
                top_right: theme::RADIUS_LG,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
        },
        ..container::Style::default()
    }
}

fn tile_style(t: &Theme) -> container::Style {
    let palette = t.extended_palette();

    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        border: Border { color: palette.background.strong.color, width: 1.0, radius: theme::RADIUS_SM.into() },
        ..container::Style::default()
    }
}

fn step_style(t: &Theme, enabled: bool) -> container::Style {
    let pair = t.extended_palette().background.strong;
    let alpha = if enabled { 1.0 } else { DISABLED_ALPHA };

    container::Style {
        background: Some(Background::Color(Color { a: alpha, ..pair.color })),
        text_color: Some(Color { a: alpha, ..pair.text }),
        border: border::rounded(theme::RADIUS_SM),
        ..container::Style::default()
    }
}

fn input_style(t: &Theme, status: text_input::Status) -> text_input::Style {
    text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border::default(),
        ..text_input::default(t, status)
    }
}

fn control_style(t: &Theme, status: button::Status) -> button::Style {
    let base = theme::toggle_button(t, status, false);

    if status != button::Status::Disabled {
        return base;
    }

    button::Style {
        background: base.background.map(|background| match background {
            Background::Color(color) => Background::Color(Color { a: color.a * DISABLED_ALPHA, ..color }),
            other => other,
        }),
        text_color: Color { a: DISABLED_ALPHA, ..base.text_color },
        ..base
    }
}

fn tile<'a>(width: f32, content: impl Into<Element<'a, Message>>) -> Container<'a, Message> {
    container(content)
        .width(Length::Fixed(width))
        .height(Length::Fixed(TILE_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(tile_style)
}

fn label_tile<'a>(width: f32, label: impl text::IntoFragment<'a>) -> Container<'a, Message> {
    tile(width, text(label).size(TILE_TEXT_SIZE))
}

fn tile_input<'a>(
    placeholder: &str,
    value: &str,
    enabled: bool,
    on_input: impl Fn(String) -> Message + 'a,
) -> TextInput<'a, Message> {
    let input = text_input(placeholder, value)
        .width(Length::Fill)
        .padding(0)
        .size(TILE_TEXT_SIZE)
        .align_x(alignment::Horizontal::Center)
        .style(input_style);

    if enabled { input.on_input(on_input) } else { input }
}

fn hinted_input<'a>(
    hint: &'a str,
    placeholder: &str,
    value: &'a str,
    enabled: bool,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let input = text_input(placeholder, value)
        .width(Length::Fixed(COL3_W))
        .padding(INPUT_PADDING)
        .size(TILE_TEXT_SIZE)
        .align_x(alignment::Horizontal::Center)
        .style(theme::rounded_input)
        .on_input_maybe(enabled.then_some(on_input));

    hover_hint(input, hint)
}

fn control_button<'a>(label: &'a str, font: Font, size: f32, width: f32, on_press: Option<Message>) -> Button<'a, Message> {
    button(text(label).font(font).size(size).width(Length::Fill).height(Length::Fill).center())
        .width(Length::Fixed(width))
        .height(Length::Fixed(TILE_HEIGHT))
        .padding(0)
        .on_press_maybe(on_press)
        .style(control_style)
}

fn feedback_button<'a>(idle: &'a str, busy: &'a str, done: &'a str, state: Feedback, press: Message) -> Element<'a, Message> {
    let (label, style): (&'a str, theme::ButtonStyleFn) = match state {
        Feedback::Idle => (idle, theme::primary_button),
        Feedback::Busy => (busy, theme::warning_status),
        Feedback::Done(true) => (done, theme::success_status),
        Feedback::Done(false) => (FAILURE_LABEL, theme::danger_status),
        Feedback::Aborted => (ABORTED_LABEL, theme::danger_status),
    };

    button(text(label).size(TILE_TEXT_SIZE).width(Length::Fill).height(Length::Fill).center())
        .width(Length::Fixed(ACTION_W))
        .height(Length::Fixed(ACTION_H))
        .padding(0)
        .on_press(press)
        .style(style)
        .into()
}

fn step_control<'a>(label: &'a str, direction: i8, enabled: bool) -> Element<'a, Message> {
    let face = container(text(label).font(fonts::MISC_SYMBOLS).size(TILE_TEXT_SIZE))
        .width(Length::Fixed(NAV_W))
        .height(Length::Fixed(TILE_HEIGHT))
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(move |t: &Theme| step_style(t, enabled));

    if !enabled {
        return face.into();
    }

    mouse_area(face)
        .on_press(Message::HoldStart(direction))
        .on_release(Message::HoldEnd)
        .on_exit(Message::HoldCancel)
        .into()
}

fn divider<'a>() -> Container<'a, Message> {
    container(rule::vertical(1))
        .width(Length::Fixed(DIVIDER_W))
        .height(Length::Fixed(TILE_HEIGHT * 2.0 + GAP))
        .align_x(alignment::Horizontal::Center)
}

fn digits_only(value: &str) -> bool {
    value.chars().all(|c| c.is_ascii_digit())
}

impl State {
    pub fn update(&mut self, message: Message, canvas: &mut canvas::State, animated: bool) {
        match message {
            Message::TogglePlay => canvas.is_playing = !canvas.is_playing,
            Message::HoldStart(dir) => {
                self.hold_dir = dir;
                self.hold_timer = 0.0;
            }
            Message::HoldEnd => {
                if self.hold_dir != 0 && self.hold_timer <= HOLD_DELAY_SECS {
                    self.step(canvas, animated, self.hold_dir as f32);
                }
                self.hold_dir = 0;
                self.hold_timer = 0.0;
            }
            Message::HoldCancel => {
                self.hold_dir = 0;
                self.hold_timer = 0.0;
            }
            Message::FrameInputChanged(value) => {
                let trimmed = value.trim();

                if !digits_only(trimmed) {
                    return;
                }

                let target = if trimmed.is_empty() { Some(0.0) } else { trimmed.parse::<f32>().ok() };

                if let Some(target) = target {
                    canvas.current_frame = target;
                    clamp_frame(canvas, animated);
                }
            }
            Message::RangeStartChanged(value) => {
                if !digits_only(&value) {
                    return;
                }

                if value.is_empty() {
                    canvas.loop_start = None;
                } else if let Ok(parsed) = value.parse::<f32>() {
                    canvas.loop_start = Some(parsed.max(0.0));

                    if canvas.current_frame < parsed {
                        canvas.current_frame = parsed;
                    }
                }

                self.range_start_input = value;
            }
            Message::RangeEndChanged(value) => {
                if !digits_only(&value) {
                    return;
                }

                if value.is_empty() {
                    canvas.loop_end = None;
                } else if let Ok(parsed) = value.parse::<f32>() {
                    canvas.loop_end = Some(parsed.max(0.0));

                    if canvas.current_frame > parsed {
                        canvas.current_frame = parsed;
                    }
                }

                self.range_end_input = value;
            }
            Message::QualityChanged(value) => self.quality_input = value,
            Message::CompressionChanged(value) => self.compression_input = value,
            Message::ToggleExpanded | Message::SetFormat(_) | Message::Copy | Message::Export => {}
        }
    }

    pub(super) fn set_quality_text(&mut self, text: String) {
        self.quality_input = text;
    }

    pub(super) fn set_compression_text(&mut self, text: String) {
        self.compression_input = text;
    }

    fn step(&mut self, canvas: &mut canvas::State, animated: bool, direction: f32) {
        let (min, max) = frame_bounds(canvas, animated);
        let stepped = canvas.current_frame + direction;

        canvas.current_frame = if stepped < min {
            max.unwrap_or(min)
        } else if max.is_some_and(|max| stepped > max) {
            min
        } else {
            stepped
        };
    }

    pub fn holding(&self) -> bool {
        self.hold_dir != 0
    }

    pub fn tick(&mut self, canvas: &mut canvas::State, animated: bool) {
        if self.hold_dir == 0 {
            self.hold_timer = 0.0;
            return;
        }

        self.hold_timer += TICK_SECS;

        if self.hold_timer <= HOLD_DELAY_SECS {
            return;
        }

        let delta = self.hold_dir as f32 * TICK_SECS * 30.0;

        self.step(canvas, animated, delta);
    }

    pub(super) fn view<'a>(&'a self, panel: Panel<'a>) -> Element<'a, Message> {
        let expand_icon = if panel.expanded { "▼" } else { "▲" };
        let toggle = button(
            text(expand_icon).size(TOGGLE_TEXT_SIZE).font(bold()).width(Length::Fill).height(Length::Fill).center(),
        )
        .width(Length::Fill)
        .height(Length::Fixed(TOGGLE_HEIGHT))
        .padding(0)
        .on_press(Message::ToggleExpanded)
        .style(button::text);

        let actions = row![
            feedback_button(COPY_IDLE, COPY_BUSY, COPY_DONE, panel.copy, Message::Copy),
            feedback_button(EXPORT_IDLE, EXPORT_BUSY, EXPORT_DONE, panel.export, Message::Export),
        ]
        .spacing(ACTION_GAP);

        let body = column![
            rule::horizontal(1),
            container(actions)
                .width(Length::Fill)
                .padding(Padding { top: GRID_PAD, right: 0.0, bottom: GRID_PAD, left: 0.0 })
                .align_x(alignment::Horizontal::Center),
            rule::horizontal(1),
            container(self.transport_row(&panel)).padding(Padding { top: GRID_PAD, right: 0.0, bottom: 0.0, left: 0.0 }),
        ]
        .spacing(ROW_GAP)
        .width(Length::Fill);

        let drawer = container(body).width(Length::Fill).padding(Padding { top: ROW_GAP, ..Padding::ZERO });
        let panel_column = column![toggle, slide(drawer, panel.expanded, Slide::Down)].width(Length::Fixed(PANEL_WIDTH));

        opaque(container(panel_column).padding(PANEL_PAD).style(panel_style))
    }

    fn transport_row<'a>(&'a self, panel: &Panel<'a>) -> Element<'a, Message> {
        let canvas = panel.canvas;
        let scrubbable = panel.loaded && panel.animated;
        let play_icon = if canvas.is_playing { PAUSE_GLYPH } else { PLAY_GLYPH };
        let selected = format_label(panel.format);
        let formats = pick_list(panel.formats.clone(), Some(selected), |label: &str| Message::SetFormat(format_of(label)))
            .width(Length::Fixed(ICON_W))
            .padding(FORMAT_PADDING)
            .text_size(TILE_TEXT_SIZE)
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu);

        let transport = column![
            control_button(play_icon, fonts::MISC_SYMBOLS, PLAY_TEXT_SIZE, ICON_W, scrubbable.then_some(Message::TogglePlay)),
            formats,
        ]
        .spacing(GAP);

        let end_hint = panel.export_end.to_string();
        let frame_row: Element<'a, Message> = if canvas.is_playing && scrubbable {
            row![
                tile(RANGE_W, tile_input("0", &self.range_start_input, scrubbable, Message::RangeStartChanged)),
                label_tile(TILDE_W, "~"),
                tile(RANGE_W, tile_input(&end_hint, &self.range_end_input, scrubbable, Message::RangeEndChanged)),
            ]
            .spacing(GAP)
            .into()
        } else {
            let frame_input = tile_input("0", &frame_text(canvas), scrubbable, Message::FrameInputChanged);

            row![
                step_control("◀", -1, scrubbable),
                tile(FRAME_INPUT_W, frame_input),
                step_control("▶", 1, scrubbable),
            ]
            .spacing(GAP)
            .into()
        };

        let shown_max = if scrubbable {
            canvas.loop_end.map_or_else(|| ENDLESS_LABEL.to_owned(), |end| (end.trunc() as i32).to_string())
        } else {
            "0".to_owned()
        };
        let counter_row = row![
            label_tile(RANGE_W, frame_text(canvas)),
            label_tile(TILDE_W, "/"),
            label_tile(RANGE_W, shown_max),
        ]
        .spacing(GAP);

        let playback = column![frame_row, counter_row].spacing(GAP);

        let output = column![
            hinted_input(QUALITY_HINT, "80", &self.quality_input, panel.loaded, Message::QualityChanged),
            hinted_input(COMPRESSION_HINT, "30", &self.compression_input, panel.loaded, Message::CompressionChanged),
        ]
        .spacing(GAP);

        row![transport, divider(), playback, divider(), output]
            .spacing(GAP)
            .align_y(alignment::Vertical::Center)
            .into()
    }
}
