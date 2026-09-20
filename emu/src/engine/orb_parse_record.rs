use crate::Fault;

use super::{
    JsonNode, OrbDef, json_container_as_int, json_string_as_int, json_value_as_int,
};

const SITE: &str = "orb_parse_record";

pub fn orb_parse_record(record: &mut OrbDef, node: Option<&JsonNode>) -> Result<(), Fault> {
    let Some(JsonNode::Object(fields)) = node else {
        return Err(Fault::NullPointer { site: SITE });
    };

    record.grade = fields
        .get(b"gradeID".as_slice())
        .map_or(Ok(0), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    record.trait_index = fields
        .get(b"attribute".as_slice())
        .map_or(Ok(0xc), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    record.abil = fields
        .get(b"content".as_slice())
        .map_or(Ok(0), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    let mut index = 0usize;

    loop {
        let Some(JsonNode::Array(values)) = fields.get(b"value".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        if index >= values.len() {
            return Ok(());
        }

        let Some(JsonNode::Array(values)) = fields.get(b"value".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        let element = values.get(index).ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: index as i64,
            limit: values.len() as i64,
        })?;

        let value = match element {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(element)),
        }? as i32;

        record.values.push(value);
        index += 1;
    }
}
