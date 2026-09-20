use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
    string_to_int,
};

pub fn load_clear_count_reward_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.clear_count_rewards.clear();

    let Some(bytes) = open_asset_stream(ctx, b"MapStageDataClearCountReward.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(maps)) = root.get(b"MapID".as_slice()) else {
        return Ok(());
    };

    for (map_key, map_node) in maps {
        let map = string_to_int(map_key)?;
        let JsonNode::Object(stages) = map_node else {
            continue;
        };

        for (stage_key, stage_node) in stages {
            let stage = string_to_int(stage_key)?;
            let JsonNode::Object(entry) = stage_node else {
                continue;
            };
            let Some(JsonNode::Array(rows)) = entry.get(b"data".as_slice()) else {
                continue;
            };

            for row in rows {
                let JsonNode::Object(fields) = row else {
                    continue;
                };
                let item = fields
                    .get(b"DropItemID".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })? as i32;
                let quantity = fields
                    .get(b"Quantity".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })? as i32;

                ctx.clear_count_rewards
                    .entry(map)
                    .or_default()
                    .entry(stage)
                    .or_default()
                    .push([item, quantity]);
            }
        }
    }

    Ok(())
}
