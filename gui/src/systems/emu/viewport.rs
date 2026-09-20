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

pub struct Viewport {
    frame: Rc<RefCell<EmuFrame>>,
    sheets: Rc<RefCell<SheetCache>>,
    design_height: f32,
    touches: TouchQueue,
    active: bool,
    covered: bool,
}

pub struct Scene {
    vertices: Vec<Vertex>,
    runs: Vec<(Option<Box<str>>, u32, u32)>,
    uploads: Vec<(Box<str>, Sheet)>,
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("vertices", &self.vertices.len())
            .field("runs", &self.runs.len())
            .finish()
    }
}

impl<Message> shader::Program<Message> for Viewport {
    type State = ();
    type Primitive = Scene;

    fn draw(&self, _state: &(), _cursor: mouse::Cursor, bounds: Rectangle) -> Scene {
        let frame = self.frame.borrow();

        if !self.active || frame.quads.is_empty() || bounds.height <= 0.0 {
            return Scene {
                vertices: Vec::new(),
                runs: Vec::new(),
                uploads: Vec::new(),
            };
        }

        let scale = if self.design_height > 0.0 {
            bounds.height / self.design_height
        } else {
            1.0
        };
        let sheets = self.sheets.borrow();
        let mut vertices: Vec<Vertex> = Vec::with_capacity(frame.quads.len() * 6);
        let mut runs: Vec<(Option<Box<str>>, u32, u32)> = Vec::new();
        let mut uploads: Vec<(Box<str>, Sheet)> = Vec::new();

        let mut absent: Vec<&str> = Vec::new();

        for quad in &frame.quads {
            let sheet = quad.sheet.as_ref().and_then(|name| {
                sheets.get(name.as_ref()).map(|sheet| (name.clone(), sheet))
            });

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
                    (point[0] * scale + bounds.x) / bounds.width * 2.0 - 1.0,
                    1.0 - (point[1] * scale + bounds.y) / bounds.height * 2.0,
                ]
            };
            let corner = |slot: usize| Vertex {
                position: clip(quad.corners[slot]),
                uv: uv[slot],
                color: quad.color,
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
                Some((last, _, last_end)) if *last == key => *last_end = end,
                _ => runs.push((key, start, end)),
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

    fn update(
        &self,
        _state: &mut (),
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        if !self.active {
            return None;
        }

        let scale = if self.design_height > 0.0 && bounds.height > 0.0 {
            bounds.height / self.design_height
        } else {
            1.0
        };
        let at = cursor.position_in(bounds).map(|point| {
            (
                (point.x / scale).round() as i32,
                (point.y / scale).round() as i32,
            )
        });

        if !self.covered && let Some((x, y)) = at {
            let touch = match event {
                iced::Event::Mouse(mouse::Event::CursorMoved { .. }) => Some(Touch::Moved { x, y }),
                iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    Some(Touch::Pressed { x, y })
                }
                iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    Some(Touch::Released)
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
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.active && self.covered {
            mouse::Interaction::Progress
        } else {
            mouse::Interaction::None
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
            pipeline.ensure_sheet(
                device,
                queue,
                name,
                sheet.width,
                sheet.height,
                &sheet.pixels,
            );
        }

        let runs = self
            .runs
            .iter()
            .map(|(sheet, start, end)| Run {
                sheet: sheet.clone(),
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

pub fn overlay<'a, Message: 'a>(
    base: Element<'a, Message>,
    frame: Option<&Rc<RefCell<EmuFrame>>>,
    sheets: Option<&Rc<RefCell<SheetCache>>>,
    covered: bool,
    design_height: f32,
    touches: Option<&TouchQueue>,
) -> Element<'a, Message> {
    let active = frame.is_some_and(|frame| !frame.borrow().quads.is_empty());
    let frame = frame.cloned().unwrap_or_default();
    let sheets = sheets.cloned().unwrap_or_default();
    let touches = touches.cloned().unwrap_or_else(super::input::queue);
    let painted = shader::Shader::new(Viewport {
        frame,
        sheets,
        design_height,
        touches,
        active,
        covered,
    })
    .width(Length::Fill)
    .height(Length::Fill);

    iced::widget::stack![base, painted].into()
}
