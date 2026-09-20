use crate::Fault;

use super::{
    AppContext, EventItemStore, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
    string_to_int,
};

pub fn load_point_stage_settings(
    ctx: &mut AppContext,
    store: &mut EventItemStore,
) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"PointStageSettings.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Ok(());
    };
    let Some(JsonNode::Object(points)) = root.get(b"PointID".as_slice()) else {
        return Ok(());
    };

    for (point_key, entry) in points {
        let point_id = string_to_int(point_key)?;
        let JsonNode::Object(fields) = entry else {
            break;
        };
        let Some(JsonNode::Array(values)) = fields.get(b"MapID".as_slice()) else {
            continue;
        };
        let mut index = 0usize;

        while index < values.len() {
            let value = values.get(index).map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

            store.point_maps.entry(point_id).or_default().push(value);
            index += 1;
        }
    }

    Ok(())
}
