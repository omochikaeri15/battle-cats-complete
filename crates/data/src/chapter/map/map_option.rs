//! Per-map behavioral configuration such as crown tiers and repeat timers.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::{Column, FromColumn};

/// Represents errors that can occur during the parsing of map options.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum MapOptionError {
    /// The supplied bytes yielded no parseable rows beyond the header.
    EmptyFile,
    /// The supplied bytes did not begin with the required header row.
    MissingHeaders,
}

impl fmt::Display for MapOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid map option data."
            ),
            Self::MissingHeaders => write!(
                formatter,
                "The file is missing the required header row."
            ),
        }
    }
}

impl std::error::Error for MapOptionError {}

/// Selects what a map discards when its repeat timer elapses.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ResetType {
    /// Nothing is reset, so the map's rewards are claimable once only.
    #[default]
    None,
    /// The map's reward flags are cleared, making its drops claimable again.
    ResetRewards,
    /// The reward flags and the cleared marker are both cleared.
    ResetRewardsAndClear,
    /// The accumulated clear count is cleared.
    ResetMaxClears,
    /// A reset code this parser does not recognize, carrying its raw value.
    Unknown(u8),
}

impl From<u8> for ResetType {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::ResetRewards,
            2 => Self::ResetRewardsAndClear,
            3 => Self::ResetMaxClears,
            _ => Self::Unknown(value),
        }
    }
}

impl FromColumn for ResetType {
    fn from_column(text: &str) -> Option<Self> {
        text.parse::<u8>().ok().map(Self::from)
    }
}

/// The behavioral configuration of a single map.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOptionEntry {
    /// The identifier of the map this configuration applies to.
    pub map_id: u32,
    /// The highest crown difficulty the map can be attempted at.
    pub max_crowns: u8,
    /// Whether the map exposes the additional Abyss difficulty tier.
    pub has_abyss: bool,
    /// The enemy strength percentage applied at one crown, if declared.
    pub crown_1_mag: Option<u32>,
    /// The enemy strength percentage applied at two crowns, if declared.
    pub crown_2_mag: Option<u32>,
    /// The enemy strength percentage applied at three crowns, if declared.
    pub crown_3_mag: Option<u32>,
    /// The enemy strength percentage applied at four crowns, if declared.
    pub crown_4_mag: Option<u32>,
    /// What the map discards when its repeat timer elapses.
    pub reset_type: ResetType,
    /// The number of times the map may be cleared for rewards.
    pub max_clears: u32,
    /// The delay in minutes before the map becomes available again.
    pub cooldown_minutes: u32,
    /// Whether the map is removed from the selection list once cleared.
    pub hidden_upon_clear: bool,
    /// The multiplier applied to every unit's base deployment cost, or zero when the map leaves costs alone.
    pub cost_multiplier: u32,
    /// The trailing comment text accompanying the row in the source file.
    pub comment: String,
}

impl MapOptionEntry {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `Map_option.csv` row
    /// from the parser's own table instead of restating it. The indices the
    /// table skips carry nothing this parser reads, and
    /// [`MapOptionEntry::comment`] is the row's trailing comment text rather
    /// than a column.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        map_id            : 0;
        max_crowns        : 1, Raw, 1;
        has_abyss         : 2;
        crown_1_mag       : 3, Raw, -1;
        crown_2_mag       : 4, Raw, -1;
        crown_3_mag       : 5, Raw, -1;
        crown_4_mag       : 6, Raw, -1;
        reset_type        : 8;
        max_clears        : 9;
        cooldown_minutes  : 11;
        hidden_upon_clear : 14;
        cost_multiplier   : 16;
    };
}

/// The parsed contents of the map option table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapOption {
    /// The map configurations, keyed by map identifier.
    pub entries: HashMap<u32, MapOptionEntry>,
}

impl MapOption {
    /// Parses the map option table into per-map behavioral configurations.
    ///
    /// The leading header row is skipped. Rows short of the expected column
    /// count have known-optional columns reinserted at their documented
    /// positions, which keeps older regional files readable against the current
    /// layout. Trailing comment text is retained on the resulting entry.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the map option file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `MapOption` on success, or a
    /// `MapOptionError` if the header was absent or no rows were parseable.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, MapOptionError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<MapOption, MapOptionError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut lines_iterator = file_content.lines();

    if lines_iterator.next().is_none() {
        return Err(MapOptionError::MissingHeaders);
    }

    let mut entries = HashMap::new();
    let mut has_content = false;

    for file_line in lines_iterator {
        let mut data_part = file_line;
        let mut comment_part = "";

        if let Some((before_comment, after_comment)) = file_line.split_once("//") {
            data_part = before_comment;
            comment_part = after_comment;
        }

        let trimmed_data = data_part.trim();
        if trimmed_data.is_empty() {
            continue;
        }

        has_content = true;

        let mut parts: Vec<&str> = trimmed_data.split(separator_char).collect();

        let expected_columns: usize = 19;
        let injection_points = [(2, "0")];

        let missing_cols = expected_columns.saturating_sub(parts.len());
        for &(inject_idx, default_val) in injection_points.iter().take(missing_cols) {
            if inject_idx <= parts.len() {
                parts.insert(inject_idx, default_val);
            }
        }

        let Some(map_id_string) = parts.first() else { continue; };
        let Ok(map_id) = map_id_string.trim().parse::<u32>() else { continue; };

        let mut entry = MapOptionEntry {
            comment: comment_part.trim().to_string(),
            ..MapOptionEntry::default()
        };
        columns::apply(&parts, MapOptionEntry::COLUMNS, &mut entry);

        entries.insert(map_id, entry);
    }

    if !has_content {
        return Err(MapOptionError::EmptyFile);
    }

    Ok(MapOption { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(MapOptionEntry::COLUMNS);
    }
}
