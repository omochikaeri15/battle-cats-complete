use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::file::{self, Separator};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EquipmentSlotError {
    EmptyFile,
}

impl fmt::Display for EquipmentSlotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(f, "The provided file bytes contained no valid Talent Orb slot data."),
        }
    }
}

impl std::error::Error for EquipmentSlotError {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentSlot {
    pub unit_id: Option<i32>,
    pub slot_count: Option<i32>,
    pub rest: Vec<Option<i32>>,
}

impl EquipmentSlot {
    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Vec<Self>, EquipmentSlotError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<Vec<EquipmentSlot>, EquipmentSlotError> {
    let file_content = file::scrub(bytes);
    let delimiter = file::resolve(separator, &file_content);

    let rows: Vec<EquipmentSlot> = file_content
        .lines()
        .skip(1)
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let mut cells = line.split(delimiter).map(|cell| cell.trim().parse::<i32>().ok());

            EquipmentSlot {
                unit_id: cells.next().flatten(),
                slot_count: cells.next().flatten(),
                rest: cells.collect(),
            }
        })
        .collect();

    if rows.is_empty() {
        return Err(EquipmentSlotError::EmptyFile);
    }

    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "//header\n30,1\n83,2,0,1\n";

    #[test]
    fn a_row_lands_cell_for_cell() {
        let rows = EquipmentSlot::parse(SAMPLE, None).unwrap();

        assert_eq!(rows[0], EquipmentSlot { unit_id: Some(30), slot_count: Some(1), rest: Vec::new() });
        assert_eq!(rows[1], EquipmentSlot { unit_id: Some(83), slot_count: Some(2), rest: vec![Some(0), Some(1)] });
    }

    #[test]
    fn the_header_row_is_skipped_unread() {
        let rows = EquipmentSlot::parse("1,2\n30,1\n", None).unwrap();

        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].unit_id, Some(30));
    }

    #[test]
    fn a_file_without_rows_is_rejected() {
        assert_eq!(EquipmentSlot::parse("//header\n", None), Err(EquipmentSlotError::EmptyFile));
    }
}
