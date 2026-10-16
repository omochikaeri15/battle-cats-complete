use std::sync::Arc;

use iced::mouse;
use iced::wgpu;
use iced::widget::shader::{self, Shader};
use iced::window;
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};
use image::RgbaImage;

use nyanko::graphics::backdrop::{Backdrop, Camera, Quad, Screen, Surface};

use crate::widget::LINE_PIXELS;

use super::pipeline::{self, Pipeline, Scene};

pub(super) const ZOOM_MIN: f32 = 0.1;
pub(super) const ZOOM_MAX: f32 = 8.0;
const ZOOM_RATE_PER_PIXEL: f32 = 0.0016;
const HORIZON_SHARE: f32 = 0x208 as f32 / 0x280 as f32;
pub(super) const FRAME_ADVANCE_PER_TICK: f32 = 1.0;

#[derive(Debug, Clone)]
pub enum Message {
    Panned(Vector),
    Zoomed(f32, Point),
    Resized(Size),
    Tick,
}

fn zoom_delta(delta: &mouse::ScrollDelta) -> f32 {
    match delta {
        mouse::ScrollDelta::Lines { y, .. } => *y * LINE_PIXELS,
        mouse::ScrollDelta::Pixels { y, .. } => *y,
    }
}

pub struct State {
    pub center: f32,
    pub lift: f32,
    pub zoom: f32,
    pub viewport: Size,
    fitted: bool,
    pub current_frame: f32,
    pub is_playing: bool,
    pub loop_start: Option<f32>,
    pub loop_end: Option<f32>,
}

impl Default for State {
    fn default() -> Self {
        let screen = Screen::default();

        Self {
            center: screen.width as f32 / 2.0,
            lift: 0.0,
            zoom: 1.0,
            viewport: Size::new(screen.width as f32, screen.height as f32),
            fitted: false,
            current_frame: 0.0,
            is_playing: true,
            loop_start: None,
            loop_end: None,
        }
    }
}

impl State {
    pub fn camera(&self) -> Camera {
        Camera { x: self.center, y: self.lift, zoom: self.zoom, width: self.viewport.width, height: self.viewport.height }
    }

    pub fn update(&mut self, message: Message, screen: Screen, animated: bool) -> bool {
        match message {
            Message::Panned(delta) => {
                self.center -= delta.x / self.zoom.max(f32::EPSILON);
                self.lift += delta.y;
                true
            }
            Message::Zoomed(pixels, at) => {
                let factor = (pixels * ZOOM_RATE_PER_PIXEL).exp();
                let before = self.camera();
                let zoom = (self.zoom * factor).clamp(ZOOM_MIN, ZOOM_MAX);
                let horizon = screen.height as f32 * HORIZON_SHARE;
                let scene_x = (at.x - before.width / 2.0) / before.zoom.max(f32::EPSILON) + self.center;
                let scene_y = (at.y - before.horizon()) / before.zoom.max(f32::EPSILON) + horizon;

                self.zoom = zoom;
                self.center = scene_x - (at.x - before.width / 2.0) / zoom;
                self.lift = at.y - horizon - zoom * (scene_y - horizon);
                true
            }
            Message::Resized(size) => {
                if size.width < 1.0 || size.height < 1.0 {
                    return false;
                }

                self.viewport = size;

                if !self.fitted {
                    self.fitted = true;
                    self.lift = ((size.height - screen.height as f32) / 2.0).round();
                }

                true
            }
            Message::Tick => {
                if !self.is_playing || !animated {
                    return false;
                }

                self.current_frame += FRAME_ADVANCE_PER_TICK;

                if let Some(end) = self.loop_end
                    && self.current_frame > end
                {
                    self.current_frame = self.loop_start.unwrap_or(0.0);
                }

                false
            }
        }
    }

    pub fn view<'a>(&'a self, backdrop: Option<&'a Backdrop>, quads: &Arc<Vec<Quad>>, stamp: u64) -> Element<'a, Message> {
        Shader::new(Viewport { state: self, backdrop, quads: Arc::clone(quads), stamp })
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

struct Viewport<'a> {
    state: &'a State,
    backdrop: Option<&'a Backdrop>,
    quads: Arc<Vec<Quad>>,
    stamp: u64,
}

#[derive(Default)]
struct Interaction {
    drag_origin: Option<Point>,
}

impl shader::Program<Message> for Viewport<'_> {
    type State = Interaction;
    type Primitive = Painted;

    fn update(
        &self,
        interaction: &mut Interaction,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        match event {
            Event::Window(window::Event::RedrawRequested(_)) => {
                let size = bounds.size();

                (size != self.state.viewport).then(|| shader::Action::publish(Message::Resized(size)))
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let position = cursor.position_in(bounds)?;
                interaction.drag_origin = Some(position);
                Some(shader::Action::capture())
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                interaction.drag_origin.take().map(|_| shader::Action::capture())
            }
            Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let origin = interaction.drag_origin?;
                let position = Point::new(position.x - bounds.x, position.y - bounds.y);
                interaction.drag_origin = Some(position);
                let delta = position - origin;
                Some(shader::Action::publish(Message::Panned(delta)).and_capture())
            }
            Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let at = cursor.position_in(bounds)?;
                let pixels = zoom_delta(delta);

                if pixels == 0.0 {
                    return None;
                }

                Some(shader::Action::publish(Message::Zoomed(pixels, at)).and_capture())
            }
            _ => None,
        }
    }

    fn draw(&self, _interaction: &Interaction, _cursor: mouse::Cursor, _bounds: Rectangle) -> Painted {
        let Some(backdrop) = self.backdrop else {
            return Painted { quads: Arc::new(Vec::new()), images: Vec::new(), stamp: self.stamp };
        };
        let mut images: Vec<(Surface, Arc<RgbaImage>)> = Vec::new();

        for surface in self.quads.iter().filter_map(|quad| quad.surface) {
            if images.iter().any(|(held, _)| *held == surface) {
                continue;
            }

            if let Some(image) = backdrop.image(surface) {
                images.push((surface, Arc::clone(image)));
            }
        }

        Painted { quads: Arc::clone(&self.quads), images, stamp: self.stamp }
    }

    fn mouse_interaction(&self, interaction: &Interaction, bounds: Rectangle, cursor: mouse::Cursor) -> mouse::Interaction {
        if interaction.drag_origin.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) && self.backdrop.is_some() {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

pub struct Painted {
    quads: Arc<Vec<Quad>>,
    images: Vec<(Surface, Arc<RgbaImage>)>,
    stamp: u64,
}

impl std::fmt::Debug for Painted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Painted").field("quads", &self.quads.len()).finish()
    }
}

impl shader::Primitive for Painted {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        let physical = viewport.physical_size();
        let window = Size::new(physical.width as f32, physical.height as f32);

        if window.width <= 0.0 || window.height <= 0.0 || self.quads.is_empty() {
            pipeline.write(device, queue, &Scene::empty());
            return;
        }

        let factor = viewport.scale_factor();
        let key = pipeline::Key { stamp: self.stamp, bounds: *bounds, factor, window };

        if pipeline.holds(&key) {
            return;
        }

        let origin = Vector::new(bounds.x * factor, bounds.y * factor);
        let images = &self.images;
        let scene = pipeline::build(
            &self.quads,
            |surface| images.iter().find(|(held, _)| *held == surface).map(|(_, image)| Arc::clone(image)),
            |corner| {
                [
                    (corner[0] * factor + origin.x) / window.width * 2.0 - 1.0,
                    1.0 - (corner[1] * factor + origin.y) / window.height * 2.0,
                ]
            },
        );

        pipeline.write(device, queue, &scene);
        pipeline.hold(key);
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.draw(encoder, target, clip_bounds, None);
    }
}
