use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::cell_value;
use crate::common::file::{self, Separator};

const END_ROW: i32 = -1;

const TERMINATOR: char = '＠';

const OPEN: char = '【';
const CLOSE: char = '】';

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GlossaryError {
    EmptyFile,
}

impl fmt::Display for GlossaryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no glossary text."),
        }
    }
}

impl std::error::Error for GlossaryError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlossaryEntry {
    pub name: Option<String>,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glossary {
    pub entries: Vec<GlossaryEntry>,
}

impl Glossary {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, GlossaryError> {
        parse_glossary(bytes.as_ref(), separator)
    }

    pub fn entry(&self, line: u8) -> Option<&GlossaryEntry> {
        self.entries.get(usize::from(line).checked_sub(1)?)
    }

    pub fn name(&self, line: u8) -> Option<&str> {
        self.entry(line)?.name.as_deref()
    }
}

fn parse_glossary(bytes: &[u8], separator: Option<Separator>) -> Result<Glossary, GlossaryError> {
    let content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &content);
    let mut has_text = false;

    let entries = content
        .lines()
        .map(|line| {
            let mut cells = line
                .split(delimiter)
                .map(str::trim)
                .take_while(|cell| !cell.starts_with(TERMINATOR))
                .filter(|cell| !cell.is_empty())
                .peekable();

            let name = cells
                .peek()
                .and_then(|cell| cell.strip_prefix(OPEN)?.strip_suffix(CLOSE))
                .map(str::to_owned);

            if name.is_some() {
                cells.next();
            }

            let lines: Vec<String> = cells.map(str::to_owned).collect();

            has_text |= name.is_some() || !lines.is_empty();

            GlossaryEntry { name, lines }
        })
        .collect();

    if !has_text {
        return Err(GlossaryError::EmptyFile);
    }

    Ok(Glossary { entries })
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttributeIndexError {
    EmptyFile,
}

impl fmt::Display for AttributeIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no dictionary trait rows."),
        }
    }
}

impl std::error::Error for AttributeIndexError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRow {
    pub row: i32,
    pub unknown_1: i32,
    pub unknown_2: i32,
    pub unknown_3: i32,
    pub unknown_4: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeIndex {
    pub rows: Vec<AttributeRow>,
}

impl AttributeIndex {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, AttributeIndexError> {
        parse_attributes(bytes.as_ref())
    }

    pub fn trait_lines(&self) -> u8 {
        u8::try_from(self.rows.len()).unwrap_or(u8::MAX)
    }
}

fn parse_attributes(bytes: &[u8]) -> Result<AttributeIndex, AttributeIndexError> {
    let content = file::scrub(bytes);
    let mut rows = Vec::new();

    for line in content.lines() {
        let cells: Vec<i32> = line.split(',').map(cell_value).collect();
        let value = |index: usize| cells.get(index).copied().unwrap_or(0);

        if value(0) == END_ROW {
            break;
        }

        rows.push(AttributeRow { row: value(0), unknown_1: value(1), unknown_2: value(2), unknown_3: value(3), unknown_4: value(4) });
    }

    if rows.is_empty() {
        return Err(AttributeIndexError::EmptyFile);
    }

    Ok(AttributeIndex { rows })
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictionaryIndexError {
    EmptyFile,
}

impl fmt::Display for DictionaryIndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no dictionary ability rows."),
        }
    }
}

impl std::error::Error for DictionaryIndexError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryRow {
    pub row: i32,
    pub order: i32,
    pub kind: i32,
    pub talent: i32,
    pub icon: i32,
    pub inactive_icon: i32,
    pub category: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryIndex {
    pub rows: Vec<DictionaryRow>,
}

impl DictionaryIndex {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, DictionaryIndexError> {
        parse_index(bytes.as_ref())
    }

    pub fn glossary_line(&self, ability: u8, traits: &AttributeIndex) -> Option<u8> {
        self.rows.iter().find(|row| row.row == i32::from(ability))?;

        ability.checked_add(1)?.checked_add(traits.trait_lines())
    }
}

fn parse_index(bytes: &[u8]) -> Result<DictionaryIndex, DictionaryIndexError> {
    let content = file::scrub(bytes);
    let mut rows = Vec::new();

    for line in content.lines() {
        let cells: Vec<i32> = line.split(',').map(cell_value).collect();
        let value = |index: usize| cells.get(index).copied().unwrap_or(0);

        if value(0) == END_ROW {
            break;
        }

        rows.push(DictionaryRow {
            row: value(0),
            order: value(1),
            kind: value(2),
            talent: value(3),
            icon: value(4),
            inactive_icon: value(5),
            category: value(6),
        });
    }

    if rows.is_empty() {
        return Err(DictionaryIndexError::EmptyFile);
    }

    Ok(DictionaryIndex { rows })
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TraitLabelError {
    EmptyFile,
}

impl fmt::Display for TraitLabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no trait labels."),
        }
    }
}

impl std::error::Error for TraitLabelError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitLabel {
    pub name: String,
    pub plural: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitLabels {
    pub entries: Vec<TraitLabel>,
}

impl TraitLabels {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, TraitLabelError> {
        parse_labels(bytes.as_ref())
    }

    pub fn name(&self, row: u8) -> Option<&str> {
        self.entries.get(usize::from(row)).map(|label| label.name.as_str())
    }
}

fn parse_labels(bytes: &[u8]) -> Result<TraitLabels, TraitLabelError> {
    let content = file::scrub(bytes);

    let entries: Vec<TraitLabel> = content
        .lines()
        .map(|line| {
            let (name, plural) = line.split_once('\t').unwrap_or((line, ""));

            TraitLabel { name: name.trim().to_owned(), plural: plural.trim().to_owned() }
        })
        .collect();

    if entries.iter().all(|label| label.name.is_empty()) {
        return Err(TraitLabelError::EmptyFile);
    }

    Ok(TraitLabels { entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bracketed_line_splits_into_its_name_and_explanation() {
        let glossary = Glossary::parse("【Weaken】|Indicated enemy types will have their|attack power reduced for a set period.|＠\n", None).unwrap();

        assert_eq!(glossary.name(1), Some("Weaken"));
        assert_eq!(glossary.entry(1).unwrap().lines, ["Indicated enemy types will have their", "attack power reduced for a set period."]);
        assert_eq!(glossary.name(2), None);
    }

    #[test]
    fn a_plain_line_keeps_its_text_without_a_name() {
        let glossary = Glossary::parse("Effective against Red enemies.|＠\n＠\n【Slow】|＠", None).unwrap();

        assert_eq!(glossary.entry(1).unwrap(), &GlossaryEntry { name: None, lines: vec!["Effective against Red enemies.".to_owned()] });
        assert_eq!(glossary.entry(2).unwrap(), &GlossaryEntry::default());
        assert_eq!(glossary.name(3), Some("Slow"));
    }

    #[test]
    fn a_comma_written_glossary_reads_the_same_way() {
        let glossary = Glossary::parse("【攻撃力ダウン】,ターゲットとなる敵キャラクターの,＠", Some(Separator::Comma)).unwrap();

        assert_eq!(glossary.name(1), Some("攻撃力ダウン"));
        assert_eq!(glossary.entry(1).unwrap().lines.len(), 1);
        assert_eq!(Glossary::parse("＠\n|＠", None), Err(GlossaryError::EmptyFile));
    }

    #[test]
    fn the_ability_table_points_past_the_trait_lines() {
        let index = DictionaryIndex::parse("0,0,0,1,163,195,0\n1,15,1,10,164,196,1\n14,28,1,0,177,209,1\n-1\n99,99,0,5,0,0,0\n").unwrap();
        let sixteen = AttributeIndex::parse("0,6,77,129,2\n".repeat(16)).unwrap();
        let fifteen = AttributeIndex::parse("0,6,77,129,2\n".repeat(15)).unwrap();

        assert_eq!(index.rows.len(), 3);
        assert_eq!(index.glossary_line(0, &sixteen), Some(17));
        assert_eq!(index.glossary_line(14, &sixteen), Some(31), "the row number picks the line, wherever the row is listed");
        assert_eq!(index.glossary_line(0, &fifteen), Some(16), "an older dictionary with one trait fewer starts a line earlier");
        assert_eq!(index.glossary_line(28, &sixteen), None, "a listing position is not a row number");
        assert_eq!(index.glossary_line(99, &sixteen), None, "rows after the end marker are never read");
        assert_eq!(DictionaryIndex::parse("-1\n"), Err(DictionaryIndexError::EmptyFile));
        assert_eq!(AttributeIndex::parse("-1\n"), Err(AttributeIndexError::EmptyFile));
    }

    #[test]
    fn trait_labels_are_read_by_row() {
        let labels = TraitLabels::parse("Red\tRed Enemies\nFloating\tFloating Enemies\n").unwrap();

        assert_eq!(labels.name(0), Some("Red"));
        assert_eq!(labels.entries[1].plural, "Floating Enemies");
        assert_eq!(labels.name(2), None);
        assert_eq!(TraitLabels::parse("\t\n"), Err(TraitLabelError::EmptyFile));
    }
}
