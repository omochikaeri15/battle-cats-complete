//! Lineup restrictions that stages impose on the player's deployable units.
use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::Column;

/// Represents errors that can occur during the parsing of stage lineup restrictions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum StageOptionError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for StageOptionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid stage option data."
            ),
        }
    }
}

impl std::error::Error for StageOptionError {}

/// One lineup restriction and the stages it applies to.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOptionEntry {
    /// The identifier of the map the restriction applies to.
    pub map_id: u32,
    /// The crown difficulty the restriction applies at, or negative to apply at every difficulty.
    pub target_crowns: i8,
    /// The index of the stage the restriction applies to, or negative to apply to every stage.
    pub target_stage: i32,
    /// A bitmask of the rarity tiers permitted, where each bit selects one tier.
    pub rarity_mask: u8,
    /// The greatest number of units deployable at once.
    pub deploy_limit: u32,
    /// A bitmask of the lineup rows permitted, where each bit selects one row.
    pub allowed_rows: u8,
    /// The lowest deployment cost a unit may have to be eligible.
    pub min_cost: u32,
    /// The highest deployment cost a unit may have to be eligible.
    pub max_cost: u32,
    /// The identifier of the unit restriction group applied, or zero when none is.
    pub charagroup_id: u32,
}

impl StageOptionEntry {
    /// The column mapping this parser applies, in the order it applies it.
    ///
    /// Published so a consumer can read the layout of a `stage_option.csv` row
    /// from the parser's own table instead of restating it.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        map_id        : 0;
        target_crowns : 1, Raw, -1;
        target_stage  : 2, Raw, -1;
        rarity_mask   : 3;
        deploy_limit  : 4;
        allowed_rows  : 5;
        min_cost      : 6;
        max_cost      : 7;
        charagroup_id : 8;
    };
}

/// The parsed contents of the stage lineup restriction table.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageOption {
    /// The restrictions applying to each map, keyed by map identifier.
    pub entries: HashMap<u32, Vec<StageOptionEntry>>,
}

impl StageOption {
    /// Parses the stage lineup restriction table into per-map restriction lists.
    ///
    /// A map may declare several restrictions covering different stages and
    /// crown difficulties, and a single restriction may apply across all of
    /// them, so each map's restrictions are collected into a list rather than
    /// reduced to one.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the stage option file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed `StageOption` on success, or a
    /// `StageOptionError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, StageOptionError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<StageOption, StageOptionError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut entries: HashMap<u32, Vec<StageOptionEntry>> = HashMap::new();
    let mut has_content = false;

    let lines_iterator = file_content.lines().skip(1);

    for file_line in lines_iterator {
        let mut clean_line = file_line;
        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        if parts.len() < 9 {
            continue;
        }

        let Some(raw_map_id_str) = parts.first() else { continue; };
        let Ok(map_id) = raw_map_id_str.trim().parse::<u32>() else { continue; };

        let mut entry = StageOptionEntry::default();
        columns::apply(&parts, StageOptionEntry::COLUMNS, &mut entry);

        entries.entry(map_id).or_default().push(entry);
    }

    if !has_content {
        return Err(StageOptionError::EmptyFile);
    }

    Ok(StageOption { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(StageOptionEntry::COLUMNS);
    }
}
