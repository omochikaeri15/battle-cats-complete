use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

use super::CostType;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapStageDataError {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataHeader {
    pub map_number: i32,
    pub item_reward_setting: i32,
    pub score_reward_setting: i32,
    pub map_condition: i32,
    pub stage_condition: i32,
    pub user_rank_threshold: i32,
    pub cost_type: CostType,
    pub rest: Vec<Option<i32>>,
    pub map_pattern: i32,
}

impl MapStageDataHeader {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DropReward {
    pub chance: u32,
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimedScore {
    pub score: u32,
    pub item_id: u32,
    pub amount: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct DropOdds {
    pub first: f64,
    pub repeat: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RewardStructure {
    #[default]
    None,
    Treasure {
        drop_rule: i32,
        drops: Vec<DropReward>,
    },
    Timed {
        drop: DropReward,
        scores: Vec<TimedScore>,
    },
}

impl RewardStructure {
    pub fn drops(&self) -> &[DropReward] {
        match self {
            Self::None => &[],
            Self::Treasure { drops, .. } => drops,
            Self::Timed { drop, .. } => std::slice::from_ref(drop),
        }
    }

    pub fn odds(&self) -> Vec<DropOdds> {
        let drops = self.drops();
        let rule = match self {
            Self::Treasure { drop_rule, .. } => *drop_rule,
            _ => -2,
        };

        if rule == -3 || rule == -4 {
            return weighted_odds(drops, rule == -4);
        }

        let tiers = if rule >= 0 { drops.len().min(3) } else { drops.len().min(1) };
        let once = rule == 1 || rule == 2;
        let mut first_miss = 1.0;
        let mut repeat_miss = 1.0;

        drops
            .iter()
            .enumerate()
            .map(|(tier, drop)| {
                if tier >= tiers {
                    return DropOdds::default();
                }

                let rate = f64::from(drop.chance.min(100)) / 100.0;
                let repeat_rate = if once && tier == 0 { 0.0 } else { rate };
                let odds = DropOdds { first: first_miss * rate * 100.0, repeat: repeat_miss * repeat_rate * 100.0 };

                first_miss *= 1.0 - rate;
                repeat_miss *= 1.0 - repeat_rate;

                odds
            })
            .collect()
    }
}

fn weighted_odds(drops: &[DropReward], repeats: bool) -> Vec<DropOdds> {
    let total: u64 = drops.iter().map(|drop| u64::from(drop.chance)).sum();

    if total == 0 || drops.len() >= 11 {
        return vec![DropOdds::default(); drops.len()];
    }

    drops
        .iter()
        .map(|drop| {
            let first = f64::from(drop.chance) * 100.0 / total as f64;

            DropOdds { first, repeat: if repeats { first } else { 0.0 } }
        })
        .collect()
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageDataEntry {
    pub cost: u32,
    pub xp: u32,
    pub init_track: u32,
    pub bgm_change_percent: u32,
    pub boss_track: i16,
    pub rewards: RewardStructure,
}

impl MapStageDataEntry {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        cost               : 0;
        xp                 : 1;
        init_track         : 2;
        bgm_change_percent : 3;
        boss_track         : 4;
    };
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapStageData {
    pub header: MapStageDataHeader,
    pub entries: Vec<MapStageDataEntry>,
}

impl MapStageData {
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
    let drop = DropReward {
        chance: parse_cell(parts, 5),
        item_id: parse_cell(parts, 6),
        amount: parse_cell(parts, 7),
    };
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

    RewardStructure::Timed { drop, scores }
}

fn parse_cell(parts: &[&str], index: usize) -> u32 {
    parts.get(index).and_then(|part| part.trim().parse().ok()).unwrap_or(0)
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
            RewardStructure::Timed {
                drop: DropReward { chance: 0, item_id: 1, amount: 1 },
                scores: vec![
                    TimedScore { score: 8500, item_id: 13, amount: 10 },
                    TimedScore { score: 5000, item_id: 3, amount: 1 },
                ],
            },
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

    fn odds_of(row: &str) -> Vec<(f64, f64)> {
        let data = MapStageData::parse(format!("{HEADERS}{row}\n"), None).unwrap();

        data.entries[0].rewards.odds().iter().map(|odds| (odds.first, odds.repeat)).collect()
    }

    // A guaranteed pool picks one entry by weight, it doesn't hand out every entry at 100%.
    #[test]
    fn a_guaranteed_pool_splits_one_drop_by_weight() {
        assert_eq!(odds_of("30,0,1,1,1,25,10,1,-4,75,11,1,-1"), vec![(25.0, 25.0), (75.0, 75.0)]);
        assert_eq!(odds_of("30,0,1,1,1,25,10,1,-3,75,11,1,-1"), vec![(25.0, 0.0), (75.0, 0.0)]);
    }

    // Rules 0-2 roll each slot in turn and stop at the first hit; 1 skips slot one once it's claimed.
    #[test]
    fn a_chained_pool_rolls_each_slot_after_the_last_misses() {
        assert_eq!(odds_of("30,0,1,1,1,50,10,1,0,50,11,1,100,12,1,-1"), vec![(50.0, 50.0), (25.0, 25.0), (25.0, 25.0)]);
        assert_eq!(odds_of("30,0,1,1,1,50,10,1,1,50,11,1,-1"), vec![(50.0, 0.0), (25.0, 50.0)]);
    }

    #[test]
    fn a_score_ladder_keeps_the_drop_beside_it() {
        let row = "30,0,1,1,1,5,1028,1,-2,-2,-2,-2,-2,-2,-2,1,7000,6,50000,-1";

        assert_eq!(odds_of(row), vec![(5.0, 5.0)]);
    }
}
