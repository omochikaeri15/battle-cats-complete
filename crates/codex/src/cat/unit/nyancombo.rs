use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

const INVISIBLE: [char; 6] = ['\u{200b}', '\u{200c}', '\u{200d}', '\u{2060}', '\u{feff}', '\u{00ad}'];

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NyancomboError {
    EmptyFile,
}

impl fmt::Display for NyancomboError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided combo text file contained no valid entries."),
        }
    }
}

impl std::error::Error for NyancomboError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nyancombo {
    pub text: Option<String>,
}

impl Nyancombo {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, NyancomboError> {
        parse_inner(bytes.as_ref(), separator)
    }

    pub fn parse_row<B: AsRef<[u8]>>(bytes: B, id: usize, separator: Option<Separator>) -> Option<Self> {
        parse_row_inner(bytes.as_ref(), id, separator)
    }
}

fn parse_line(line: &str, separator: Option<Separator>) -> Nyancombo {
    let stripped = separator.map_or_else(
        || {
            Separator::ALL
                .iter()
                .find_map(|candidate| line.strip_suffix(candidate.char()))
                .unwrap_or(line)
        },
        |separator| line.strip_suffix(separator.char()).unwrap_or(line),
    );

    Nyancombo {
        text: (!is_blank(stripped)).then(|| stripped.to_owned()),
    }
}

fn is_blank(text: &str) -> bool {
    text.chars().all(|c| c.is_whitespace() || c.is_control() || INVISIBLE.contains(&c))
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<Nyancombo>, NyancomboError> {
    let file_content = file::scrub(bytes);

    let entries: Vec<Nyancombo> = file_content
        .lines()
        .map(|line| parse_line(line, separator))
        .collect();

    if entries.is_empty() {
        return Err(NyancomboError::EmptyFile);
    }

    Ok(entries)
}

fn parse_row_inner(bytes: &[u8], id: usize, separator: Option<Separator>) -> Option<Nyancombo> {
    let file_content = file::scrub(bytes);

    file_content.lines().nth(id).map(|line| parse_line(line, separator))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES_EN: &str = "Cat Army\nNyanko, Inc.\n\nCool Breeze, Hot Day";

    const EFFECTS_EN: &str = "Unit Attack Up|\n\"Immune to Waves\" Effects |";

    const BANDS_EN: &str = " (Sm)|\n (M)|\n (L)|\n (XL)|\n Down|\n Activated|";

    const BANDS_JA: &str = "【小】,\n【中】,\n【大】,\n【究極】,\nダウン,\n付与,";

    fn texts(table: &[Nyancombo]) -> Vec<Option<&str>> {
        table.iter().map(|entry| entry.text.as_deref()).collect()
    }

    #[test]
    fn a_name_carrying_a_comma_survives_whole() {
        let table = Nyancombo::parse(NAMES_EN, None).unwrap();

        assert_eq!(
            texts(&table),
            [Some("Cat Army"), Some("Nyanko, Inc."), None, Some("Cool Breeze, Hot Day")],
        );
    }

    #[test]
    fn a_trailing_delimiter_is_stripped_rather_than_split_on() {
        assert_eq!(
            texts(&Nyancombo::parse(EFFECTS_EN, None).unwrap()),
            [Some("Unit Attack Up"), Some("\"Immune to Waves\" Effects ")],
        );

        assert_eq!(
            texts(&Nyancombo::parse(BANDS_JA, Some(Separator::Comma)).unwrap()),
            [Some("【小】"), Some("【中】"), Some("【大】"), Some("【究極】"), Some("ダウン"), Some("付与")],
        );
    }

    #[test]
    fn a_band_keeps_the_space_that_separates_it_from_its_effect() {
        let bands = Nyancombo::parse(BANDS_EN, Some(Separator::Pipe)).unwrap();
        let effects = Nyancombo::parse(EFFECTS_EN, Some(Separator::Pipe)).unwrap();

        assert_eq!(texts(&bands)[0], Some(" (Sm)"));
        assert_eq!(texts(&bands)[5], Some(" Activated"));

        let described = format!(
            "{}{}",
            effects[0].text.clone().unwrap_or_default(),
            bands[0].text.clone().unwrap_or_default(),
        );

        assert_eq!(described, "Unit Attack Up (Sm)");
    }

    #[test]
    fn an_invisible_placeholder_counts_as_a_blank_line() {
        let parsed = Nyancombo::parse("Cat Army\n\u{200b}\n\u{feff}  \nRed Strike\n", None).expect("the table parses");

        assert_eq!(parsed[1].text, None);
        assert_eq!(parsed[2].text, None);
        assert_eq!(parsed[3].text.as_deref(), Some("Red Strike"));
    }

    #[test]
    fn a_blank_line_holds_its_place_in_the_file() {
        let table = Nyancombo::parse(NAMES_EN, None).unwrap();

        assert_eq!(table.len(), 4);
        assert_eq!(table[2], Nyancombo::default());
        assert_eq!(table[3].text.as_deref(), Some("Cool Breeze, Hot Day"));
    }

    #[test]
    fn a_stated_delimiter_leaves_the_other_two_alone() {
        let table = Nyancombo::parse("Nyanko, Inc.", Some(Separator::Comma)).unwrap();

        assert_eq!(texts(&table), [Some("Nyanko, Inc.")]);
    }

    #[test]
    fn one_row_reads_the_same_as_the_whole_table() {
        let table = Nyancombo::parse(NAMES_EN, None).unwrap();

        for (id, entry) in table.iter().enumerate() {
            assert_eq!(Nyancombo::parse_row(NAMES_EN, id, None).as_ref(), Some(entry));
        }

        assert_eq!(Nyancombo::parse_row(NAMES_EN, table.len(), None), None);
    }

    #[test]
    fn a_file_without_lines_is_rejected() {
        assert_eq!(Nyancombo::parse("", None), Err(NyancomboError::EmptyFile));
    }
}
