use iced::alignment::Vertical;
use iced::widget::{row, text, text_input};
use iced::{Element, Length};

use crate::app::theme;

const LABEL_SPACING: f32 = 10.0;
const LABEL_WIDTH: f32 = 190.0;
const INPUT_WIDTH: f32 = 80.0;

pub fn entry_row<'a, Message: Clone + 'a>(
    label: &'a str,
    placeholder: &str,
    value: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(LABEL_WIDTH)),
        text_input(placeholder, value)
            .on_input(on_input)
            .width(Length::Fixed(INPUT_WIDTH))
            .style(theme::rounded_input),
    ]
        .spacing(LABEL_SPACING)
        .align_y(Vertical::Center)
        .into()
}
