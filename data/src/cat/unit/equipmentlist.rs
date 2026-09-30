//! Parsing of the Talent Orb definitions.
//!
//! The engine declares one record of `equipmentlist.json` per orb, and an orb's
//! identifier is its position in the file.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Represents errors that can occur while parsing the Talent Orb definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EquipmentListError {
    /// The supplied bytes were not a JSON document.
    InvalidJson,
    /// The document carried no `ID` array.
    MissingList,
}

impl fmt::Display for EquipmentListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson => write!(f, "The provided file bytes were not a valid JSON document."),
            Self::MissingList => write!(f, "The provided document contained no Talent Orb list."),
        }
    }
}

impl std::error::Error for EquipmentListError {}

/// One Talent Orb definition.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    /// The grade the orb is ranked at, as a row of `equipmentgrade.csv`.
    #[serde(rename = "gradeID")]
    pub grade_id: Option<i32>,
    /// The effect the orb confers.
    pub content: Option<i32>,
    /// The magnitudes the effect is applied with.
    #[serde(default)]
    pub value: Vec<i32>,
    /// The trait the effect is limited to, absent on an orb that names none.
    pub attribute: Option<i32>,
}

impl Equipment {
    /// Parses the Talent Orb definitions into one record per orb.
    ///
    /// A record that does not read as an orb is held as a default record, so a
    /// record's position in the returned vector is always the orb identifier.
    ///
    /// # Arguments
    /// * `bytes` - The raw, decrypted byte slice of the `equipmentlist.json` file.
    ///
    /// # Returns
    /// A `Result` containing the parsed records in file order on success, or an
    /// `EquipmentListError` if the document could not be read.
    pub fn parse<B: AsRef<[u8]>>(bytes: B) -> Result<Vec<Self>, EquipmentListError> {
        parse_inner(bytes.as_ref())
    }
}

fn parse_inner(bytes: &[u8]) -> Result<Vec<Equipment>, EquipmentListError> {
    let root: serde_json::Value = serde_json::from_slice(bytes).map_err(|_| EquipmentListError::InvalidJson)?;
    let list = root.get("ID").and_then(serde_json::Value::as_array).ok_or(EquipmentListError::MissingList)?;

    Ok(list
        .iter()
        .map(|record| serde_json::from_value(record.clone()).unwrap_or_default())
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two records of a real `equipmentlist.json`, one naming a trait and one not.
    const SAMPLE: &str = r#"{"ID":[{"gradeID":0,"content":0,"value":[100],"attribute":0},{"gradeID":3,"content":18,"value":[14]}]}"#;

    #[test]
    fn a_record_lands_field_for_field() {
        let orbs = Equipment::parse(SAMPLE).unwrap();

        assert_eq!(orbs[0], Equipment { grade_id: Some(0), content: Some(0), value: vec![100], attribute: Some(0) });
    }

    #[test]
    fn an_orb_that_names_no_trait_reads_as_absent() {
        let orbs = Equipment::parse(SAMPLE).unwrap();

        assert_eq!(orbs[1].attribute, None);
        assert_eq!(orbs[1].content, Some(18));
    }

    #[test]
    fn an_unreadable_record_holds_its_place() {
        let orbs = Equipment::parse(r#"{"ID":[7,{"gradeID":1,"content":2,"value":[3]}]}"#).unwrap();

        assert_eq!(orbs.len(), 2);
        assert_eq!(orbs[0], Equipment::default());
        assert_eq!(orbs[1].grade_id, Some(1));
    }

    #[test]
    fn a_document_without_the_list_is_rejected() {
        assert_eq!(Equipment::parse("{}"), Err(EquipmentListError::MissingList));
        assert_eq!(Equipment::parse("nope"), Err(EquipmentListError::InvalidJson));
    }
}
