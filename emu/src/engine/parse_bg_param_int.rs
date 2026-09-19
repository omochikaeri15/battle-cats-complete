use std::collections::BTreeMap;

use crate::Fault;

use super::{
    BgParamSpec, JsonNode, bg_param_base_id, json_container_as_int, json_container_as_string,
    json_string_as_int, json_string_as_string, json_value_as_int, json_value_as_string,
    parse_bg_param_bound_int,
};

const SITE: &str = "parse_bg_param_int";

pub fn parse_bg_param_int(
    spec: &mut BgParamSpec<i32>,
    node: Option<&BTreeMap<Vec<u8>, JsonNode>>,
) -> Result<(), Fault> {
    spec.enabled = 0;
    spec.value = 0;
    spec.values.clear();
    spec.base = 0;
    spec.has_min = 0;
    spec.min = 0;
    spec.min_base = 0;
    spec.has_max = 0;
    spec.max = 0;
    spec.max_base = 0;
    spec.rand_group = -1;

    let Some(node) = node else {
        return Ok(());
    };

    spec.enabled = 1;

    if let Some(found) = node.get(b"value".as_slice()) {
        spec.value = match found {
            JsonNode::String(text) => json_string_as_int(text)?,
            JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
            _ => json_value_as_int(found),
        } as i32;
    } else if node.contains_key(b"values".as_slice()) {
        let mut index = 0usize;

        loop {
            let Some(JsonNode::Array(values)) = node.get(b"values".as_slice()) else {
                return Err(Fault::NullPointer { site: SITE });
            };

            if index >= values.len() {
                break;
            }

            let found = &values[index];
            let value = match found {
                JsonNode::String(text) => json_string_as_int(text)?,
                JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                _ => json_value_as_int(found),
            } as i32;

            spec.values.push(value);
            index += 1;
        }
    } else {
        if let Some(JsonNode::Object(bound)) = node.get(b"min".as_slice()) {
            parse_bg_param_bound_int(&mut spec.has_min, &mut spec.min, &mut spec.min_base, bound)?;
        }

        if let Some(JsonNode::Object(bound)) = node.get(b"max".as_slice()) {
            parse_bg_param_bound_int(&mut spec.has_max, &mut spec.max, &mut spec.max_base, bound)?;
        }

        if let Some(found) = node.get(b"randGroup".as_slice()) {
            spec.rand_group = match found {
                JsonNode::String(text) => json_string_as_int(text)?,
                JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_int(),
                _ => json_value_as_int(found),
            } as i32;
        }
    }

    let name = node
        .get(b"base".as_slice())
        .map_or(Vec::new(), |found| match found {
            JsonNode::String(text) => json_string_as_string(text),
            JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
            _ => json_value_as_string(found),
        });

    spec.base = bg_param_base_id(&name);

    Ok(())
}
