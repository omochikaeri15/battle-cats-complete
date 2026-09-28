use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Seeds {
    pub rng: u32,
    pub entropy: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
pub struct Screen {
    pub width: f32,
    pub height: f32,
    pub phone: bool,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Options {
    pub music: i32,
    pub effects: i32,
    pub two_rows: bool,
    pub vibrate: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(default)]
pub struct Stage {
    pub map_id: i32,
    pub stage: i32,
    pub layout: Option<i32>,
    pub crown: i32,
    pub map_name: String,
    pub stage_name: String,
    pub dungeon_stage: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(default)]
pub struct Tutorial {
    pub battle_cleared: bool,
    pub deck_seen: bool,
    pub two_rows_seen: bool,
    pub cat_god_seen: bool,
    pub shop_seen: bool,
}

impl Default for Tutorial {
    fn default() -> Self {
        Self { battle_cleared: true, deck_seen: true, two_rows_seen: true, cat_god_seen: true, shop_seen: true }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Unit {
    pub unit: i32,
    pub form: i32,
    pub level: u32,
    pub plus: u32,
    pub talents: Vec<(i32, i32)>,
    pub orbs: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Level {
    pub level: u32,
    pub plus: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Parts {
    pub cannon: i32,
    pub foundation: i32,
    pub style: i32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum God {
    Absent,
    #[default]
    Present,
    Discounted,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct Setup {
    pub stage: Stage,
    pub lineup: Vec<Unit>,
    pub tech: Vec<Level>,
    pub treasures: Vec<Vec<i32>>,
    pub cannon: i32,
    pub style: i32,
    pub foundation: i32,
    pub parts: BTreeMap<i32, Parts>,
    pub items: Vec<bool>,
    pub speed_engaged: bool,
    pub altar: Option<i32>,
    pub cat_god: God,
    pub tutorial: Tutorial,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct Save {
    pub version: String,
    pub app: String,
    pub seeds: Seeds,
    pub screen: Screen,
    pub options: Options,
    pub setup: Setup,
    pub icons: Vec<String>,
    pub costs: Vec<i32>,
    pub altar_cap: Option<i32>,
    pub terminated: bool,
}
