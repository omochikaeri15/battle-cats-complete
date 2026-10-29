use crate::Fault;

use super::{
    JsonNode, PointRuleDetail, json_container_as_int, json_string_as_int, json_value_as_int,
};

pub fn parse_point_rule_params(
    detail: &mut PointRuleDetail,
    params: Option<&Vec<JsonNode>>,
) -> Result<bool, Fault> {
    let Some(params) = params else {
        return Ok(false);
    };

    detail.kill_point_base = params.first().map_or(Ok(0), |found| match found {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(found)),
    })? as i32;

    if params.len() >= 2 {
        let mut index = 1usize;

        while index < params.len() {
            let threshold = params.get(index.wrapping_add(1)).map_or(Ok(0), |found| {
                match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }
            })? as i32;
            let points = params.get(index).map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

            detail.bands.insert(threshold, points);
            index += 2;
        }
    }

    Ok(true)
}
