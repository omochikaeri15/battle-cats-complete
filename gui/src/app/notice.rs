use iced::widget::{button, column, markdown, scrollable, text, Space};
use iced::{Alignment, Element, Length, Size, Theme};

use crate::common::digest;
use crate::widget::{popup, smooth_scroll};

use super::Message;

const POPUP: popup::Spec = popup::Spec::new(popup::Kind::Notice, Size::new(500.0, 400.0));
const SCROLLBAR_GAP: f32 = 8.0;

// If "NOTICE_CONTENT" is empty, no notice appears
pub(super) const NOTICE_TITLE: &str = "NEW LOCALIZED FILES";
pub(super) const NOTICE_CONTENT: &str = r#"
As of v4.2.0 a ton of new localized files were added to the exceptions, and tons of pages were updated to read their contents instead of using hardcoded strings. Many localized files that were not caught by exceptions get imported as Thai only, which means the app may display only Thai to you. To fix your languages, go to `Settings > Import > Manage Exceptions` and click `Set to Default` if it isn't default already. Then delete your game data under `Settings > Files` and click `Delete "game"`. Finally, re-import under the Import page.
"#;

pub(super) fn parse_content() -> Vec<markdown::Item> {
    crate::common::markdown::parse(NOTICE_CONTENT)
}

pub(super) fn hash() -> String {
    digest::hash(&[NOTICE_TITLE, NOTICE_CONTENT])
}

pub(super) fn should_show(acknowledged: &[String]) -> bool {
    !NOTICE_CONTENT.trim().is_empty() && !acknowledged.contains(&hash())
}

pub(super) fn update(state: &mut popup::State, message: popup::Message) -> bool {
    state.update(message, POPUP)
}

pub(super) fn view<'a>(state: &'a popup::State, window: Size, theme: Theme, items: &'a [markdown::Item]) -> Element<'a, Message> {
    state.view(NOTICE_TITLE, POPUP, window, Message::Notice, move || {
        column![
            smooth_scroll(
                scrollable(
                    markdown::view(items, markdown::Settings::with_text_size(14.0, super::theme::markdown_style(&theme)))
                        .map(Message::OpenUrl)
                )
                    .height(Length::Fill)
                    .spacing(SCROLLBAR_GAP)
            ),
            Space::new().height(20.0),
            button(text("Acknowledge").size(16.0))
                .style(button::primary)
                .on_press(Message::AcknowledgeNotice),
        ]
        .align_x(Alignment::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(20.0)
        .into()
    }, None)
}

