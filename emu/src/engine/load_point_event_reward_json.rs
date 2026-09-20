use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int, json_string_as_string,
    json_value_as_int, json_value_as_string, open_asset_stream, string_to_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PointReward {
    pub reward_id: i32,
    pub point: i32,
    pub kind: i32,
    pub id: i32,
    pub quantity: i32,
    pub label0: Vec<u8>,
    pub label1: Vec<u8>,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct PointEventReward {
    pub reset_map_id: i32,
    pub rewards: Vec<PointReward>,
}

pub fn load_point_event_reward_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.point_event_rewards.clear();

    let Some(bytes) = open_asset_stream(ctx, b"PointEventReward.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(points)) = root.get(b"PointID".as_slice()) else {
        return Ok(());
    };

    for (point_key, entry) in points {
        let point_id = string_to_int(point_key)?;
        let reset_map_id = match entry {
            JsonNode::Object(fields) => fields
                .get(b"ResetMapID".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })?,
            _ => 0,
        } as i32;

        ctx.point_event_rewards
            .entry(point_id)
            .or_default()
            .reset_map_id = reset_map_id;

        let listed = match entry {
            JsonNode::Object(fields) => match fields.get(b"Rewards".as_slice()) {
                Some(JsonNode::Array(values)) => Some(values),
                _ => None,
            },
            _ => None,
        };
        let Some(values) = listed else {
            continue;
        };
        let mut index = 0usize;

        while index < values.len() {
            ctx.point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .push(PointReward::default());

            let reward_id = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"RewardID".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.reward_id = reward_id;
            }

            let point = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"Point".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.point = point;
            }

            let kind = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"Type".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.kind = kind;
            }

            let id = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"ID".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.id = id;
            }

            let quantity = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"Quantity".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.quantity = quantity;
            }

            let label0 = match values.get(index) {
                Some(JsonNode::Object(fields)) => match fields.get(b"Label0".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.label0 = label0;
            }

            let label1 = match values.get(index) {
                Some(JsonNode::Object(fields)) => match fields.get(b"Label1".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                },
                _ => Vec::new(),
            };

            if let Some(last) = ctx
                .point_event_rewards
                .entry(point_id)
                .or_default()
                .rewards
                .last_mut()
            {
                last.label1 = label1;
            }

            index += 1;
        }
    }

    Ok(())
}
