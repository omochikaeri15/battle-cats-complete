use crate::Fault;

use super::{
    JsonNode, PointRule, PointRuleDetail, json_container_as_int, json_container_as_string,
    json_string_as_int, json_string_as_string, json_value_as_int, json_value_as_string,
    parse_point_rule_params,
};

pub fn parse_point_rule_entry(entry: &mut PointRule, node: Option<&JsonNode>) -> Result<bool, Fault> {
    let Some(JsonNode::Object(fields)) = node else {
        return Ok(false);
    };

    entry.kind = fields
        .get(b"RuleType".as_slice())
        .map_or(Ok(0), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;
    entry.page_headline = match fields.get(b"PageHeadline".as_slice()) {
        Some(JsonNode::String(text)) => json_string_as_string(text),
        Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
        Some(found) => json_value_as_string(found),
        None => Vec::new(),
    };

    if let Some(JsonNode::Array(values)) = fields.get(b"RuleHeadlineLabel".as_slice()) {
        let mut index = 0usize;

        while index < values.len() {
            let text = match values.get(index) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            };

            entry.headline_labels.push(text);
            index += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"RuleExplanationLabel".as_slice()) {
        let mut index = 0usize;

        while index < values.len() {
            let text = match values.get(index) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            };

            entry.explanation_labels.push(text);
            index += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"RuleExplanationTypeLabel".as_slice()) {
        let mut index = 0usize;

        while index < values.len() {
            let value = values.get(index).map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

            entry.explanation_types.push(value);
            index += 1;
        }
    }

    if entry.kind == 0 {
        entry.detail = Some(PointRuleDetail::default());

        let params = match fields.get(b"Parameters".as_slice()) {
            Some(JsonNode::Array(values)) => Some(values),
            _ => None,
        };

        if let Some(detail) = entry.detail.as_mut() {
            parse_point_rule_params(detail, params)?;
        }
    }

    Ok(true)
}
