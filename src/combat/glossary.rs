//! Parsing of the in-game dictionary text that names traits and abilities.
//!
//! The Cat and Enemy dictionaries each ship a glossary, `nyankoPictureBook2`
//! and `EnemyPictureBook2`, laid out line for line the same way in every
//! language: the first sixteen lines describe the traits and the rest describe
//! the abilities, each as the name in fullwidth corner brackets followed by
//! the lines of its explanation. The Talent Orb screen names the twelve traits
//! an orb can target in `attribute_explonation`, one per line with the plural
//! form beside it. An ability's registry entry records the line that names it
//! in each glossary and its row in the orb table, so a name is always read
//! from the files rather than kept in the crate.
//!
//! The Cat dictionary does not address its glossary by line directly. Each
//! ability it lists has a row in `nyankoPictureBookData_EffectAbility` giving
//! the engine's own ability identifier and the glossary entry it is described
//! by, counted from after the trait lines, and the trait lines are as many as
//! `nyankoPictureBookData_Attribute` has rows. A glossary whose lines have
//! moved therefore still reads correctly as long as both tables moved with it.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::cell_value;
use crate::common::file::{self, Separator};

/// The row identifier that ends the table early.
const END_ROW: i32 = -1;

/// The character the engine ends a glossary line with, which is never rendered.
const TERMINATOR: char = '＠';

/// The fullwidth corner brackets a glossary wraps a name in.
const OPEN: char = '【';
const CLOSE: char = '】';

/// Represents errors that can occur while parsing a dictionary glossary.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GlossaryError {
    /// The supplied bytes carried no text in any cell.
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

/// One glossary line, describing a trait or an ability.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlossaryEntry {
    /// The bracketed name, without its brackets, or `None` when the line opens with plain text.
    pub name: Option<String>,
    /// The explanation lines in the order they are printed, with the terminator removed.
    pub lines: Vec<String>,
}

/// The parsed contents of one dictionary glossary.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Glossary {
    /// Every line's entry in file order, blank lines included, so an entry's position plus one is its line.
    pub entries: Vec<GlossaryEntry>,
}

impl Glossary {
    /// Parses a dictionary glossary.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `nyankoPictureBook2` or `EnemyPictureBook2` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed glossary on success, or a
    /// `GlossaryError` if no cell carried text.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, GlossaryError> {
        parse_glossary(bytes.as_ref(), separator)
    }

    /// Returns the entry a glossary line holds.
    ///
    /// # Arguments
    /// * `line` - The one-based line, as an ability's registry entry records it.
    ///
    /// # Returns
    /// An `Option` holding the entry, or `None` when the file is shorter than that.
    pub fn entry(&self, line: u8) -> Option<&GlossaryEntry> {
        self.entries.get(usize::from(line).checked_sub(1)?)
    }

    /// Returns the name a glossary line carries.
    ///
    /// # Arguments
    /// * `line` - The one-based line, as an ability's registry entry records it.
    ///
    /// # Returns
    /// An `Option` holding the name, or `None` when the line has no bracketed name.
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

/// Represents errors that can occur while parsing the Cat dictionary's trait table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AttributeIndexError {
    /// The supplied bytes carried no rows.
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

/// One row of the Cat dictionary's trait table, column for column.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeRow {
    /// The row's own number.
    pub row: i32,
    /// The second column, whose meaning this crate does not know.
    pub unknown_1: i32,
    /// The third column, whose meaning this crate does not know.
    pub unknown_2: i32,
    /// The fourth column, whose meaning this crate does not know.
    pub unknown_3: i32,
    /// The fifth column, whose meaning this crate does not know.
    pub unknown_4: i32,
}

/// The parsed contents of the Cat dictionary's trait table.
///
/// The glossary opens with one line per row of this table, so its length is
/// where the ability entries start.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttributeIndex {
    /// Every row before the end marker, in file order.
    pub rows: Vec<AttributeRow>,
}

impl AttributeIndex {
    /// Parses the Cat dictionary's trait table.
    ///
    /// Rows are read the way the engine reads them, stopping at the first row
    /// whose number is the end marker.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `nyankoPictureBookData_Attribute.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows on success, or an
    /// `AttributeIndexError` if the file held no rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, AttributeIndexError> {
        parse_attributes(bytes.as_ref())
    }

    /// Returns the number of glossary lines the traits occupy before the abilities begin.
    ///
    /// # Returns
    /// A `u8` holding the count, which saturates should a table run past what a line index can hold.
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

/// Represents errors that can occur while parsing the Cat dictionary's ability table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictionaryIndexError {
    /// The supplied bytes carried no rows.
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

/// One row of the Cat dictionary's ability table, column for column.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryRow {
    /// The row's own number.
    pub row: i32,
    /// The engine's identifier for the ability.
    pub ability: i32,
    /// The kind of entry: 0 for an innate ability, 1 for a talent, 2 for a talent that only buffs.
    pub kind: i32,
    /// The one-based glossary entry describing the ability, counted from after the trait lines, or 0 when it has none.
    pub entry: i32,
    /// The icon drawn for the entry.
    pub icon: i32,
    /// The icon drawn for the entry when the ability is inactive.
    pub inactive_icon: i32,
    /// The tab the entry is filed under.
    pub category: i32,
}

/// The parsed contents of the Cat dictionary's ability table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DictionaryIndex {
    /// Every row before the end marker, in file order.
    pub rows: Vec<DictionaryRow>,
}

impl DictionaryIndex {
    /// Parses the Cat dictionary's ability table.
    ///
    /// Rows are read the way the engine reads them, stopping at the first row
    /// whose number is the end marker.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `nyankoPictureBookData_EffectAbility.csv` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed rows on success, or a
    /// `DictionaryIndexError` if the file held no rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, DictionaryIndexError> {
        parse_index(bytes.as_ref())
    }

    /// Returns the glossary line describing an ability.
    ///
    /// # Arguments
    /// * `ability` - The engine's identifier for the ability, as an ability's registry entry records it.
    /// * `traits` - The trait table the same dictionary ships, whose rows the glossary opens with.
    ///
    /// # Returns
    /// An `Option` holding the one-based line of `nyankoPictureBook2`, or `None`
    /// when the table does not list the ability or lists it without an entry.
    pub fn glossary_line(&self, ability: u8, traits: &AttributeIndex) -> Option<u8> {
        let row = self.rows.iter().find(|row| row.ability == i32::from(ability))?;
        let entry = u8::try_from(row.entry).ok().filter(|entry| *entry > 0)?;

        entry.checked_add(traits.trait_lines())
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
            ability: value(1),
            kind: value(2),
            entry: value(3),
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

/// Represents errors that can occur while parsing the orb trait labels.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TraitLabelError {
    /// The supplied bytes carried no label on any line.
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

/// One trait as the Talent Orb screen names it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitLabel {
    /// The trait's bare name.
    pub name: String,
    /// The trait's plural, as the enemies carrying it are collectively called.
    pub plural: String,
}

/// The parsed contents of the orb trait label table.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraitLabels {
    /// Every line's label in file order, so a label's position is the row an ability's registry entry records.
    pub entries: Vec<TraitLabel>,
}

impl TraitLabels {
    /// Parses the orb trait label table.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `attribute_explonation` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed labels on success, or a
    /// `TraitLabelError` if no line carried a name.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, TraitLabelError> {
        parse_labels(bytes.as_ref())
    }

    /// Returns a trait's bare name.
    ///
    /// # Arguments
    /// * `row` - The zero-based row, as an ability's registry entry records it.
    ///
    /// # Returns
    /// An `Option` holding the name, or `None` when the file is shorter than that.
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
        assert_eq!(index.glossary_line(15, &sixteen), Some(26));
        assert_eq!(index.glossary_line(0, &fifteen), Some(16), "an older dictionary with one trait fewer starts a line earlier");
        assert_eq!(index.glossary_line(28, &sixteen), None, "an icon-only row names no glossary entry");
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
