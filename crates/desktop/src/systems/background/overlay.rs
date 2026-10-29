use iced::mouse;
use iced::widget::canvas;
use iced::widget::canvas::{Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Renderer, Size, Theme};

use codex::graphics::backdrop::Camera;

use crate::widget::{toast, Tone};

const MIN_SELECTION_AREA: f32 = 25.0;
const HINT_TEXT: &str = "Right click & drag to set camera";
const DIM_ALPHA: f32 = 125.0 / 255.0;
const FILL_ALPHA: f32 = 30.0 / 255.0;
const DRAG_STROKE: f32 = 2.0;
const HELD_STROKE: f32 = 1.0;

#[derive(Default)]
pub struct State {
    pub selecting: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    Selected(Camera),
    Canceled,
}

impl State {
    pub fn view(&self, live: Camera, held: Option<Camera>) -> Element<'_, Message> {
        canvas(Selector { selecting: self.selecting, live, held }).width(Length::Fill).height(Length::Fill).into()
    }

    pub fn hint_view<'a, M: 'a>(&self) -> Element<'a, M> {
        toast(HINT_TEXT, None, self.selecting, Tone::Hint)
    }
}

fn dim_outside(frame: &mut canvas::Frame, size: Size, top_left: Point, box_size: Size, color: Color) {
    let left = top_left.x.clamp(0.0, size.width);
    let top = top_left.y.clamp(0.0, size.height);
    let right = (top_left.x + box_size.width).clamp(0.0, size.width);
    let bottom = (top_left.y + box_size.height).clamp(0.0, size.height);

    if top > 0.0 {
        frame.fill_rectangle(Point::ORIGIN, Size::new(size.width, top), color);
    }
    if bottom < size.height {
        frame.fill_rectangle(Point::new(0.0, bottom), Size::new(size.width, size.height - bottom), color);
    }
    if left > 0.0 {
        frame.fill_rectangle(Point::new(0.0, top), Size::new(left, bottom - top), color);
    }
    if right < size.width {
        frame.fill_rectangle(Point::new(right, top), Size::new(size.width - right, bottom - top), color);
    }
}

fn corners(start: Point, end: Point) -> (Point, Size) {
    (Point::new(start.x.min(end.x), start.y.min(end.y)), Size::new((end.x - start.x).abs(), (end.y - start.y).abs()))
}

struct Selector {
    selecting: bool,
    live: Camera,
    held: Option<Camera>,
}

#[derive(Default)]
struct Drag {
    anchor: Option<(Point, Point)>,
}

impl canvas::Program<Message> for Selector {
    type State = Drag;

    fn update(&self, drag: &mut Drag, event: &canvas::Event, bounds: Rectangle, cursor: mouse::Cursor) -> Option<canvas::Action<Message>> {
        if !self.selecting {
            return None;
        }

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                let position = cursor.position_in(bounds)?;

                drag.anchor = Some((position, position));
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                let (start, _) = drag.anchor?;

                drag.anchor = Some((start, Point::new(position.x - bounds.x, position.y - bounds.y)));
                Some(canvas::Action::capture())
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right)) => {
                let (start, end) = drag.anchor.take()?;
                let (top_left, size) = corners(start, end);
                let message = if size.width * size.height > MIN_SELECTION_AREA {
                    let scene_top_left = self.live.unproject([top_left.x, top_left.y]);
                    let scene_bottom_right = self.live.unproject([top_left.x + size.width, top_left.y + size.height]);

                    Message::Selected(Camera::framing(
                        scene_top_left,
                        [scene_bottom_right[0] - scene_top_left[0], scene_bottom_right[1] - scene_top_left[1]],
                    ))
                } else {
                    Message::Canceled
                };

                Some(canvas::Action::publish(message).and_capture())
            }
            _ => None,
        }
    }

    fn draw(&self, drag: &Drag, renderer: &Renderer, _theme: &Theme, bounds: Rectangle, _cursor: mouse::Cursor) -> Vec<Geometry> {
        if !self.selecting && self.held.is_none() {
            return Vec::new();
        }

        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let yellow = Color::from_rgb8(255, 255, 0);
        let dim = Color::from_rgba8(0, 0, 0, DIM_ALPHA);

        if self.selecting {
            if let Some((start, end)) = drag.anchor {
                let (top_left, size) = corners(start, end);

                dim_outside(&mut frame, bounds.size(), top_left, size, dim);
                frame.fill_rectangle(top_left, size, Color::from_rgba8(255, 255, 0, FILL_ALPHA));
                frame.stroke(&Path::rectangle(top_left, size), Stroke::default().with_color(yellow).with_width(DRAG_STROKE));
            } else {
                frame.fill_rectangle(Point::ORIGIN, bounds.size(), dim);
            }
        } else if let Some(held) = self.held {
            let min = self.live.project(held.unproject([0.0, 0.0]));
            let max = self.live.project(held.unproject([held.width, held.height]));
            let top_left = Point::new(min[0], min[1]);
            let size = Size::new(max[0] - min[0], max[1] - min[1]);

            dim_outside(&mut frame, bounds.size(), top_left, size, dim);
            frame.stroke(&Path::rectangle(top_left, size), Stroke::default().with_color(yellow).with_width(HELD_STROKE));
        }

        vec![frame.into_geometry()]
    }
}
