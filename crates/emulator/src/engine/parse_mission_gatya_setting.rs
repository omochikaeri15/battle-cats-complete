use crate::Fault;

use super::{JsonNode, json_container_as_int, json_string_as_int, json_value_as_int};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionGatyaItem {
    pub kind: i32,
    pub item_id: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionGatyaSetting {
    pub rarity: Vec<i32>,
    pub series: Vec<i32>,
    pub items: Vec<MissionGatyaItem>,
}

pub fn parse_mission_gatya_setting(
    out: &mut MissionGatyaSetting,
    node: Option<&JsonNode>,
) -> Result<(), Fault> {
    out.rarity.clear();
    out.series.clear();
    out.items.clear();

    let Some(JsonNode::Object(fields)) = node else {
        return Err(Fault::null_pointer());
    };

    if let Some(JsonNode::Array(values)) = fields.get(b"Rarity".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.rarity.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"Series".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.series.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    let Some(JsonNode::Array(entries)) = fields.get(b"Item".as_slice()) else {
        return Ok(());
    };

    let mut slot = 0usize;

    while slot < entries.len() {
        let element = entries.get(slot).ok_or(Fault::index_out_of_range(slot as i64, entries.len() as i64))?;
        let JsonNode::Object(item) = element else {
            return Err(Fault::null_pointer());
        };
        let kind = item
            .get(b"Type".as_slice())
            .map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;
        let item_id = item
            .get(b"ItemID".as_slice())
            .map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

        out.items.push(MissionGatyaItem { kind, item_id });
        slot += 1;
    }

    Ok(())
}
