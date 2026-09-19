use crate::Fault;

use super::{json_parser_new, JsonNode};

pub fn json_parse_document(source: Option<Vec<u8>>) -> Result<Option<JsonNode>, Fault> {
    let Some(source) = source else {
        return Ok(None);
    };

    Ok(json_parser_new(source)?.root.take())
}
