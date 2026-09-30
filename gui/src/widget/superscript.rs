use iced::alignment::Vertical;
use iced::{Color, Element, Length, Theme};
use iced::font::{Font, Weight};
use iced::widget::{column, row, text, Row, Space};

use kore::common::superscript::{self, Segment};

const SUPERSCRIPT_SHRINK: f32 = 3.0;
const SUPERSCRIPT_ALPHA: f32 = 0.7;
const SUPERSCRIPT_SPACING: f32 = 1.25;

const BOLD: Font = Font { weight: Weight::Bold, ..Font::DEFAULT };

pub fn text_with_superscript<'a, Message: 'a>(raw_text: &str, text_size: f32) -> Element<'a, Message> {
    superscript_lines(raw_text, text_size, None, false)
}

pub(crate) fn tinted_superscript<'a, Message: 'a>(
    raw_text: &str,
    text_size: f32,
    tint: Option<Color>,
) -> Element<'a, Message> {
    superscript_lines(raw_text, text_size, tint, false)
}

pub(crate) fn strong_superscript<'a, Message: 'a>(
    raw_text: &str,
    text_size: f32,
    tint: Option<Color>,
) -> Element<'a, Message> {
    superscript_lines(raw_text, text_size, tint, true)
}

fn superscript_lines<'a, Message: 'a>(raw_text: &str, text_size: f32, tint: Option<Color>, bold: bool) -> Element<'a, Message> {
    let mut lines_col = column![];

    for line in raw_text.split('\n') {
        lines_col = lines_col.push(superscript_line(line, text_size, tint, bold));
    }

    lines_col.into()
}

fn superscript_line<'a, Message: 'a>(line: &str, text_size: f32, tint: Option<Color>, bold: bool) -> Element<'a, Message> {
    let plain = |body: &str| {
        let node = text(body.to_string()).size(text_size);
        let node = if bold { node.font(BOLD) } else { node };

        match tint {
            Some(color) => node.color(color),
            None => node,
        }
    };

    let mut result_row = row![].align_y(Vertical::Top);
    let mut has_content = false;

    for segment in superscript::segments(line) {
        result_row = match segment {
            Segment::Plain(body) => result_row.push(plain(body)),
            Segment::Raised(body) => push_superscript(result_row, has_content, body, text_size, tint),
        };
        has_content = true;
    }

    result_row.into()
}

fn push_superscript<'a, Message: 'a>(
    result_row: Row<'a, Message>,
    has_content: bool,
    super_str: &str,
    text_size: f32,
    tint: Option<Color>,
) -> Row<'a, Message> {
    let result_row = if has_content {
        result_row.push(Space::new().width(Length::Fixed(SUPERSCRIPT_SPACING)))
    } else {
        result_row
    };

    result_row.push(weak_superscript_text(super_str, text_size, tint))
}

fn weak_superscript_text<'a, Message: 'a>(super_str: &str, text_size: f32, tint: Option<Color>) -> Element<'a, Message> {
    let node = text(super_str.to_string()).size((text_size - SUPERSCRIPT_SHRINK).max(1.0));

    if let Some(color) = tint {
        return node.color(Color { a: SUPERSCRIPT_ALPHA, ..color }).into();
    }

    node
        .style(|theme: &Theme| text::Style {
            color: Some(Color { a: SUPERSCRIPT_ALPHA, ..theme.palette().text }),
        })
        .into()
}
