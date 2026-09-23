use std::cell::RefCell;
use std::rc::Rc;

use iced::wgpu;
use iced::widget::shader;
use iced::{Element, Length, Rectangle, mouse};
use tracing::debug;

use super::assets::{Sheet, SheetCache};
use super::input::{Touch, TouchQueue};
use super::pipeline::{Pipeline, Run, Vertex};
use super::sink::Frame as EmuFrame;

const PIXELS_PER_LINE: f32 = 40.0;
const SPREAD_PER_LINE: f32 = 28.0;
const BAR_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

struct Fit {
    scale: f32,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

fn fit(bounds: Rectangle, design_width: f32, aspect: Option<f32>) -> Fit {
    let wide = if design_width > 0.0 { bounds.width / design_width } else { 1.0 };

    let Some(aspect) = aspect.filter(|aspect| *aspect > 0.0 && design_width > 0.0) else {
        return Fit { scale: wide, x: 0.0, y: 0.0, width: bounds.width, height: bounds.height };
    };

    let scale = wide.min(bounds.height / (design_width * aspect));
    let width = design_width * scale;
    let height = design_width * aspect * scale;

    Fit { scale, x: (bounds.width - width) / 2.0, y: (bounds.height - height) / 2.0, width, height }
}

pub struct Viewport {
    frame: Rc<RefCell<EmuFrame>>,
    sheets: Rc<RefCell<SheetCache>>,
    design_width: f32,
    touches: TouchQueue,
    active: bool,
    covered: bool,
    frozen: bool,
    aspect: Option<f32>,
}

#[derive(Default)]
pub struct Held {
    pressed: bool,
}

pub struct Scene {
    pub(super) vertices: Vec<Vertex>,
    pub(super) runs: Vec<(Option<Box<str>>, u8, u32, u32)>,
    pub(super) uploads: Vec<(Box<str>, Sheet)>,
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("vertices", &self.vertices.len())
            .field("runs", &self.runs.len())
            .finish()
    }
}

pub(super) fn paint(frame: &EmuFrame, sheets: &SheetCache, design_width: f32, aspect: Option<f32>, bounds: Rectangle) -> Scene {
    let fit = fit(bounds, design_width, aspect);
    let scale = fit.scale;
    let mut vertices: Vec<Vertex> = Vec::with_capacity(frame.quads.len() * 6);
    let mut runs: Vec<(Option<Box<str>>, u8, u32, u32)> = Vec::new();
    let mut uploads: Vec<(Box<str>, Sheet)> = Vec::new();

    let mut absent: Vec<&str> = Vec::new();

    for quad in &frame.quads {
        let label = quad.label.map(super::text::label_key);
        let named = quad.sheet.as_ref().or(label.as_ref());
        let sheet = named.and_then(|name| {
            sheets.get(name.as_ref()).map(|sheet| (name.clone(), sheet))
        });

        if label.is_some() && sheet.is_none() {
            continue;
        }

        if let Some(name) = quad.sheet.as_deref()
            && sheet.is_none()
            && !absent.contains(&name)
        {
            absent.push(name);
        }

        if let Some((name, sheet)) = &sheet
            && !uploads.iter().any(|(seen, _)| seen == name)
        {
            uploads.push((name.clone(), (*sheet).clone()));
        }

        let key = sheet.as_ref().map(|(name, _)| name.clone());
        let uv = sheet.as_ref().map_or([[0.0f32; 2]; 4], |(_, sheet)| {
            let width = sheet.width.max(1) as f32;
            let height = sheet.height.max(1) as f32;
            let left = quad.source[0] / width;
            let top = quad.source[1] / height;
            let right = (quad.source[0] + quad.source[2]) / width;
            let bottom = (quad.source[1] + quad.source[3]) / height;

            [[left, top], [left, bottom], [right, bottom], [right, top]]
        });

        let clip = |point: [f32; 2]| {
            [
                (point[0] * scale + fit.x + bounds.x) / bounds.width * 2.0 - 1.0,
                1.0 - (point[1] * scale + fit.y + bounds.y) / bounds.height * 2.0,
            ]
        };
        let corner = |slot: usize| Vertex {
            position: clip(quad.corners[slot]),
            uv: uv[slot],
            color: quad.colors[slot],
        };
        let start = vertices.len() as u32;

        vertices.extend([
            corner(0),
            corner(1),
            corner(2),
            corner(0),
            corner(2),
            corner(3),
        ]);

        let end = vertices.len() as u32;

        match runs.last_mut() {
            Some((last, blend, _, last_end)) if *last == key && *blend == quad.blend => {
                *last_end = end;
            }
            _ => runs.push((key, quad.blend, start, end)),
        }
    }

    if aspect.is_some() {
        let bars = [
            [0.0, 0.0, bounds.width, fit.y],
            [0.0, fit.y + fit.height, bounds.width, bounds.height - fit.y - fit.height],
            [0.0, fit.y, fit.x, fit.height],
            [fit.x + fit.width, fit.y, bounds.width - fit.x - fit.width, fit.height],
        ];
        let start = vertices.len() as u32;

        for [x, y, width, height] in bars.into_iter().filter(|bar| bar[2] > 0.0 && bar[3] > 0.0) {
            let corner = |cx: f32, cy: f32| Vertex {
                position: [
                    (cx + bounds.x) / bounds.width * 2.0 - 1.0,
                    1.0 - (cy + bounds.y) / bounds.height * 2.0,
                ],
                uv: [0.0, 0.0],
                color: BAR_COLOR,
            };

            vertices.extend([
                corner(x, y),
                corner(x, y + height),
                corner(x + width, y + height),
                corner(x, y),
                corner(x + width, y + height),
                corner(x + width, y),
            ]);
        }

        let end = vertices.len() as u32;

        if end > start {
            runs.push((None, 0, start, end));
        }
    }

    if !absent.is_empty() {
        debug!("emu: {} sheets drawn but never decoded: {absent:?}", absent.len());
    }

    Scene {
        vertices,
        runs,
        uploads,
    }
}

impl<Message> shader::Program<Message> for Viewport {
    type State = Held;
    type Primitive = Scene;

    fn draw(&self, _state: &Held, _cursor: mouse::Cursor, bounds: Rectangle) -> Scene {
        let frame = self.frame.borrow();

        if !self.active || frame.quads.is_empty() || bounds.width <= 0.0 {
            return Scene {
                vertices: Vec::new(),
                runs: Vec::new(),
                uploads: Vec::new(),
            };
        }

        paint(&frame, &self.sheets.borrow(), self.design_width, self.aspect, bounds)
    }

    fn update(
        &self,
        state: &mut Held,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        if !self.active {
            return None;
        }

        let scale = if self.design_width > 0.0 && bounds.width > 0.0 { bounds.width / self.design_width } else { 1.0 };
        let at = cursor.position_in(bounds).map(|point| {
            (
                (point.x / scale).round() as i32,
                (point.y / scale).round() as i32,
            )
        });

        if state.pressed {
            let lifted = matches!(
                event,
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    | iced::Event::Window(iced::window::Event::Unfocused)
            );

            if lifted {
                state.pressed = false;
                self.touches.borrow_mut().push(Touch::Released);

                return None;
            }

            let dragged = cursor.position().filter(|_| matches!(event, iced::Event::Mouse(mouse::Event::CursorMoved { .. })));

            if let Some(point) = dragged.filter(|_| at.is_none()) {
                let x = ((point.x - bounds.x).clamp(0.0, bounds.width) / scale).round() as i32;
                let y = ((point.y - bounds.y).clamp(0.0, bounds.height) / scale).round() as i32;

                self.touches.borrow_mut().push(Touch::Moved { x, y });

                return Some(shader::Action::capture());
            }
        }

        if !self.covered && !self.frozen && let Some((x, y)) = at {
            let touch = match event {
                iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(Touch::Moved { x, y }),
                iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    state.pressed = true;

                    Some(Touch::Pressed { x, y })
                }
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Touch::Released)
                }
                iced::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                    let lines = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => *y / PIXELS_PER_LINE,
                    };

                    Some(Touch::Pinched {
                        spread: (lines * SPREAD_PER_LINE) as i32,
                    })
                }
                _ => None,
            };

            if let Some(touch) = touch {
                self.touches.borrow_mut().push(touch);
            }
        }

        matches!(
            event,
            iced::Event::Mouse(_) | iced::Event::Keyboard(_) | iced::Event::Touch(_)
        )
        .then(shader::Action::capture)
    }

    fn mouse_interaction(
        &self,
        _state: &Held,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        match (self.active, self.covered) {
            (true, true) => mouse::Interaction::Progress,
            (true, false) => mouse::Interaction::Idle,
            (false, _) => mouse::Interaction::None,
        }
    }
}

impl shader::Primitive for Scene {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        for (name, sheet) in &self.uploads {
            pipeline.ensure_sheet(device, queue, name, sheet);
        }

        let runs = self
            .runs
            .iter()
            .map(|(sheet, blend, start, end)| Run {
                sheet: sheet.clone(),
                blend: *blend,
                range: *start..*end,
            })
            .collect();

        pipeline.write(device, queue, &self.vertices, runs);
    }

    fn render(
        &self,
        pipeline: &Pipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip_bounds: &Rectangle<u32>,
    ) {
        pipeline.draw(encoder, target, clip_bounds);
    }
}

pub struct Feed<'a> {
    pub frame: Option<&'a Rc<RefCell<EmuFrame>>>,
    pub sheets: Option<&'a Rc<RefCell<SheetCache>>>,
    pub touches: Option<&'a TouchQueue>,
    pub covered: bool,
    pub frozen: bool,
    pub design_width: f32,
    pub aspect: Option<f32>,
}

pub fn overlay<'a, Message: 'a>(
    base: Element<'a, Message>,
    above: Element<'a, Message>,
    feed: Feed<'_>,
) -> Element<'a, Message> {
    let Feed {
        frame,
        sheets,
        touches,
        covered,
        frozen,
        design_width,
        aspect,
    } = feed;
    let active = frame.is_some_and(|frame| !frame.borrow().quads.is_empty());
    let frame = frame.cloned().unwrap_or_default();
    let sheets = sheets.cloned().unwrap_or_default();
    let touches = touches.cloned().unwrap_or_else(super::input::queue);
    let painted = shader::Shader::new(Viewport {
        frame,
        sheets,
        design_width,
        touches,
        active,
        covered,
        frozen,
        aspect,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    iced::widget::stack![base, painted, above].into()
}
