use crate::Fault;

use super::{JsonNode, json_container_as_int, json_string_as_int, json_value_as_int};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Medal {
    pub kind: i32,
    pub condition_maps: Vec<i32>,
    pub medals: Vec<i32>,
    pub limit: i32,
    pub condition_stage: i32,
    pub condition_star: i32,
    pub line: i32,
    pub grade: i32,
    pub maps: Vec<i32>,
    pub stage: i32,
    pub treasure: i32,
    pub star: i32,
    pub action: i32,
    pub count: i32,
    pub chara: i32,
    pub name: Vec<u8>,
    pub explanation: Vec<u8>,
}

pub fn parse_medal_entry(medal: &mut Medal, node: Option<&JsonNode>) -> Result<(), Fault> {
    medal.condition_maps.clear();

    let Some(JsonNode::Object(fields)) = node else {
        return Ok(());
    };

    match fields.get(b"condition".as_slice()) {
        Some(JsonNode::Object(inner)) => {
            medal.limit = match inner.get(b"limit".as_slice()) {
                Some(found) => match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }?,
                None => -1,
            } as i32;
            medal.condition_stage = match inner.get(b"stage".as_slice()) {
                Some(found) => match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }?,
                None => -1,
            } as i32;
            medal.condition_star = match inner.get(b"star".as_slice()) {
                Some(found) => match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }?,
                None => 0,
            } as i32;

            if let Some(JsonNode::Array(values)) = inner.get(b"map".as_slice()) {
                let mut index = 0usize;

                while index < values.len() {
                    let value = match values.get(index) {
                        Some(found) => match found {
                            JsonNode::String(text) => json_string_as_int(text),
                            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                            _ => Ok(json_value_as_int(found)),
                        }?,
                        None => 0,
                    } as i32;

                    medal.condition_maps.push(value);
                    index += 1;
                }
            }
        }
        _ => {
            medal.limit = -1;
            medal.condition_star = 0;
        }
    }

    medal.line = match fields.get(b"line".as_slice()) {
        Some(found) => match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        }?,
        None => 0,
    } as i32;
    medal.grade = match fields.get(b"grade".as_slice()) {
        Some(found) => match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        }?,
        None => 0,
    } as i32;

    if let Some(JsonNode::Array(values)) = fields.get(b"map".as_slice()) {
        medal.kind = 0;
        medal.maps.clear();

        let mut index = 0usize;

        while index < values.len() {
            let value = match values.get(index) {
                Some(found) => match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }?,
                None => 0,
            } as i32;

            medal.maps.push(value);
            index += 1;
        }

        medal.stage = match fields.get(b"stage".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => -1,
        } as i32;
        medal.treasure = match fields.get(b"treasure".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => -1,
        } as i32;
        medal.star = match fields.get(b"star".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => 0,
        } as i32;

        return Ok(());
    }

    if fields.contains_key(b"action".as_slice()) {
        medal.kind = 1;
        medal.action = match fields.get(b"action".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => 0,
        } as i32;
        medal.count = match fields.get(b"count".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => 0,
        } as i32;

        return Ok(());
    }

    if fields.contains_key(b"chara".as_slice()) {
        medal.kind = 2;
        medal.chara = match fields.get(b"chara".as_slice()) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => 0,
        } as i32;

        return Ok(());
    }

    let Some(JsonNode::Array(values)) = fields.get(b"medal".as_slice()) else {
        return Ok(());
    };

    medal.kind = 3;
    medal.medals.clear();

    let mut index = 0usize;

    while index < values.len() {
        let value = match values.get(index) {
            Some(found) => match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            }?,
            None => 0,
        } as i32;

        medal.medals.push(value);
        index += 1;
    }

    Ok(())
}
