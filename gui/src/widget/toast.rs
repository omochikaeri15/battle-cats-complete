use iced::alignment::{Horizontal, Vertical};
use iced::widget::{column, container, text};
use iced::{Element, Length, Padding, Theme};

use crate::app::theme;

use super::slide::{slide, Slide};

const OVERHANG: f32 = 4.0;
const PADDING_X: f32 = 7.0;
const PADDING_Y: f32 = 7.0;
const MESSAGE_SIZE: f32 = 13.0;
const DETAIL_SIZE: f32 = 12.0;
const HEADLINE_SIZE: f32 = 28.0;
const HEADLINE_PADDING_X: f32 = 32.0;
const HEADLINE_PADDING_TOP: f32 = 8.0;
const HEADLINE_PADDING_BOTTOM: f32 = 12.0;

struct Scale {
    message: f32,
    padding_x: f32,
    padding_top: f32,
    padding_bottom: f32,
}

const NORMAL: Scale = Scale { message: MESSAGE_SIZE, padding_x: PADDING_X, padding_top: PADDING_Y, padding_bottom: PADDING_Y };
const HEADLINE: Scale = Scale {
    message: HEADLINE_SIZE,
    padding_x: HEADLINE_PADDING_X,
    padding_top: HEADLINE_PADDING_TOP,
    padding_bottom: HEADLINE_PADDING_BOTTOM,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Tone {
    Alert,
    Hint,
}

pub(crate) fn toast<'a, Message: 'a>(message: &'a str, detail: Option<&'a str>, showing: bool, tone: Tone) -> Element<'a, Message> {
    build(message, detail, showing, tone, &NORMAL)
}

pub(crate) fn headline<'a, Message: 'a>(message: &'a str, showing: bool) -> Element<'a, Message> {
    build(message, None, showing, Tone::Alert, &HEADLINE)
}

fn build<'a, Message: 'a>(message: &'a str, detail: Option<&'a str>, showing: bool, tone: Tone, scale: &Scale) -> Element<'a, Message> {
    let mut said = column![theme::centered_text(message).size(scale.message)].align_x(Horizontal::Center);

    if let Some(detail) = detail {
        said = said.push(
            theme::centered_text(detail)
                .size(DETAIL_SIZE)
                .style(|theme: &Theme| text::Style { color: Some(theme::weak_text_color(theme)) }),
        );
    }

    let banner = container(said)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Center)
        .padding(Padding { top: OVERHANG + scale.padding_top, right: scale.padding_x, bottom: scale.padding_bottom, left: scale.padding_x })
        .style(match tone {
            Tone::Alert => theme::notice_banner,
            Tone::Hint => theme::hint_banner,
        });

    container(slide(banner, showing, Slide::Up).floating())
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Top)
        .padding(Padding::default().top(match tone {
            Tone::Alert => -OVERHANG,
            Tone::Hint => 0.0,
        }))
        .into()
}
