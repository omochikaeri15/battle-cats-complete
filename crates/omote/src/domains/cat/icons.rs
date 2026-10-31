use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::common::assets;
use kokoro::common::formats::SpriteSheet;
use kokoro::systems::combat::registry::{AbilityIcon, get_fallback_by_icon};
use kokoro::systems::combat::{AbilityItem, CustomIcon};

use crate::systems::context::Shared;
use crate::systems::images::Pictures;
use crate::systems::picture::Pic;

pub(crate) const ICON_SIZE: f32 = 40.0;
pub(crate) const IMG015: &str = "img015";
pub(crate) const IMG022: &str = "img022";
const MISSING_ICON_ID: usize = 9999;

pub(crate) fn custom_bytes(icon: CustomIcon) -> Option<(&'static str, &'static [u8])> {
    match icon {
        CustomIcon::Multihit => Some(("multihit", assets::MULTIHIT)),
        CustomIcon::Kamikaze => Some(("kamikaze", assets::KAMIKAZE)),
        CustomIcon::BossWave => Some(("boss_wave", assets::BOSS_WAVE)),
        CustomIcon::Dojo => Some(("dojo", assets::DOJO)),
        CustomIcon::StarredAlien => Some(("starred_alien", assets::STARRED_ALIEN)),
        CustomIcon::Burrow => Some(("burrow", assets::BURROW)),
        CustomIcon::Revive => Some(("revive", assets::REVIVE)),
        CustomIcon::Stop => Some(("stop", assets::STOP)),
        CustomIcon::DeathTimer => Some(("death_timer", assets::DEATH_TIMER)),
        CustomIcon::God => Some(("god", assets::GOD)),
        CustomIcon::Unknown => Some(("unknown", assets::UNKNOWN)),
        CustomIcon::None => None,
    }
}

pub(crate) fn custom_url(pictures: &Pictures, icon: CustomIcon) -> Option<Rc<str>> {
    custom_bytes(icon).and_then(|(name, bytes)| pictures.asset(name, bytes))
}

pub(crate) fn sheet_url(pictures: &Pictures, name: &str, icon: usize) -> Option<Rc<str>> {
    pictures.cut(name, icon)
}

pub(crate) fn sheets(pictures: &Pictures, globals: &Shared, name: &str) -> Rc<Vec<SpriteSheet>> {
    pictures.sheets(&globals.0.vault().vfs, name)
}

pub(crate) fn fallback(text: &str, size: f32) -> Element {
    rsx! {
        div { class: "fallback-icon", style: "width: {size}px; height: {size}px;", "{text}" }
    }
}

pub(crate) fn image(url: Rc<str>, size: f32) -> Element {
    rsx! {
        Pic { url, class: "ability-icon", style: format!("width: {size}px; height: {size}px;") }
    }
}

pub(crate) fn item_icon(pictures: &Pictures, item: &AbilityItem, size: f32) -> Element {
    if item.custom_icon != CustomIcon::None
        && let Some(url) = custom_url(pictures, item.custom_icon)
    {
        return image(url, size);
    }

    if let Some(icon_id) = item.icon_id
        && let Some(url) = sheet_url(pictures, IMG015, icon_id)
    {
        let Some(border) = item.border_id.and_then(|border| sheet_url(pictures, IMG015, border)) else {
            return image(url, size);
        };

        return rsx! {
            div { class: "icon-stack", style: "width: {size}px; height: {size}px;",
                Pic { url }
                Pic { url: border }
            }
        };
    }

    let icon = if item.custom_icon != CustomIcon::None { AbilityIcon::Custom(item.custom_icon) } else { AbilityIcon::Standard(item.icon_id.unwrap_or(MISSING_ICON_ID)) };

    fallback(get_fallback_by_icon(icon), size)
}

pub(crate) fn ability_icon(pictures: &Pictures, icon: AbilityIcon, size: f32, fallback_text: &str) -> Element {
    let url = match icon {
        AbilityIcon::Custom(custom) => custom_url(pictures, custom),
        AbilityIcon::Standard(icon_id) => sheet_url(pictures, IMG015, icon_id),
        AbilityIcon::None => None,
    };

    url.map_or_else(|| fallback(fallback_text, size), |url| image(url, size))
}
