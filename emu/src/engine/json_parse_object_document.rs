use crate::Fault;

use super::{JsonNode, json_parse_document};

pub fn json_parse_object_document(source: Option<Vec<u8>>) -> Result<Option<JsonNode>, Fault> {
    Ok(json_parse_document(source)?.filter(|root| matches!(root, JsonNode::Object(_))))
}
