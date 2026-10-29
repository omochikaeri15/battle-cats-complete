use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum EquipmentListError {
    InvalidJson,
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equipment {
    #[serde(rename = "gradeID")]
    pub grade_id: Option<i32>,
    pub content: Option<i32>,
    #[serde(default)]
    pub value: Vec<i32>,
    pub attribute: Option<i32>,
}

impl Equipment {
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
