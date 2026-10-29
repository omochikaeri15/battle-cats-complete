use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

use super::ComboStrength;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NyancomboParamError {
    EmptyFile,
}

impl fmt::Display for NyancomboParamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid combo magnitude data."),
        }
    }
}

impl std::error::Error for NyancomboParamError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NyancomboParam {
    pub magnitudes: Vec<Option<i32>>,
}

impl NyancomboParam {
    pub fn magnitude(&self, strength: ComboStrength) -> Option<i32> {
        usize::try_from(strength.index())
            .ok()
            .and_then(|band| self.magnitudes.get(band))
            .copied()
            .flatten()
    }

    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, NyancomboParamError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<NyancomboParam>, NyancomboParamError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let mut rows = Vec::new();
    let mut has_content = false;

    for line in file_content.lines() {
        if line.trim().is_empty() {
            rows.push(NyancomboParam::default());
            continue;
        }

        has_content = true;
        rows.push(NyancomboParam {
            magnitudes: line.split(delimiter).map(|cell| cell.trim().parse().ok()).collect(),
        });
    }

    if !has_content {
        return Err(NyancomboParamError::EmptyFile);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCALING_ROW: &str = "10\t15\t20\t30\t-20";

    const CONFERRED_ROW: &str = "2500\t2500\t2500\t2500\t2500\t2500";

    #[test]
    fn a_real_row_lands_cell_for_cell() {
        let rows = NyancomboParam::parse(SCALING_ROW, None).unwrap();

        assert_eq!(rows[0].magnitudes, [Some(10), Some(15), Some(20), Some(30), Some(-20)]);
        assert_eq!(rows[0].magnitude(ComboStrength::Small), Some(10));
        assert_eq!(rows[0].magnitude(ComboStrength::Medium), Some(15));
        assert_eq!(rows[0].magnitude(ComboStrength::Large), Some(20));
        assert_eq!(rows[0].magnitude(ComboStrength::Ultimate), Some(30));
        assert_eq!(rows[0].magnitude(ComboStrength::Down), Some(-20));
    }

    #[test]
    fn a_row_keeps_the_width_the_file_declares() {
        let scaling = NyancomboParam::parse(SCALING_ROW, None).unwrap();
        let conferred = NyancomboParam::parse(CONFERRED_ROW, None).unwrap();

        assert_eq!(scaling[0].magnitudes.len(), 5);
        assert_eq!(conferred[0].magnitudes.len(), 6);

        assert_eq!(scaling[0].magnitude(ComboStrength::Grant), None);
        assert_eq!(conferred[0].magnitude(ComboStrength::Grant), Some(2500));
    }

    #[test]
    fn a_band_outside_the_row_reads_as_absent() {
        let rows = NyancomboParam::parse(SCALING_ROW, None).unwrap();

        assert_eq!(rows[0].magnitude(ComboStrength::Unknown(9)), None);
        assert_eq!(rows[0].magnitude(ComboStrength::Unknown(-1)), None);
        assert_eq!(rows[0].magnitude(ComboStrength::Unknown(0)), Some(10));
    }

    #[test]
    fn an_unreadable_cell_is_held_in_place() {
        let rows = NyancomboParam::parse("10\tx\t20", None).unwrap();

        assert_eq!(rows[0].magnitudes, [Some(10), None, Some(20)]);
        assert_eq!(rows[0].magnitude(ComboStrength::Large), Some(20));
    }

    #[test]
    fn a_blank_line_holds_its_place_in_the_file() {
        let rows = NyancomboParam::parse(format!("{SCALING_ROW}\n\n{CONFERRED_ROW}"), None).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[1], NyancomboParam::default());
        assert_eq!(rows[2].magnitudes.len(), 6);
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(NyancomboParam::parse("", None), Err(NyancomboParamError::EmptyFile));
        assert_eq!(NyancomboParam::parse("\n\n", None), Err(NyancomboParamError::EmptyFile));
    }
}
