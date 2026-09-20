use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int, json_string_as_string,
    json_value_as_int, json_value_as_string, open_asset_stream, string_to_int,
};

pub fn load_map_option_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.unlock_groups.clear();

    let Some(bytes) = open_asset_stream(ctx, b"MapConditions.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(data)) = root.get(b"data".as_slice()) else {
        return Ok(());
    };

    for (map_key, entry) in data {
        let map_id = string_to_int(map_key)?;
        let conditions = match entry {
            JsonNode::Object(fields) => match fields.get(b"condition".as_slice()) {
                Some(JsonNode::Array(values)) => Some(values),
                _ => None,
            },
            _ => None,
        };

        if let Some(values) = conditions {
            let mut index = 0usize;

            while index < values.len() {
                let value = match entry {
                    JsonNode::Object(fields) => match fields.get(b"condition".as_slice()) {
                        Some(JsonNode::Array(listed)) => listed.get(index),
                        _ => None,
                    },
                    _ => None,
                };
                let value = value.map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.unlock_groups
                    .entry(map_id)
                    .or_default()
                    .conditions
                    .push(value);

                index += 1;
            }
        }

        let required = match entry {
            JsonNode::Object(fields) => fields
                .get(b"count".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })?,
            _ => 0,
        } as i32;

        ctx.unlock_groups.entry(map_id).or_default().required = required;

        let stage = match entry {
            JsonNode::Object(fields) => fields
                .get(b"stage".as_slice())
                .map_or(Ok(-1), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })?,
            _ => -1,
        } as i32;

        ctx.unlock_groups.entry(map_id).or_default().stage = stage;

        let flag_id = match entry {
            JsonNode::Object(fields) => fields.get(b"serverSetting".as_slice()).map_or(
                Ok(-1),
                |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                },
            )?,
            _ => -1,
        } as i32;

        ctx.unlock_groups.entry(map_id).or_default().flag_id = flag_id;

        let limit_message = match entry {
            JsonNode::Object(fields) => match fields.get(b"limitMessage".as_slice()) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };

        ctx.unlock_groups.entry(map_id).or_default().limit_message = limit_message;

        let hidden = match entry {
            JsonNode::Object(fields) => fields
                .get(b"hidden".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })?,
            _ => 0,
        } as i32;

        ctx.unlock_groups.entry(map_id).or_default().hidden = u8::from(hidden != 0);
    }

    Ok(())
}
