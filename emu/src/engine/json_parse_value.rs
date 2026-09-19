use crate::Fault;

use super::{json_parse_number, json_parse_string, JsonNode, JsonParser};

pub fn json_parse_value(parser: &mut JsonParser, token: i32) -> Result<Option<JsonNode>, Fault> {
    match token {
        8 => json_parse_string(parser),
        0xc => {
            parser.cursor -= 1;
            json_parse_number(parser)
        }
        0xa => Ok(Some(JsonNode::Bool(1))),
        0xb => Ok(Some(JsonNode::Bool(0))),
        9 => Ok(Some(JsonNode::Null)),
        _ => Ok(None),
    }
}
