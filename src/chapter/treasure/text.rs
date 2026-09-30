//! Parsing of the localized treasure text tables.
//!
//! Three tables share one layout. `Treasure1_*` names one treasure per line in
//! stage order, `Treasure2_*` holds the grade names and the popup captions, and
//! `Treasure3_*` names each set on its line, followed by the activation caption
//! and the effect it announces, and then the captions shown while a set is
//! still locked.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur while parsing a localized treasure text table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreasureTextError {
    /// The supplied bytes carried no text in any cell.
    EmptyFile,
}

impl fmt::Display for TreasureTextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no treasure text."),
        }
    }
}

impl std::error::Error for TreasureTextError {}

/// The parsed contents of one localized treasure text table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreasureText {
    /// Every line's cells in file order, trimmed, without the empty cell a trailing delimiter leaves.
    pub rows: Vec<Vec<String>>,
}

impl TreasureText {
    /// Parses a localized treasure text table.
    ///
    /// Every line contributes a row, blank ones included, so a row's position is
    /// the stage, grade or set it describes.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `Treasure1_*`, `Treasure2_*` or `Treasure3_*` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows on success, or a
    /// `TreasureTextError` if no cell carried text.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, TreasureTextError> {
        parse_inner(bytes.as_ref(), separator)
    }

    /// Returns one cell's text.
    ///
    /// # Arguments
    /// * `row` - The zero based line.
    /// * `column` - The zero based cell within the line.
    ///
    /// # Returns
    /// An `Option` holding the text, or `None` when the cell is missing or empty.
    pub fn cell(&self, row: usize, column: usize) -> Option<&str> {
        self.rows.get(row)?.get(column).map(String::as_str).filter(|text| !text.is_empty())
    }

    /// Returns the name a line opens with: a treasure in `Treasure1_*`, a set in `Treasure3_*`.
    ///
    /// # Arguments
    /// * `row` - The stage index or the set index.
    ///
    /// # Returns
    /// An `Option` holding the name, or `None` when the line leaves it unwritten.
    pub fn name(&self, row: usize) -> Option<&str> {
        self.cell(row, 0)
    }

    /// Returns a grade's name from `Treasure2_*`.
    ///
    /// # Arguments
    /// * `level` - The grade level, one for Inferior through three for Superior.
    ///
    /// # Returns
    /// An `Option` holding the name, or `None` for a level outside one to three.
    pub fn grade(&self, level: i32) -> Option<&str> {
        let row = usize::try_from(level).ok().filter(|level| (1..=3).contains(level))?;

        self.cell(row - 1, 0)
    }

    /// Returns the effect a set announces from `Treasure3_*`.
    ///
    /// # Arguments
    /// * `group` - The set index.
    ///
    /// # Returns
    /// An `Option` holding the caption, or `None` when the line leaves it unwritten.
    pub fn set_effect(&self, group: usize) -> Option<&str> {
        self.cell(group, 2)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<TreasureText, TreasureTextError> {
    let content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &content);

    let rows: Vec<Vec<String>> = content
        .lines()
        .map(|line| {
            let line = line.strip_suffix(delimiter).unwrap_or(line);

            if line.is_empty() {
                return Vec::new();
            }

            line.split(delimiter).map(|cell| cell.trim().to_owned()).collect()
        })
        .collect();

    if rows.iter().flatten().all(String::is_empty) {
        return Err(TreasureTextError::EmptyFile);
    }

    Ok(TreasureText { rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_set_line_carries_its_name_and_effect() {
        let text = TreasureText::parse("Energy Drink|Activated!|Worker Cat Efficiency increased!|\n", None).unwrap();

        assert_eq!(text.name(0), Some("Energy Drink"));
        assert_eq!(text.set_effect(0), Some("Worker Cat Efficiency increased!"));
        assert_eq!(text.rows[0].len(), 3);
    }

    #[test]
    fn grades_count_from_one() {
        let text = TreasureText::parse("Inferior|\nNormal|\nSuperior|\nTreasure|\n", None).unwrap();

        assert_eq!(text.grade(1), Some("Inferior"));
        assert_eq!(text.grade(3), Some("Superior"));
        assert_eq!(text.grade(0), None);
        assert_eq!(text.grade(4), None);
    }

    #[test]
    fn a_blank_line_keeps_its_place() {
        let text = TreasureText::parse("Fur|\n\nSlot Machine|\n", None).unwrap();

        assert_eq!(text.name(1), None);
        assert_eq!(text.name(2), Some("Slot Machine"));
        assert_eq!(TreasureText::parse("|\n|", None), Err(TreasureTextError::EmptyFile));
    }
}
