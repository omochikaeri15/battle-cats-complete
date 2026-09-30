//! The Aku Realms altar tables, `DemonCastlelimit.csv` and `DemonCastledefine.csv`.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns::{self, Column};
use crate::common::file::{self, Separator};

const STAGES_PER_MAP: i32 = 100;

/// Represents errors that can occur while parsing an altar table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DemonCastleError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for DemonCastleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(formatter, "The provided byte slice contained no valid altar data."),
        }
    }
}

impl std::error::Error for DemonCastleError {}

/// One stage whose clear raises or lifts an altar's level cap.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemonCastleLimit {
    /// The stage the row belongs to, written as the global map identifier times one hundred plus the stage index.
    pub stage_key: i32,
    /// The amount a clear of the stage adds to the altar's level cap.
    pub amount: i32,
    /// Whether a clear of the stage unseals the altar, lifting its cap altogether.
    pub unseal: i32,
    /// The enemy identifier of the altar castle the row applies to.
    pub enemy_id: i32,
    /// Any trailing columns beyond the known layout, held as `None` where a column does not read as an integer.
    pub rest: Vec<Option<i32>>,
}

impl Default for DemonCastleLimit {
    fn default() -> Self {
        Self { stage_key: -1, amount: -1, unseal: -1, enemy_id: -1, rest: Vec::new() }
    }
}

impl DemonCastleLimit {
    const WIDTH: usize = 4;

    /// The column mapping this parser applies, in the order it applies it.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent -1;
        stage_key : 0;
        amount    : 1;
        unseal    : 2;
        enemy_id  : 3;
    };

    /// The global map identifier encoded in [`DemonCastleLimit::stage_key`].
    ///
    /// # Returns
    /// An `i32` holding the global map identifier.
    pub fn map_id(&self) -> i32 {
        self.stage_key / STAGES_PER_MAP
    }

    /// The stage index encoded in [`DemonCastleLimit::stage_key`].
    ///
    /// # Returns
    /// An `i32` holding the stage's index within its map.
    pub fn stage(&self) -> i32 {
        self.stage_key % STAGES_PER_MAP
    }

    /// Parses the altar cap table into one row per stage.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `DemonCastlelimit.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows in file order on success, or a
    /// `DemonCastleError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, DemonCastleError> {
        parse_limit(bytes.as_ref(), separator)
    }
}

/// One altar castle and the castle it becomes once unsealed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DemonCastleDefine {
    /// The enemy identifier of the altar castle.
    pub enemy_id: i32,
    /// The enemy identifier of the castle that replaces the altar once it is unsealed.
    pub spent_enemy_id: i32,
    /// Any trailing columns beyond the known layout, held as `None` where a column does not read as an integer.
    pub rest: Vec<Option<i32>>,
}

impl Default for DemonCastleDefine {
    fn default() -> Self {
        Self { enemy_id: -1, spent_enemy_id: -1, rest: Vec::new() }
    }
}

impl DemonCastleDefine {
    const WIDTH: usize = 2;

    /// The column mapping this parser applies, in the order it applies it.
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        absent -1;
        enemy_id       : 0;
        spent_enemy_id : 1;
    };

    /// Parses the altar castle table into one row per altar.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `DemonCastledefine.csv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows in file order on success, or a
    /// `DemonCastleError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, DemonCastleError> {
        parse_define(bytes.as_ref(), separator)
    }
}

fn strip_comment(line: &str) -> &str {
    line.split_once("//").map_or(line, |(before_comment, _)| before_comment)
}

fn cells(bytes: &[u8], separator: Option<Separator>) -> Vec<Vec<String>> {
    let content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &content);

    content
        .lines()
        .map(strip_comment)
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split(delimiter).map(|cell| cell.trim().to_owned()).collect::<Vec<String>>())
        .filter(|parts| parts.iter().any(|cell| !cell.is_empty()))
        .collect()
}

fn trailing(parts: &[&str], width: usize) -> Vec<Option<i32>> {
    parts.iter().skip(width).filter(|cell| !cell.is_empty()).map(|cell| cell.parse::<i32>().ok()).collect()
}

fn parse_limit(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<DemonCastleLimit>, DemonCastleError> {
    let rows: Vec<DemonCastleLimit> = cells(bytes, separator)
        .iter()
        .map(|owned| {
            let parts: Vec<&str> = owned.iter().map(String::as_str).collect();
            let mut row = DemonCastleLimit::default();

            columns::apply(&parts, DemonCastleLimit::COLUMNS, &mut row);
            row.rest = trailing(&parts, DemonCastleLimit::WIDTH);

            row
        })
        .collect();

    if rows.is_empty() {
        return Err(DemonCastleError::EmptyFile);
    }

    Ok(rows)
}

fn parse_define(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<DemonCastleDefine>, DemonCastleError> {
    let rows: Vec<DemonCastleDefine> = cells(bytes, separator)
        .iter()
        .map(|owned| {
            let parts: Vec<&str> = owned.iter().map(String::as_str).collect();
            let mut row = DemonCastleDefine::default();

            columns::apply(&parts, DemonCastleDefine::COLUMNS, &mut row);
            row.rest = trailing(&parts, DemonCastleDefine::WIDTH);

            row
        })
        .collect();

    if rows.is_empty() {
        return Err(DemonCastleError::EmptyFile);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(DemonCastleLimit::COLUMNS);
        columns::assert_one_field_per_column(DemonCastleDefine::COLUMNS);
    }

    #[test]
    fn limit_rows_land_column_for_column_and_skip_the_trailing_notes() {
        let rows = DemonCastleLimit::parse("3000000,1,0,574\n404200,0,1,574\n\n//note\n//another note\n", None).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].stage_key, rows[0].amount, rows[0].unseal, rows[0].enemy_id), (3_000_000, 1, 0, 574));
        assert_eq!((rows[1].map_id(), rows[1].stage()), (4042, 0));
        assert_eq!(rows[1].unseal, 1);
    }

    #[test]
    fn define_rows_follow_the_leading_note() {
        let rows = DemonCastleDefine::parse("//a note about the ids\n574,575\n", None).unwrap();

        assert_eq!(rows, vec![DemonCastleDefine { enemy_id: 574, spent_enemy_id: 575, rest: Vec::new() }]);
    }

    #[test]
    fn a_file_of_notes_alone_is_empty() {
        assert_eq!(DemonCastleLimit::parse("//only a note\n", None), Err(DemonCastleError::EmptyFile));
    }
}
