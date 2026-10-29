use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::{file, Reader};

pub const GROUPS: usize = 11;

pub const GROUP_STAGES: usize = 8;

const TERMINATOR: i32 = -1;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreasureDataError {
    EmptyFile,
}

impl fmt::Display for TreasureDataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no treasure set data."),
        }
    }
}

impl std::error::Error for TreasureDataError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreasureGroup {
    pub count: i32,
    pub stages: Vec<i32>,
    pub image: i32,
    pub percent: i32,
    pub effect: i32,
    pub chapter_only: i32,
}

impl TreasureGroup {
    pub fn members(&self) -> impl Iterator<Item = i32> + '_ {
        self.stages.iter().copied().take_while(|stage| *stage != TERMINATOR)
    }

    pub fn is_chapter_only(&self) -> bool {
        self.chapter_only != 0
    }

    pub fn kind(&self) -> super::TreasureEffect {
        super::TreasureEffect::from(self.effect)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TreasureData {
    pub groups: Vec<TreasureGroup>,
}

impl TreasureData {
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Self, TreasureDataError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<TreasureData, TreasureDataError> {
    let content = file::scrub(bytes);

    if content.trim().is_empty() {
        return Err(TreasureDataError::EmptyFile);
    }

    let mut reader = Reader::new(&content);
    let mut groups = vec![TreasureGroup::default(); GROUPS];

    for group in &mut groups {
        reader.row();
        group.count = reader.value(0);
    }

    for group in &mut groups {
        reader.row();

        for column in 0..GROUP_STAGES {
            let stage = reader.value(column);
            group.stages.push(stage);

            if stage == TERMINATOR {
                break;
            }
        }
    }

    for group in &mut groups {
        reader.row();
        group.image = reader.value(0);
    }

    for group in &mut groups {
        reader.row();
        group.percent = reader.value(0);
        group.effect = reader.value(1);
        group.chapter_only = reader.value(2);
    }

    Ok(TreasureData { groups })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(counts: &str, stages: &str, images: &str, effects: &str) -> String {
        [counts, stages, images, effects].map(|row| format!("{row}\n").repeat(GROUPS)).concat()
    }

    #[test]
    fn every_section_lands_on_its_set() {
        let data = TreasureData::parse(table("3,\t//note", "5,4,3,-1,   //note", "9,", "50,9,0,   //note")).unwrap();

        assert_eq!(data.groups.len(), GROUPS);
        assert_eq!(data.groups[0].count, 3);
        assert_eq!(data.groups[0].stages, [5, 4, 3, -1]);
        assert_eq!(data.groups[0].members().collect::<Vec<_>>(), [5, 4, 3]);
        assert_eq!(data.groups[10].image, 9);
        assert_eq!((data.groups[10].percent, data.groups[10].effect), (50, 9));
        assert!(!data.groups[10].is_chapter_only());
    }

    #[test]
    fn a_stage_row_stops_after_eight_cells_without_a_terminator() {
        let data = TreasureData::parse(table("8", "45,41,38,35,32,29,26,23,-1", "0", "100,16,0")).unwrap();

        assert_eq!(data.groups[0].stages, [45, 41, 38, 35, 32, 29, 26, 23]);
    }

    #[test]
    fn a_short_file_repeats_its_last_row() {
        let data = TreasureData::parse("2\n").unwrap();

        assert!(data.groups.iter().all(|group| group.count == 2 && group.effect == 0));
        assert_eq!(TreasureData::parse(""), Err(TreasureDataError::EmptyFile));
    }
}
