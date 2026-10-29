use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
};

pub fn load_treasure_gauge_json(ctx: &mut AppContext) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"Treasure_gauge.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let max_value = root.get(b"MaxValue".as_slice()).map_or(Ok(0), |found| match found {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(found)),
    })? as i32;

    ctx.treasure_gauge.max_value = max_value;
    ctx.treasure_gauge.gains.clear();

    let stage_clear = root.get(b"StageClear".as_slice()).map_or(Ok(0), |found| match found {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(found)),
    })? as i32;

    *ctx.treasure_gauge.gains.entry(0).or_insert(0) = stage_clear;

    let rank_up = root.get(b"RankUp".as_slice()).map_or(Ok(0), |found| match found {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(found)),
    })? as i32;

    *ctx.treasure_gauge.gains.entry(1).or_insert(0) = rank_up;

    let gamatoto = root.get(b"Gamatoto".as_slice()).map_or(Ok(0), |found| match found {
        JsonNode::String(text) => json_string_as_int(text),
        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
        _ => Ok(json_value_as_int(found)),
    })? as i32;

    *ctx.treasure_gauge.gains.entry(2).or_insert(0) = gamatoto;

    Ok(())
}
