use std::collections::BTreeMap;

use crate::Fault;

use super::{
    JsonNode, bg_param_base_id, json_container_as_double, json_container_as_string,
    json_string_as_double, json_string_as_string, json_value_as_double, json_value_as_string,
};

pub fn parse_bg_param_bound_float(
    has: &mut u8,
    value: &mut f32,
    base: &mut i32,
    node: &BTreeMap<Vec<u8>, JsonNode>,
) -> Result<(), Fault> {
    *value = 0.0;
    *base = 0;
    *has = 1;

    if let Some(found) = node.get(b"value".as_slice()) {
        *value = match found {
            JsonNode::String(text) => json_string_as_double(text)?,
            JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_double(),
            _ => json_value_as_double(found),
        } as f32;
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
