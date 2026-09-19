use std::collections::BTreeMap;

use crate::Fault;

use super::{json_next_token, json_parse_array, json_parse_object, JsonNode, JsonParser};

pub fn json_parse(parser: &mut JsonParser) -> Result<bool, Fault> {
    parser.cursor = parser.begin;

    let mut token = json_next_token(parser);

    while token == 7 {
        token = json_next_token(parser);
    }

    match token {
        3 => {
            let mut members = BTreeMap::new();
            let parsed = json_parse_object(parser, &mut members)?;

            parser.root = Some(JsonNode::Object(members));

            Ok(parsed)
        }
        5 => {
            let mut items = Vec::new();
            let parsed = json_parse_array(parser, &mut items)?;

            parser.root = Some(JsonNode::Array(items));

            Ok(parsed)
        }
        _ => Ok(false),
    }
}
