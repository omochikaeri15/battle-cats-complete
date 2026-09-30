//! Parsing of the localized text tables the Cat combo screen is built from.
//!
//! Three files share one format, a single string per line: `Nyancombo_<lang>.csv`
//! names each combo, `Nyancombo1_<lang>.csv` names each effect, and
//! `Nyancombo2_<lang>.csv` names each band an effect is granted at. A line's
//! position in its file is its identifier, so all three are read by this parser
//! and addressed by the columns of `NyancomboData.csv`.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur while parsing a localized combo text table.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NyancomboError {
    /// The supplied bytes yielded no lines at all.
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

/// One line of a localized combo text table.
///
/// A line is taken whole rather than split, because several localized combo
/// names carry a comma. Whatever precedes the trailing delimiter is kept exactly
/// as written, because a band suffix opens with the space that separates it from
/// the effect name it follows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Nyancombo {
    /// The line's text. `None` when the line is blank, which is how the file marks an untranslated entry.
    pub text: Option<String>,
}

impl Nyancombo {
    /// Parses a localized combo text table into one entry per line.
    ///
    /// Every line contributes an entry, blank ones included, so an entry's
    /// position in the returned vector is the identifier that addresses it. For
    /// a combo name table that identifier is the line number of the matching row
    /// of `NyancomboData.csv`, not the identifier that row carries.
    ///
    /// Only the regional text tables are translated in full. Every other region
    /// stops one line short of the Japanese table and leaves several entries
    /// blank, so a caller wanting a complete set reads the Japanese table as a
    /// fallback.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the combo text file.
    /// * `separator` - The delimiter the file terminates its lines with, or `None` to strip whichever one a line ends with.
    ///
    /// # Returns
    /// A `Result` containing the parsed entries in file order on success, or a
    /// `NyancomboError` if the file contained no lines.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, NyancomboError> {
        parse_inner(bytes.as_ref(), separator)
    }

    /// Parses a single line of a localized combo text table by its identifier.
    ///
    /// This avoids materializing the entire table when only one entry is
    /// required.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the combo text file.
    /// * `id` - The zero-based line offset to read.
    /// * `separator` - The delimiter the file terminates its lines with, or `None` to strip whichever one the line ends with.
    ///
    /// # Returns
    /// An `Option` containing the parsed entry, or `None` if the identifier lies
    /// beyond the end of the table.
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
        text: (!stripped.trim().is_empty()).then(|| stripped.to_owned()),
    }
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

    /// Lines of a real `Nyancombo_en.csv`, two of which carry a comma inside the name.
    const NAMES_EN: &str = "Cat Army\nNyanko, Inc.\n\nCool Breeze, Hot Day";

    /// Lines of a real `Nyancombo1_en.csv`, the second of which pads its trailing delimiter.
    const EFFECTS_EN: &str = "Unit Attack Up|\n\"Immune to Waves\" Effects |";

    /// Every line of a real `Nyancombo2_en.csv`, each opening with a separating space.
    const BANDS_EN: &str = " (Sm)|\n (M)|\n (L)|\n (XL)|\n Down|\n Activated|";

    /// Every line of a real `Nyancombo2_ja.csv`, which terminates its lines with a comma instead.
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
