use std::collections::BTreeMap;

use crate::Fault;

use super::{JsonNode, JsonParser, json_next_token, json_parse_array, json_parse_value};

pub fn json_parse_object(
    parser: &mut JsonParser,
    members: &mut BTreeMap<Vec<u8>, JsonNode>,
) -> Result<bool, Fault> {
    let mut token = json_next_token(parser);
    let mut key: Vec<u8> = Vec::new();

    loop {
        match token {
            7 => {}
            1 => {
                if !key.is_empty() {
                    return Ok(false);
                }
            }
            5 => {
                if key.is_empty() {
                    return Ok(false);
                }

                let mut children = Vec::new();

                if !json_parse_array(parser, &mut children)? {
                    return Ok(false);
                }

                members.insert(std::mem::take(&mut key), JsonNode::Array(children));
            }
            3 => {
                if key.is_empty() {
                    return Ok(false);
                }

                let mut children = BTreeMap::new();

                if !json_parse_object(parser, &mut children)? {
                    return Ok(false);
                }

                members.insert(std::mem::take(&mut key), JsonNode::Object(children));
            }
            2 => {
                if key.is_empty() {
                    return Ok(false);
                }
            }
            4 => return Ok(true),
            0 | 6 | 0xd => return Ok(false),
            _ => {
                if key.is_empty() {
                    match json_parse_value(parser, token)? {
                        Some(JsonNode::String(text)) => key = text,
                        _ => return Ok(false),
                    }
                } else {
                    let Some(value) = json_parse_value(parser, token)? else {
                        return Ok(false);
                    };

                    members.insert(std::mem::take(&mut key), value);
                }
            }
        }

        token = json_next_token(parser);
    }
}
