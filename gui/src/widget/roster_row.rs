use iced::widget::image::Handle;
use iced::widget::{container, image as iced_image, mouse_area, row, tooltip};
use iced::{Element, Length};

use crate::common::row_window::ROW_HEIGHT;
use crate::widget::list_row;

const TOOLTIP_PADDING: f32 = 8.0;

pub(crate) fn roster_row<'a, Message: Clone + 'a>(
    handle: Handle,
    is_selected: bool,
    on_press: Message,
    on_down: Option<Message>,
    tooltip_content: Element<'a, Message>,
) -> Element<'a, Message> {
    let image = iced_image(handle).height(Length::Fixed(ROW_HEIGHT));
    let face: Element<'a, Message> = match on_down {
        Some(on_down) => mouse_area(row![image].width(Length::Fill)).on_press(on_down).into(),
        None => row![image].width(Length::Fill).into(),
    };
    let image_button = list_row(face, is_selected, false, Length::Fill, on_press);

    let tip = container(tooltip_content).padding(TOOLTIP_PADDING).style(container::bordered_box);

    tooltip(image_button, tip, tooltip::Position::Right).into()
}
