use super::JsonNode;

pub fn json_value_as_string(node: &JsonNode) -> Vec<u8> {
    match node {
        JsonNode::Null => b"null".to_vec(),
        JsonNode::Bool(value) => if *value == 0 { b"false".to_vec() } else { b"true".to_vec() },
        JsonNode::Int(value) => value.to_string().into_bytes(),
        JsonNode::Uint(value) => value.to_string().into_bytes(),
        JsonNode::Double(value) => format!("{value:.6}").into_bytes(),
        _ => Vec::new(),
    }
}
