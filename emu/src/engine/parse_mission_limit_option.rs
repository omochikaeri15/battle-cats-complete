use crate::Fault;

use super::{
    JsonNode, json_container_as_int, json_container_as_string, json_string_as_int,
    json_string_as_string, json_value_as_int, json_value_as_string,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct MissionLimitOption {
    pub item_limit: i32,
    pub cat_castle_limit: i32,
    pub form_lv_limit: i32,
    pub rarity_limit: Vec<i32>,
    pub slot_limit: Vec<i32>,
    pub slot_option_rule: Vec<Vec<u8>>,
    pub cost_limit_min: Vec<i32>,
    pub cost_limit_max: Vec<i32>,
    pub chara_limit: Vec<i32>,
    pub map: Vec<i32>,
    pub stage: Vec<i32>,
}

pub fn parse_mission_limit_option(
    out: &mut MissionLimitOption,
    node: Option<&JsonNode>,
) -> Result<(), Fault> {
    out.item_limit = -1;
    out.cat_castle_limit = -1;
    out.form_lv_limit = -1;
    out.rarity_limit.clear();
    out.slot_limit.clear();
    out.slot_option_rule.clear();
    out.cost_limit_min.clear();
    out.cost_limit_max.clear();
    out.chara_limit.clear();
    out.map.clear();
    out.stage.clear();

    let Some(JsonNode::Object(fields)) = node else {
        return Err(Fault::null_pointer());
    };

    if let Some(JsonNode::Array(values)) = fields.get(b"Map".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.map.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"Stage".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.stage.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    out.item_limit = fields
        .get(b"ItemLimit".as_slice())
        .map_or(Ok(-1), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    out.cat_castle_limit = fields
        .get(b"CatCastleLimit".as_slice())
        .map_or(Ok(-1), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    out.form_lv_limit = fields
        .get(b"FormLvLimit".as_slice())
        .map_or(Ok(-1), |found| match found {
            JsonNode::String(text) => json_string_as_int(text),
            JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
            _ => Ok(json_value_as_int(found)),
        })? as i32;

    if let Some(JsonNode::Array(values)) = fields.get(b"RarityLimit".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.rarity_limit.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"SlotLimit".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.slot_limit.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"SlotOptionRule".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.slot_option_rule.push(match element {
                JsonNode::String(text) => json_string_as_string(text),
                JsonNode::Array(_) | JsonNode::Object(_) => json_container_as_string(),
                _ => json_value_as_string(element),
            });
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"CostLimitMin".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.cost_limit_min.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"CostLimitMax".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.cost_limit_max.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    if let Some(JsonNode::Array(values)) = fields.get(b"CharaLimit".as_slice()) {
        let mut slot = 0usize;

        while slot < values.len() {
            let element = values.get(slot).ok_or(Fault::index_out_of_range(slot as i64, values.len() as i64))?;

            out.chara_limit.push(match element {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(element)),
            }? as i32);
            slot += 1;
        }
    }

    Ok(())
}
