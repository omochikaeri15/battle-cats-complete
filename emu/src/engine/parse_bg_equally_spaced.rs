use std::collections::BTreeMap;

use crate::Fault;

use super::{bg_param_base_id, json_container_as_int, json_container_as_string, json_string_as_int, json_string_as_string, json_value_as_int, json_value_as_string, JsonNode};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct BgEquallySpaced {
    pub pos1: i32,
    pub pos2: i32,
    pub value: i32,
    pub base: i32,
}

pub fn parse_bg_equally_spaced(spaced: &mut BgEquallySpaced, node: Option<&BTreeMap<Vec<u8>, JsonNode>>) -> Result<(), Fault> {
    *spaced = BgEquallySpaced { pos1: 0, pos2: 0, value: 1, base: 1 };

    let Some(node) = node else {
        return Ok(());
    };

    spaced.pos1 = node.get(b"pos1".as_slice()).map_or(Ok(0), |value| match value {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(value)),
    })? as i32;
    spaced.pos2 = node.get(b"pos2".as_slice()).map_or(Ok(0), |value| match value {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(value)),
    })? as i32;
    spaced.value = node.get(b"value".as_slice()).map_or(Ok(0), |value| match value {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(value)),
    })? as i32;

    let base = node.get(b"base".as_slice()).map_or(Vec::new(), |value| match value {
        JsonNode::String(text) => json_string_as_string(text),
        JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
        _ => json_value_as_string(value),
    });

    spaced.base = bg_param_base_id(&base);

    Ok(())
}
