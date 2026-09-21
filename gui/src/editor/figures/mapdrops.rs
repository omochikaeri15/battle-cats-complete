use std::sync::LazyLock;

use iced::Element;

use super::resolved::Rule;
use super::{Draft, Message};

const MAP_ID: usize = 0;
const CROWNS: usize = 4;
const STAGES: usize = 8;
const MATERIALS: usize = 8;

const CROWN_FIRST: usize = 1;
const STAGE_FIRST: usize = CROWN_FIRST + CROWNS;
const DUD: usize = STAGE_FIRST + STAGES;
pub(crate) const MATERIAL_FIRST: usize = DUD + 1;
const ZOMBIE_FIRST: usize = MATERIAL_FIRST + MATERIALS;

pub(crate) const WIDTH: usize = ZOMBIE_FIRST + MATERIALS;

const CROWN_PLACES: u32 = 2;
const CROWN_UNIT: i32 = 100;

const NOTICE: &str = concat!(
    "One row per map. The drop chances share a hundred with No Drop, and a stage's count is ",
    "multiplied by the crown difficulty's own figure before it is awarded",
);

const NOTICE_SIZE: f32 = 11.0;

static LABELS: LazyLock<Vec<String>> = LazyLock::new(|| {
    (0..WIDTH)
        .map(|index| match index {
            MAP_ID => "Map ID".to_owned(),
            _ if index < STAGE_FIRST => format!("{}\u{2605} Multiplier", index - CROWN_FIRST + 1),
            _ if index < DUD => format!("Stage {} Drops", index - STAGE_FIRST + 1),
            DUD => "No Drop %".to_owned(),
            _ if index < ZOMBIE_FIRST => format!("Material {} %", index - MATERIAL_FIRST + 1),
            _ => format!("Z Material {} %", index - ZOMBIE_FIRST + 1),
        })
        .collect()
});

pub(super) fn label(index: usize) -> &'static str {
    LABELS.get(index).map_or("Column", String::as_str)
}

pub(super) fn decimals(index: usize) -> u32 {
    match (CROWN_FIRST..STAGE_FIRST).contains(&index) {
        true => CROWN_PLACES,
        false => 0,
    }
}

pub(super) fn fallback(index: usize) -> i32 {
    match (CROWN_FIRST..STAGE_FIRST).contains(&index) {
        true => CROWN_UNIT,
        false => 0,
    }
}

pub(super) fn rule(index: usize) -> Rule {
    match index {
        MAP_ID => Rule::Floor(0),
        _ if index < STAGE_FIRST => Rule::Floor(0),
        _ if index < DUD => Rule::Floor(0),
        _ => Rule::Percent,
    }
}

pub(super) fn view<'a>(draft: &'a Draft, width: f32, query: &'a str, armed: bool) -> Element<'a, Message> {
    super::unitbuy::searched(draft, width, query, armed, Some(NOTICE), NOTICE_SIZE)
}

pub(super) const CHARA_WIDTH: usize = 3;

const CHARA_NOTICE: &str =
    "One row per unit a stage clear can unlock. The drop id is the item id the stage's reward block awards";

const CHARA_LABELS: [&str; CHARA_WIDTH] = ["Drop ID", "Gift Slot", "Unit ID"];

pub(super) fn chara_label(index: usize) -> &'static str {
    CHARA_LABELS.get(index).copied().unwrap_or("Column")
}

pub(super) fn chara_rule(index: usize) -> Rule {
    match index {
        0 => Rule::Floor(-1),
        _ => Rule::Floor(0),
    }
}

pub(super) fn chara_view<'a>(draft: &'a Draft, width: f32, armed: bool) -> Element<'a, Message> {
    super::unitbuy::searched(draft, width, "", armed, Some(CHARA_NOTICE), NOTICE_SIZE)
}
