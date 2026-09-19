use crate::Fault;

use super::{json_parse_document, JsonNode};

pub fn json_parse_object_document(source: Option<Vec<u8>>) -> Result<Option<JsonNode>, Fault> {
    Ok(json_parse_document(source)?.filter(|root| matches!(root, JsonNode::Object(_))))
}
