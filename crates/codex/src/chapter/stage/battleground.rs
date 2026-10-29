use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::{Column, FromColumn, Scale};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BattlegroundError {
    EmptyFile,
    MissingConfigLine,
}

impl fmt::Display for BattlegroundError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid battleground data."
            ),
            Self::MissingConfigLine => write!(
                formatter,
                "The file has a header but is missing the required config line."
            ),
        }
    }
}

impl std::error::Error for BattlegroundError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BossType {
    #[default]
    None,
    Boss,
    ScreenShake,
    Unknown(u32),
}

impl From<u32> for BossType {
    fn from(boss_val: u32) -> Self {
        match boss_val {
            0 => Self::None,
            1 => Self::Boss,
            2 => Self::ScreenShake,
            _ => Self::Unknown(boss_val),
        }
    }
}

impl FromColumn for BossType {
    fn from_column(text: &str) -> Option<Self> {
        text.parse::<u32>().ok().map(Self::from)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EnemyAmount {
    #[default]
    Infinite,
    Limit(u32),
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattlegroundEntry {
    pub enemy_id: u32,
    pub amount: EnemyAmount,
    pub start_frame: u32,
    pub respawn_min: u32,
    pub respawn_max: u32,
    pub base_hp_perc: u32,
    pub layer_min: i32,
    pub layer_max: i32,
    pub boss_type: BossType,
    pub magnification: u32,
    pub score: u32,
    pub atk_magnification: u32,
    pub time_flag: u32,
    pub kill_count: u32,
    pub is_base: bool,
}

impl BattlegroundEntry {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        start_frame   : 2, Double;
        respawn_min   : 3, Double;
        respawn_max   : 4, Double;
        base_hp_perc  : 5;
        layer_min     : 6;
        layer_max     : 7;
        boss_type     : 8;
        magnification : 9, Raw, 100;
        score         : 10;
        time_flag     : 12;
        kill_count    : 13;
    };
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Battleground {
    pub base_id: i32,
    pub width: u32,
    pub base_hp: u32,
    pub min_spawn: u32,
    pub max_spawn: u32,
    pub background_id: u32,
    pub max_enemies: u32,
    pub anim_base_id: u32,
    pub time_limit: u32,
    pub is_no_continues: bool,
    pub is_base_indestructible: bool,
    pub unknown_value: u32,
    pub entries: Vec<BattlegroundEntry>,
}

impl Battleground {
    pub const HEADER_COLUMNS: &'static [Column<Self>] = columns::columns! {
        base_id          : 0;
        is_no_continues  : 1;
    };

    pub const CONFIG_COLUMNS: &'static [Column<Self>] = columns::columns! {
        width                  : 0;
        base_hp                : 1;
        min_spawn              : 2;
        max_spawn              : 3, Raw, "";
        background_id          : 4;
        max_enemies            : 5;
        anim_base_id           : 6;
        time_limit             : 7;
        is_base_indestructible : 8;
        unknown_value          : 9;
    };

    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, BattlegroundError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Battleground, BattlegroundError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut clean_lines_iterator = file_content.lines().filter_map(|line| {
        let clean = line.split_once("//").map(|(before, _)| before).unwrap_or(line).trim();
        if clean.is_empty() { None } else { Some(clean) }
    });

    let Some(first_line) = clean_lines_iterator.next() else {
        return Err(BattlegroundError::EmptyFile);
    };

    let first_line_parts: Vec<&str> = first_line.split(separator_char).collect();

    let mut has_header = false;
    if first_line_parts.len() <= 7 {
        has_header = true;
    } else if let Some(part) = first_line_parts.get(6)
        && part.trim().is_empty() {
            has_header = true;
        }

    let mut ground = Battleground::default();

    let config_line = if has_header {
        columns::apply(&first_line_parts, Battleground::HEADER_COLUMNS, &mut ground);

        let Some(next_line) = clean_lines_iterator.next() else {
            return Err(BattlegroundError::MissingConfigLine);
        };
        next_line
    } else {
        first_line
    };

    let config_parts: Vec<&str> = config_line.split(separator_char).collect();
    columns::apply(&config_parts, Battleground::CONFIG_COLUMNS, &mut ground);

    if config_parts.get(3).and_then(|cell| cell.trim().parse::<u32>().ok()).is_none() {
        ground.max_spawn = ground.min_spawn;
    }

    for enemy_line in clean_lines_iterator {
        let enemy_parts: Vec<&str> = enemy_line.split(separator_char).collect();

        let raw_enemy_id = read_u32(&enemy_parts, 0);
        if raw_enemy_id == 0 {
            break;
        }

        let mut entry = BattlegroundEntry::default();
        columns::apply(&enemy_parts, BattlegroundEntry::COLUMNS, &mut entry);

        entry.enemy_id = raw_enemy_id.saturating_sub(2);
        entry.is_base = raw_enemy_id == ground.anim_base_id;

        let declared_amount = read_u32(&enemy_parts, 1);
        entry.amount = if declared_amount == 0 || entry.respawn_min == 0 {
            EnemyAmount::Infinite
        } else {
            EnemyAmount::Limit(declared_amount)
        };

        entry.atk_magnification = columns::parse_cell::<u32>(
            enemy_parts.get(11).copied(),
            "0",
            Scale::Raw,
        )
        .filter(|declared| *declared != 0)
        .unwrap_or(entry.magnification);

        if entry.enemy_id == 21 && entry.start_frame == 27000 {
            continue;
        }

        ground.entries.push(entry);
    }

    Ok(ground)
}

fn read_u32(row: &[&str], index: usize) -> u32 {
    columns::parse_cell(row.get(index).copied(), "0", Scale::Raw).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(Battleground::HEADER_COLUMNS);
        columns::assert_one_field_per_column(Battleground::CONFIG_COLUMNS);
        columns::assert_one_field_per_column(BattlegroundEntry::COLUMNS);
    }
}
