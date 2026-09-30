//! Map-wide and per-stage cost, music, and reward metadata.
//!
//! Two files are written in this layout. Every map outside the story chapters
//! declares its stages in a `MapStageData*.csv`, and the three story chapters
//! declare theirs in a `stageNormal*.csv`, which closes its table with a row
//! carrying nothing but the absent sentinel and documents its own columns in a
//! comment block below that row.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

use super::CostType;

/// Represents errors that can occur during the parsing of stage metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapStageDataError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for MapStageDataError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map stage data."
            ),
        }
    }
}

impl std::error::Error for MapStageDataError {}

/// The map-wide metadata carried by the two rows above a map's stage table.
///
/// The rows gained columns over the game's lifetime, so a file may stop short of
/// the full layout. A column the file omits keeps the value the engine assumes
/// in its absence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataHeader {
    /// The map number the engine files this map under.
    pub map_number: i32,
    /// The identifier of the map's item reward stage configuration, or negative one when it declares none.
    pub item_reward_setting: i32,
    /// The identifier of the map's score reward stage configuration, or negative one when it declares none.
    pub score_reward_setting: i32,
    /// The identifier of the map-wide clear condition, or negative one when there is none.
    pub map_condition: i32,
    /// The identifier of the per-stage clear condition, or negative one when there is none.
    pub stage_condition: i32,
    /// The user rank that must be reached before the map appears.
    pub user_rank_threshold: i32,
    /// How the stages of this map charge their entry cost.
    pub cost_type: CostType,
    /// Any trailing columns beyond the known layout, retained for forward compatibility.
    ///
    /// A column that does not read as an integer is held as `None` rather than
    /// discarded, so an element's index is always its offset past the known layout.
    pub rest: Vec<Option<i32>>,
    /// The map pattern declared by the second metadata row.
    pub map_pattern: i32,
}

impl MapStageDataHeader {
    /// The column mapping this parser applies to the first metadata row, in the
    /// order it applies it.
    ///
    /// Published so a consumer can read the layout of the row from the parser's
    /// own table instead of restating it. Columns past the table are kept in
    /// [`MapStageDataHeader::rest`], and [`MapStageDataHeader::map_pattern`]
    /// comes from the second row rather than from this table.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent -1;
        map_number           : 0;
        item_reward_setting  : 1;
        score_reward_setting : 2;
        map_condition        : 3;
        stage_condition      : 4;
        user_rank_threshold  : 5, Raw, 0;
        cost_type            : 6;
    };
}

impl Default for MapStageDataHeader {
    /// Produces the metadata the engine assumes when a file declares none.
    ///
    /// The identifier fields hold negative one rather than zero, matching the
    /// sentinel the raw columns use.
    fn default() -> Self {
        Self {
            map_number: -1,
            item_reward_setting: -1,
            score_reward_setting: -1,
            map_condition: -1,
            stage_condition: -1,
            user_rank_threshold: 0,
            cost_type: CostType::Energy,
            rest: Vec::new(),
            map_pattern: 0,
        }
    }
}

/// One possible item reward from clearing a stage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropReward {
    /// The weighting or percentage governing how often this reward is selected.
    pub chance: u32,
    /// The identifier of the item awarded.
    pub item_id: u32,
    /// The quantity of the item awarded.
    pub amount: u32,
}

/// A reward granted for completing a stage within a score threshold.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedScore {
    /// The score that must be reached to earn this reward.
    pub score: u32,
    /// The identifier of the item awarded.
    pub item_id: u32,
    /// The quantity of the item awarded.
    pub amount: u32,
}

/// The reward scheme a stage uses on completion.
///
/// The two schemes are mutually exclusive and select on different criteria, so a
/// variant each keeps the inapplicable fields unrepresentable.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RewardStructure {
    /// The stage grants no completion reward.
    #[default]
    None,
    /// The stage draws from a treasure pool.
    Treasure {
        /// The rule governing how many entries of the pool may be drawn and whether draws repeat.
        drop_rule: i32,
        /// The candidate rewards in the pool.
        drops: Vec<DropReward>,
    },
    /// The stage grants rewards according to the score achieved.
    Timed(Vec<TimedScore>),
}

/// The metadata describing a single stage's cost, music, and rewards.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataEntry {
    /// The cost of attempting the stage, in energy unless the chapter or this
    /// map's metadata packs a currency into it. Decode a packed value with
    /// [`resolve_energy`](super::resolve_energy).
    pub cost: u32,
    /// The experience awarded for clearing the stage.
    pub xp: u32,
    /// The identifier of the music track played from the start of the stage.
    pub init_track: u32,
    /// The base health percentage at which the music changes to the boss track.
    pub bgm_change_percent: u32,
    /// The identifier of the music track played after the change, or negative when there is none.
    pub boss_track: i16,
    /// The reward scheme the stage uses on completion.
    pub rewards: RewardStructure,
}

impl MapStageDataEntry {
    /// The column mapping this parser applies to one stage row, in the order it
    /// applies it.
    ///
    /// Published so a consumer can read the layout of the row from the parser's
    /// own table instead of restating it. Columns past the table carry
    /// [`MapStageDataEntry::rewards`], whose shape and length depend on which
    /// scheme the row declares, so the parser reads them separately.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        cost               : 0;
        xp                 : 1;
        init_track         : 2;
        bgm_change_percent : 3;
        boss_track         : 4;
    };
}

/// The parsed contents of a map's stage metadata file.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageData {
    /// The map-wide metadata the file declares above its stage table.
    pub header: MapStageDataHeader,
    /// The per-stage metadata in stage order, so an entry's position is its stage index.
    pub entries: Vec<MapStageDataEntry>,
}

impl MapStageData {
    /// Parses a map's stage metadata file into per-stage entries.
    ///
    /// The two metadata rows are read into the header first, after which each
    /// row is one stage in play order. The reward columns read as either a treasure pool or
    /// a score ladder according to the row's declared rule.
    ///
    /// An unreadable row yields a default entry rather than being dropped, so an
    /// entry's position is always its stage index. A row carrying nothing but the
    /// absent sentinel closes the table, and everything below it is left unread.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the map's `MapStageData*.csv` or `stageNormal*.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `MapStageData` on success, or a
    /// `MapStageDataError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, MapStageDataError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<MapStageData, MapStageDataError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut lines_iterator = file_content.lines();
    let header = extract_header(
        lines_iterator.next(),
        lines_iterator.next(),
        separator_char,
    );

    let mut parsed: Vec<(usize, MapStageDataEntry)> = Vec::new();
    let mut has_content = false;

    for (stage_index, file_line) in lines_iterator.enumerate() {
        let trimmed_line = strip_comment(file_line).trim();
        if trimmed_line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if is_terminator(&parts) {
            break;
        }

        has_content = true;

        if parts.len() < 2 {
            continue;
        }

        let mut entry = MapStageDataEntry::default();
        columns::apply(&parts, MapStageDataEntry::COLUMNS, &mut entry);

        let is_time_reward = parts.len() > 15 && parts[8..15].iter().all(|part| part.trim() == "-2");

        entry.rewards = if is_time_reward {
            extract_timed_scores(&parts)
        } else {
            extract_treasure_drops(&parts)
        };

        parsed.push((stage_index, entry));
    }

    if !has_content {
        return Err(MapStageDataError::EmptyFile);
    }

    let mut entries = Vec::new();
    if let Some((last_index, _)) = parsed.last() {
        entries.resize_with(last_index + 1, MapStageDataEntry::default);
        for (stage_index, entry) in parsed {
            if let Some(slot) = entries.get_mut(stage_index) {
                *slot = entry;
            }
        }
    }

    Ok(MapStageData { header, entries })
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(before_comment, _)| before_comment)
}

fn is_terminator(parts: &[&str]) -> bool {
    let mut written = parts.iter().map(|part| part.trim()).filter(|part| !part.is_empty());

    written.next() == Some("-1") && written.next().is_none()
}

fn extract_header(
    first_line: Option<&str>,
    second_line: Option<&str>,
    separator_char: char,
) -> MapStageDataHeader {
    let mut header = MapStageDataHeader::default();

    if let Some(line) = first_line {
        let parts: Vec<&str> = strip_comment(line).split(separator_char).collect();
        let past_table = columns::apply(&parts, MapStageDataHeader::COLUMNS, &mut header);

        header.rest = parts
            .iter()
            .skip(past_table)
            .map(|part| part.trim().parse::<i32>().ok())
            .collect();
    }

    if let Some(line) = second_line
        && let Some(value) = strip_comment(line)
            .split(separator_char)
            .next()
            .and_then(|part| part.trim().parse::<i32>().ok()) {
                header.map_pattern = value;
            }

    header
}

fn extract_timed_scores(parts: &[&str]) -> RewardStructure {
    let mut scores = Vec::new();
    let score_block_count = parts.len().saturating_sub(17) / 3;

    for block_index in 0..score_block_count {
        let mut score = 0;
        if let Some(val_string) = parts.get(16 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                score = parsed_value;
            }

        let mut item_id = 0;
        if let Some(val_string) = parts.get(17 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                item_id = parsed_value;
            }

        let mut amount = 0;
        if let Some(val_string) = parts.get(18 + block_index * 3)
            && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                amount = parsed_value;
            }

        scores.push(TimedScore { score, item_id, amount });
    }

    RewardStructure::Timed(scores)
}

fn extract_treasure_drops(parts: &[&str]) -> RewardStructure {
    if parts.len() < 8 {
        return RewardStructure::None;
    }

    let mut drops = Vec::new();

    let mut base_chance = 0;
    if let Some(val_string) = parts.get(5)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_chance = parsed_value;
        }

    let mut base_item_id = 0;
    if let Some(val_string) = parts.get(6)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_item_id = parsed_value;
        }

    let mut base_amount = 0;
    if let Some(val_string) = parts.get(7)
        && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
            base_amount = parsed_value;
        }

    drops.push(DropReward {
        chance: base_chance,
        item_id: base_item_id,
        amount: base_amount,
    });

    let is_multi_drop = parts.len() > 9;
    let mut drop_rule = 0;

    if is_multi_drop {
        if let Some(val_string) = parts.get(8)
            && let Ok(parsed_value) = val_string.trim().parse::<i32>() {
                drop_rule = parsed_value;
            }

        let drop_block_count = parts.len().saturating_sub(7) / 3;
        for block_index in 1..drop_block_count {
            let mut block_chance = 0;
            if let Some(val_string) = parts.get(6 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_chance = parsed_value;
                }

            let mut block_item_id = 0;
            if let Some(val_string) = parts.get(7 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_item_id = parsed_value;
                }

            let mut block_amount = 0;
            if let Some(val_string) = parts.get(8 + block_index * 3)
                && let Ok(parsed_value) = val_string.trim().parse::<u32>() {
                    block_amount = parsed_value;
                }

            drops.push(DropReward {
                chance: block_chance,
                item_id: block_item_id,
                amount: block_amount,
            });
        }
    }

    RewardStructure::Treasure { drop_rule, drops }
}
#[cfg(test)]
mod tests {

    #[test]
    fn every_header_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(MapStageDataHeader::COLUMNS);
    }

    #[test]
    fn every_entry_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(MapStageDataEntry::COLUMNS);
    }

    use super::*;

    const HEADERS: &str = "0,0,0\n0,0,0\n";

    #[test]
    fn blank_rows_do_not_shift_stage_indices() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n\n30,3000,1,0,-1\n");
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.entries.len(), 3);
        assert_eq!(data.entries[0].cost, 10);
        assert_eq!(data.entries[0].xp, 1000);
        assert_eq!(data.entries[1], MapStageDataEntry::default());
        assert_eq!(data.entries[2].cost, 30);
        assert_eq!(data.entries[2].xp, 3000);
    }

    #[test]
    fn trailing_blank_rows_are_not_padded() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n20,2000,1,0,-1\n\n\n");
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.entries.len(), 2);
        assert_eq!(data.entries[1].cost, 20);
    }

    #[test]
    fn dense_files_are_unaffected() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n20,2000,1,0,-1\n30,3000,1,0,-1\n");
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.entries.len(), 3);
        let energies: Vec<u32> = data.entries.iter().map(|entry| entry.cost).collect();
        assert_eq!(energies, vec![10, 20, 30]);
    }

    #[test]
    fn header_columns_are_read_from_the_first_two_rows() {
        let raw = "9,-1,-1,4,5,1600,1\n3\n10,1000,1,0,-1\n";
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.header.map_number, 9);
        assert_eq!(data.header.map_condition, 4);
        assert_eq!(data.header.stage_condition, 5);
        assert_eq!(data.header.user_rank_threshold, 1600);
        assert_eq!(data.header.cost_type, CostType::Item);
        assert_eq!(data.header.map_pattern, 3);
        assert!(data.header.rest.is_empty());
    }

    #[test]
    fn omitted_header_columns_keep_their_engine_defaults() {
        let raw = "19,-1,-1,-1,-1,    //comment\n1,\t//comment\n10,1000,1,0,-1\n";
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.header.map_number, 19);
        assert_eq!(data.header.map_condition, -1);
        assert_eq!(data.header.user_rank_threshold, 0);
        assert_eq!(data.header.cost_type, CostType::Energy);
        assert_eq!(data.header.map_pattern, 1);
    }

    #[test]
    fn a_single_column_header_still_parses() {
        let raw = "21,\t//comment\n0\n10,1000,1,0,-1\n";
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.header.map_number, 21);
        assert_eq!(data.header.item_reward_setting, -1);
        assert_eq!(data.header.stage_condition, -1);
        assert_eq!(data.entries.len(), 1);
    }

    #[test]
    fn trailing_header_columns_keep_their_positions() {
        let raw = "9,-1,-1,-1,-1,0,1,,42\n0\n10,1000,1,0,-1\n";
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.header.rest, vec![None, Some(42)]);
    }

    /// The opening of a real `stageNormal1_0.csv`, whose score ladder replaces
    /// the treasure pool, closed by the sentinel row and its comment block.
    const STORY_CHAPTER: &str = concat!(
        "-1,-1,-1,\t\t\t\t\t//マップ番号 0～16,　アイテム報酬型ステージ設定(-1:OFF),　スコア報酬型ステージ設定(-1:OFF)\n",
        "-1,\t\t\t\t\t\t\t\t//マップパターン 0～\n",
        "30,0,48,99,33,0,1,1,-2,-2,-2,-2,-2,-2,-2,1,8500,13,10,5000,3,1,-1,     //0沈没日本\n",
        "30,0,3,100,3,0,1,1,-2,-2,-2,-2,-2,-2,-2,1,8500,13,10,5000,6,25000,-1,     //1韓国\n",
        "-1,\n",
        "\n",
        "//消費統率力\t\t\t0\n",
        "//クリア時の獲得XP\t\t1\n",
    );

    #[test]
    fn a_story_chapter_reads_as_the_layout_it_shares() {
        let data = MapStageData::parse(STORY_CHAPTER, None).unwrap();

        assert_eq!(data.header.map_number, -1);
        assert_eq!(data.header.item_reward_setting, -1);
        assert_eq!(data.header.score_reward_setting, -1);
        assert_eq!(data.header.map_pattern, -1);

        assert_eq!(data.entries.len(), 2);
        assert_eq!(data.entries[0].cost, 30);
        assert_eq!(data.entries[0].init_track, 48);
        assert_eq!(data.entries[0].bgm_change_percent, 99);
        assert_eq!(data.entries[0].boss_track, 33);
        assert_eq!(
            data.entries[0].rewards,
            RewardStructure::Timed(vec![
                TimedScore { score: 8500, item_id: 13, amount: 10 },
                TimedScore { score: 5000, item_id: 3, amount: 1 },
            ]),
        );
        assert_eq!(data.entries[1].init_track, 3);
    }

    #[test]
    fn the_sentinel_row_closes_the_table() {
        let raw = format!("{HEADERS}10,1000,1,0,-1\n-1,\n30,3000,1,0,-1\n");
        let data = MapStageData::parse(raw, None).unwrap();

        assert_eq!(data.entries.len(), 1);
        assert_eq!(data.entries[0].cost, 10);
    }

    #[test]
    fn a_file_with_no_data_rows_is_an_error() {
        assert_eq!(MapStageData::parse(HEADERS, None).unwrap_err(), MapStageDataError::EmptyFile);
        assert_eq!(
            MapStageData::parse(format!("{HEADERS}-1,\n"), None).unwrap_err(),
            MapStageDataError::EmptyFile,
        );
    }
}
