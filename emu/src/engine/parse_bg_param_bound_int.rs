use std::collections::BTreeMap;

use crate::Fault;

use super::{
    JsonNode, bg_param_base_id, json_container_as_int, json_container_as_string,
    json_string_as_int, json_string_as_string, json_value_as_int, json_value_as_string,
};

pub fn parse_bg_param_bound_int(
    has: &mut u8,
    value: &mut i32,
    base: &mut i32,
    node: &BTreeMap<Vec<u8>, JsonNode>,
) -> Result<(), Fault> {
    *value = 0;
    *base = 0;
    *has = 1;

    if let Some(found) = node.get(b"value".as_slice()) {
        *value = match found {
            JsonNode::String(text) => json_string_as_int(text)?,
            JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
            _ => json_value_as_int(found),
        } as i32;
    }

    let name = node
        .get(b"base".as_slice())
        .map_or(Vec::new(), |found| match found {
            JsonNode::String(text) => json_string_as_string(text),
            JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
            _ => json_value_as_string(found),
        });

    *base = bg_param_base_id(&name);

    Ok(())
}
