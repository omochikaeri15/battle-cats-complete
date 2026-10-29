use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

const TERMINATOR: char = '＠';

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum GatyaItemNameError {
    EmptyFile,
}

impl fmt::Display for GatyaItemNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid item text."),
        }
    }
}

impl std::error::Error for GatyaItemNameError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatyaItemName {
    pub name: Option<String>,
    pub description: Vec<String>,
}

impl GatyaItemName {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, GatyaItemNameError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<GatyaItemName>, GatyaItemNameError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut entries = Vec::new();
    let mut has_content = false;

    for line in file_content.lines() {
        let mut cells = line.split(delimiter).map(str::trim);

        let name = cells
            .next()
            .filter(|text| !text.is_empty() && !text.starts_with(TERMINATOR))
            .map(str::to_owned);

        let description: Vec<String> = cells
            .take_while(|text| !text.starts_with(TERMINATOR))
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
            .collect();

        has_content |= name.is_some() || !description.is_empty();
        entries.push(GatyaItemName { name, description });
    }

    if !has_content {
        return Err(GatyaItemNameError::EmptyFile);
    }

    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FULL_ENTRY: &str = "Speed Up|3x Battle Speed!|Tap to toggle between|OFF, x2, x3 speed.|＠|";

    const TERMINATED_EARLY: &str = "XP|Purchase XP to power up your Cat Units!|＠|＠||";

    const NAME_ONLY: &str = "NP|＠||||";

    const BLANK_ENTRY: &str = "|＠||||";

    const UNTERMINATED: &str = "Rich Cat|Worker Cat starts the battle at MAX Level.|Seize the advantage in battle without needing|to spend money on upgrading Worker Cat!||";

    fn parse_one(line: &str) -> GatyaItemName {
        GatyaItemName::parse(line, None).map(|mut entries| entries.remove(0)).unwrap()
    }

    #[test]
    fn a_real_entry_keeps_its_name_and_every_description_line() {
        let entry = parse_one(FULL_ENTRY);

        assert_eq!(entry.name.as_deref(), Some("Speed Up"));
        assert_eq!(
            entry.description,
            ["3x Battle Speed!", "Tap to toggle between", "OFF, x2, x3 speed."],
        );
    }

    #[test]
    fn the_terminator_ends_the_description_and_is_never_kept() {
        let terminated = parse_one(TERMINATED_EARLY);
        assert_eq!(terminated.name.as_deref(), Some("XP"));
        assert_eq!(terminated.description, ["Purchase XP to power up your Cat Units!"]);

        let named = parse_one(NAME_ONLY);
        assert_eq!(named.name.as_deref(), Some("NP"));
        assert!(named.description.is_empty());
    }

    #[test]
    fn padding_is_not_a_blank_description_line() {
        let entry = parse_one(UNTERMINATED);

        assert_eq!(entry.description.len(), 3);
        assert_eq!(entry.description[2], "to spend money on upgrading Worker Cat!");
    }

    #[test]
    fn an_unwritten_entry_holds_its_place_in_the_file() {
        let entries = GatyaItemName::parse(format!("{FULL_ENTRY}\n{BLANK_ENTRY}\n\n{NAME_ONLY}"), None).unwrap();

        assert_eq!(entries.len(), 4);
        assert_eq!(entries[1], GatyaItemName::default());
        assert_eq!(entries[2], GatyaItemName::default());
        assert_eq!(entries[3].name.as_deref(), Some("NP"));
    }

    #[test]
    fn a_comma_written_region_reads_the_same_way() {
        let entry = GatyaItemName::parse("スピードアップ,戦闘スピードが3倍速になります,＠,", Some(Separator::Comma))
            .map(|mut entries| entries.remove(0))
            .unwrap();

        assert_eq!(entry.name.as_deref(), Some("スピードアップ"));
        assert_eq!(entry.description, ["戦闘スピードが3倍速になります"]);
    }

    #[test]
    fn a_file_without_text_is_rejected() {
        assert_eq!(GatyaItemName::parse("", None), Err(GatyaItemNameError::EmptyFile));
        assert_eq!(GatyaItemName::parse("|||||\n|||||", None), Err(GatyaItemNameError::EmptyFile));
    }
}
