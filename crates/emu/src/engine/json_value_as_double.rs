use super::JsonNode;

pub fn json_value_as_double(node: &JsonNode) -> f64 {
    match node {
        JsonNode::Bool(value) => *value as i8 as f64,
        JsonNode::Int(value) => *value as f64,
        JsonNode::Uint(value) => *value as f64,
        JsonNode::Double(value) => *value,
        _ => 0.0,
    }
}
