use std::collections::BTreeMap;

use crate::Fault;

use super::{JsonNode, JsonParser, json_next_token, json_parse_object, json_parse_value};

pub fn json_parse_array(parser: &mut JsonParser, items: &mut Vec<JsonNode>) -> Result<bool, Fault> {
    let mut token = json_next_token(parser);

    loop {
        match token {
            1 | 7 => {}
            3 => {
                let mut members = BTreeMap::new();

                if !json_parse_object(parser, &mut members)? {
                    return Ok(false);
                }

                items.push(JsonNode::Object(members));
            }
            5 => {
                let mut children = Vec::new();

                if !json_parse_array(parser, &mut children)? {
                    return Ok(false);
                }

                items.push(JsonNode::Array(children));
            }
            6 => return Ok(true),
            0 | 4 | 0xd => return Ok(false),
            _ => {
                let Some(value) = json_parse_value(parser, token)? else {
                    return Ok(false);
                };

                items.push(value);
            }
        }

        token = json_next_token(parser);
    }
}
