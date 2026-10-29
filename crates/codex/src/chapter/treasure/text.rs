use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreasureTextError {
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreasureText {
    pub rows: Vec<Vec<String>>,
}

impl TreasureText {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, TreasureTextError> {
        parse_inner(bytes.as_ref(), separator)
    }

    pub fn cell(&self, row: usize, column: usize) -> Option<&str> {
        self.rows.get(row)?.get(column).map(String::as_str).filter(|text| !text.is_empty())
    }

    pub fn name(&self, row: usize) -> Option<&str> {
        self.cell(row, 0)
    }

    pub fn grade(&self, level: i32) -> Option<&str> {
        let row = usize::try_from(level).ok().filter(|level| (1..=3).contains(level))?;

        self.cell(row - 1, 0)
    }

    pub fn set_effect(&self, group: usize) -> Option<String> {
        let lines: Vec<&str> = [1, 2].into_iter().filter_map(|column| self.cell(group, column)).collect();

        (!lines.is_empty()).then(|| lines.join(" "))
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
        assert_eq!(text.set_effect(0).as_deref(), Some("Activated! Worker Cat Efficiency increased!"));
        assert_eq!(text.rows[0].len(), 3);
    }

    #[test]
    fn a_caption_broken_across_its_lines_reads_as_one_sentence() {
        let text = TreasureText::parse("Titanium Fruit|Anti-Metal abilities have|increased effect!|\n", None).unwrap();

        assert_eq!(text.set_effect(0).as_deref(), Some("Anti-Metal abilities have increased effect!"));
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
