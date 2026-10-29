use crate::Fault;

use super::{JsonNode, PointRule, PointRuleTable, parse_point_rule_entry, string_to_int};

pub fn parse_point_stage_map(
    table: &mut PointRuleTable,
    node: Option<&JsonNode>,
) -> Result<bool, Fault> {
    let Some(JsonNode::Object(fields)) = node else {
        return Ok(false);
    };

    if let Some(JsonNode::Object(stages)) = fields.get(b"StageIndex".as_slice()) {
        for (stage_key, stage_node) in stages {
            let stage = string_to_int(stage_key)?;
            let mut entry = PointRule::default();
            let listed = match stage_node {
                JsonNode::Object(_) => Some(stage_node),
                _ => None,
            };

            if parse_point_rule_entry(&mut entry, listed)? {
                table.entries.insert(stage, entry);
            }
        }
    }

    Ok(true)
}
