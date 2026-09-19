use crate::operation;

use super::JsonNode;

pub fn json_value_as_int(node: &JsonNode) -> i64 {
    match node {
        JsonNode::Bool(value) => *value as i64,
        JsonNode::Int(value) => *value,
        JsonNode::Uint(value) => *value as i64,
        JsonNode::Double(value) => operation::cvttsd2si_64(*value),
        _ => 0,
    }
}
