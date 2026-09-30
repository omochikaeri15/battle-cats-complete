//! Stage layout and enemy spawn configuration.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::{Column, FromColumn, Scale};

/// Represents errors that can occur during the parsing of a stage layout.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum BattlegroundError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
    /// The supplied bytes carried no leading row declaring the battlefield itself.
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

/// Selects the presentation treatment applied to an enemy on spawn.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum BossType {
    /// An ordinary enemy, spawned without announcement.
    #[default]
    None,
    /// A boss, announced with the boss entry effect.
    Boss,
    /// A boss whose entry additionally shakes the screen.
    ScreenShake,
    /// A treatment code this parser does not recognize, carrying its raw value.
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

/// The number of times an enemy may spawn over a stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum EnemyAmount {
    /// The enemy respawns for as long as the stage runs.
    #[default]
    Infinite,
    /// The enemy spawns at most the given number of times.
    Limit(u32),
}

/// One enemy's spawn configuration within a stage.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BattlegroundEntry {
    /// The identifier of the enemy this row spawns.
    pub enemy_id: u32,
    /// How many times the enemy may spawn.
    pub amount: EnemyAmount,
    /// The delay in frames before the enemy first becomes eligible to spawn.
    pub start_frame: u32,
    /// The shortest delay in frames between successive spawns.
    pub respawn_min: u32,
    /// The longest delay in frames between successive spawns.
    pub respawn_max: u32,
    /// The base health percentage below which the enemy begins spawning.
    pub base_hp_perc: u32,
    /// The lower bound of the random draw layer used to order overlapping sprites.
    pub layer_min: i32,
    /// The upper bound of the random draw layer used to order overlapping sprites.
    pub layer_max: i32,
    /// The presentation treatment applied when the enemy spawns.
    pub boss_type: BossType,
    /// The health scaling percentage applied to the enemy.
    pub magnification: u32,
    /// The score awarded for defeating the enemy.
    pub score: u32,
    /// The attack scaling percentage applied to the enemy, separate from its health scaling.
    pub atk_magnification: u32,
    /// The flag selecting whether `start_frame` is measured from the stage start or from the base being struck.
    pub time_flag: u32,
    /// The number of enemy defeats required before this row begins spawning.
    pub kill_count: u32,
    /// Whether this row describes the enemy base rather than a spawning enemy.
    pub is_base: bool,
}

impl BattlegroundEntry {
    /// The column mapping this parser applies to one enemy row, in the order it
    /// applies it.
    ///
    /// Published so a consumer can read the layout of the row from the parser's
    /// own table instead of restating it. The table covers the columns that
    /// depend on nothing but their own cell. [`BattlegroundEntry::enemy_id`],
    /// [`BattlegroundEntry::amount`],
    /// [`BattlegroundEntry::atk_magnification`] and
    /// [`BattlegroundEntry::is_base`] read columns 0, 1 and 11 against other
    /// columns of the row, so the parser derives them separately.
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

/// The complete layout and enemy roster of a single stage.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Battleground {
    /// The identifier of the enemy base defending the stage.
    pub base_id: i32,
    /// The horizontal extent of the battlefield in engine distance units.
    pub width: u32,
    /// The health pool of the enemy base.
    pub base_hp: u32,
    /// The shortest delay in frames between enemy spawn checks.
    pub min_spawn: u32,
    /// The longest delay in frames between enemy spawn checks.
    pub max_spawn: u32,
    /// The identifier of the background artwork the stage is drawn against.
    pub background_id: u32,
    /// The greatest number of enemies permitted on the field at once.
    pub max_enemies: u32,
    /// The identifier of the animation used for the enemy base.
    pub anim_base_id: u32,
    /// The stage's time limit in frames, or zero when it is untimed.
    pub time_limit: u32,
    /// Whether the player is barred from continuing after a loss.
    pub is_no_continues: bool,
    /// Whether the enemy base cannot be destroyed by ordinary means.
    pub is_base_indestructible: bool,
    /// A column present in the raw data whose effect on the engine is not established.
    pub unknown_value: u32,
    /// The enemy spawn configurations that make up the stage.
    pub entries: Vec<BattlegroundEntry>,
}

impl Battleground {
    /// The column mapping this parser applies to the optional leading row.
    ///
    /// A file that opens with a short row declares the base and the continue
    /// rule there, and the battlefield configuration follows on the next row.
    pub const HEADER_COLUMNS: &'static [Column<Self>] = columns::columns! {
        base_id          : 0;
        is_no_continues  : 1;
    };

    /// The column mapping this parser applies to the battlefield configuration row.
    ///
    /// [`Battleground::max_spawn`] falls back to
    /// [`Battleground::min_spawn`] rather than to a value of its own, so its
    /// entry declares no default and the parser supplies the fallback once the
    /// row has been read.
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

    /// Parses a stage layout file into its header and enemy spawn roster.
    ///
    /// The first row declares the battlefield itself and each subsequent row
    /// declares one enemy's spawn configuration. Column layouts vary between
    /// game versions, so absent trailing columns fall back to their engine
    /// defaults rather than rejecting the row.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the stage's `stage*.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `Battleground` on success, or a
    /// `BattlegroundError` if the file contained no parseable rows.
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
