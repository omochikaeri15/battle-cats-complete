use super::JsonNode;

pub fn json_value_as_bool(node: &JsonNode) -> bool {
    match node {
        JsonNode::Bool(value) => *value != 0,
        JsonNode::Int(value) => *value != 0,
        JsonNode::Uint(value) => *value != 0,
        JsonNode::Double(value) => *value != 0.0,
        _ => false,
    }
}
