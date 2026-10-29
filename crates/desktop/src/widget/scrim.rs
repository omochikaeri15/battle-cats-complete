use iced::mouse;
use iced::widget::canvas::{self, Frame, Geometry, Path};
use iced::{Color, Element, Length, Rectangle, Renderer, Theme};

const DIM: f32 = 0.55;

struct Scrim;

impl<Message> canvas::Program<Message> for Scrim {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let cover = Path::rectangle(iced::Point::ORIGIN, bounds.size());

        frame.fill(&cover, Color::from_rgba(0.0, 0.0, 0.0, DIM));

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut (),
        event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        matches!(
            event,
            iced::Event::Mouse(_) | iced::Event::Keyboard(_) | iced::Event::Touch(_)
        )
        .then(canvas::Action::capture)
    }

    fn mouse_interaction(
        &self,
        _state: &(),
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        mouse::Interaction::Idle
    }
}

pub(crate) fn scrim<'a, Message: 'a>() -> Element<'a, Message> {
    canvas::Canvas::new(Scrim)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
