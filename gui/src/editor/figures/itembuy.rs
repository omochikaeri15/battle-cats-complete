use iced::Element;

use super::resolved::Rule;
use super::{Draft, Message};

pub(crate) const ITEM_ID_COLUMN: usize = 3;

const NOTICE: &str = concat!(
    "One row per catalogue item, shared by every stage that drops it. The row's position ",
    "below the header is the line its name sits on in GatyaitemName",
);

const NOTICE_SIZE: f32 = 11.0;

pub(super) fn rule(field: &str) -> Option<Rule> {
    match field {
        "reflect_or_storage" => Some(Rule::Flag),

        "stage_drop_item_id" | "sever_id" | "src_item_id" | "gatya_ticket_id" | "img_id" => {
            Some(Rule::Floor(-1))
        }

        "rarity" | "price" | "quantity" | "category" | "index" | "main_menu_type" => Some(Rule::Floor(0)),

        _ => None,
    }
}

pub(super) fn view<'a>(draft: &'a Draft, width: f32, query: &'a str, armed: bool) -> Element<'a, Message> {
    super::unitbuy::bounded(draft, width, query, armed, Some(NOTICE), NOTICE_SIZE, draft.schema().known())
}
