//! Parsing of the combo screen's category tabs.
//!
//! The engine declares one row of `NyancomboFilter.tsv` per tab of the combo
//! list, and one cell per effect that tab admits, in the order the tab prints
//! them. The identifiers address the rows of `NyancomboParam.tsv` and the lines
//! of `Nyancombo1_<lang>.csv`, the same way a combo's own effect column does.
//! The file carries no localized text and ships in one copy for every region.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

/// Represents errors that can occur while parsing the combo category tabs.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NyancomboFilterError {
    /// The supplied bytes yielded no parseable rows.
    EmptyFile,
}

impl fmt::Display for NyancomboFilterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid combo filter data."),
        }
    }
}

impl std::error::Error for NyancomboFilterError {}

/// The effects one tab of the combo list admits.
///
/// The tabs do not partition the effect table. The first row spans every effect
/// the game declares, while the rows below it between them leave several out, so
/// the first row has to be read as its own thing rather than rebuilt by
/// combining the others.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NyancomboFilter {
    /// The effects the tab admits, in the order the tab prints them rather than in ascending order.
    ///
    /// A cell that does not read as an integer is held as `None` rather than
    /// discarded, so an element's index is always its position within the tab.
    pub effect_types: Vec<Option<i32>>,
}

impl NyancomboFilter {
    /// Parses the combo category tabs into one entry per tab.
    ///
    /// Every line contributes an entry, blank ones included, so an entry's
    /// position in the returned vector is the tab the engine draws it as.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `NyancomboFilter.tsv` file.
    /// * `separator` - The delimiter the file is written with, or `None` to detect it from the content.
    ///
    /// # Returns
    /// A `Result` containing the parsed tabs in file order on success, or a
    /// `NyancomboFilterError` if the file contained no parseable rows.
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, NyancomboFilterError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<NyancomboFilter>, NyancomboFilterError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut tabs = Vec::new();
    let mut has_content = false;

    for line in file_content.lines() {
        if line.trim().is_empty() {
            tabs.push(NyancomboFilter::default());
            continue;
        }

        has_content = true;
        tabs.push(NyancomboFilter {
            effect_types: line.split(delimiter).map(|cell| cell.trim().parse().ok()).collect(),
        });
    }

    if !has_content {
        return Err(NyancomboFilterError::EmptyFile);
    }

    Ok(tabs)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A real `NyancomboFilter.tsv` in full, which ships identically in every region.
    const REAL_FILE: &str = concat!(
        "0\t1\t2\t3\t4\t5\t6\t7\t8\t9\t10\t11\t12\t13\t14\t15\t16\t17\t18\t19\t20\t21\t22\t23\t24\t25\t26\t27\t28\n",
        "0\t1\t2\n",
        "14\t15\t16\t17\t18\t19\t20\t21\t22\t23\t24\n",
        "3\t6\t7\t10\n",
        "5\t4\t9\n",
        "11\t12\t13",
    );

    #[test]
    fn a_real_file_lands_one_tab_per_row() {
        let tabs = NyancomboFilter::parse(REAL_FILE, None).unwrap();

        assert_eq!(tabs.len(), 6);
        assert_eq!(tabs[0].effect_types.len(), 29);
        assert_eq!(tabs[1].effect_types, [Some(0), Some(1), Some(2)]);
        assert_eq!(tabs[3].effect_types, [Some(3), Some(6), Some(7), Some(10)]);
        assert_eq!(tabs[5].effect_types, [Some(11), Some(12), Some(13)]);
    }

    #[test]
    fn a_tab_keeps_the_order_the_file_prints_it_in() {
        let tabs = NyancomboFilter::parse(REAL_FILE, None).unwrap();

        assert_eq!(tabs[4].effect_types, [Some(5), Some(4), Some(9)]);
    }

    #[test]
    fn the_tabs_below_the_first_do_not_cover_it() {
        let tabs = NyancomboFilter::parse(REAL_FILE, None).unwrap();

        let mut covered: Vec<i32> = tabs[1..]
            .iter()
            .flat_map(|tab| tab.effect_types.iter().copied().flatten())
            .collect();
        covered.sort_unstable();

        let every: Vec<i32> = tabs[0].effect_types.iter().copied().flatten().collect();
        let missing: Vec<i32> = every.iter().copied().filter(|id| !covered.contains(id)).collect();

        assert_eq!(missing, [8, 25, 26, 27, 28]);
    }

    #[test]
    fn an_unreadable_cell_is_held_in_place() {
        let tabs = NyancomboFilter::parse("11\tx\t13", None).unwrap();

        assert_eq!(tabs[0].effect_types, [Some(11), None, Some(13)]);
    }

    #[test]
    fn a_blank_line_holds_its_place_in_the_file() {
        let tabs = NyancomboFilter::parse("0\t1\t2\n\n11\t12\t13", None).unwrap();

        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[1], NyancomboFilter::default());
        assert_eq!(tabs[2].effect_types, [Some(11), Some(12), Some(13)]);
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(NyancomboFilter::parse("", None), Err(NyancomboFilterError::EmptyFile));
        assert_eq!(NyancomboFilter::parse("\n\n", None), Err(NyancomboFilterError::EmptyFile));
    }
}
