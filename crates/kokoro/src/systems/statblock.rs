#[cfg(feature = "statblock")]
mod draw;
#[cfg(feature = "statblock")]
mod render;

use crate::Source;
use crate::systems::combat::AbilityItem;

#[cfg(feature = "statblock")]
pub use render::{Fonts, build_statblock_image};
#[cfg(feature = "disk")]
pub use render::save_to_disk;

#[derive(Clone)]
pub struct SpiritData {
    pub dmg_text: String,
    pub traits: Vec<AbilityItem>,
    pub h1: Vec<AbilityItem>,
    pub h2: Vec<AbilityItem>,
    pub b1: Vec<AbilityItem>,
    pub b2: Vec<AbilityItem>,
    pub footer: Vec<AbilityItem>,
}

#[derive(Clone)]
pub enum StatCell {
    Text(String),
    Frames(i32),
}

#[derive(Clone)]
pub struct StatblockData {
    pub is_cat: bool,
    pub id_str: String,
    pub name: String,
    pub icon: Option<Source>,
    pub top_label: String,
    pub top_value: String,
    pub headers_1: Vec<String>,
    pub data_1: Vec<StatCell>,
    pub headers_2: Vec<String>,
    pub data_2: Vec<StatCell>,
    pub traits: Vec<AbilityItem>,
    pub h1: Vec<AbilityItem>,
    pub h2: Vec<AbilityItem>,
    pub b1: Vec<AbilityItem>,
    pub b2: Vec<AbilityItem>,
    pub footer: Vec<AbilityItem>,
    pub spirit_data: Option<SpiritData>,
}
