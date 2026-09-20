use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int, json_string_as_string,
    json_value_as_int, json_value_as_string, open_asset_stream, string_to_int,
};

pub fn load_point_release_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.release_points.clear();

    let Some(bytes) = open_asset_stream(ctx, b"PointRelease.json", 0, 0)? else {
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
        let conditions = match entry {
            JsonNode::Object(fields) => match fields.get(b"serverSetting".as_slice()) {
                Some(JsonNode::Array(values)) => Some(values),
                _ => None,
            },
            _ => None,
        };

        if let Some(values) = conditions {
            let mut index = 0usize;

            while index < values.len() {
                let found = match entry {
                    JsonNode::Object(fields) => match fields.get(b"serverSetting".as_slice()) {
                        Some(JsonNode::Array(again)) => again.get(index),
                        _ => None,
                    },
                    _ => None,
                };
                let value = found.map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.release_points
                    .entry(point_id)
                    .or_default()
                    .conditions
                    .push(value);

                index += 1;
            }
        }

        let caps = match entry {
            JsonNode::Object(fields) => match fields.get(b"ReleasePoint".as_slice()) {
                Some(JsonNode::Array(values)) => Some(values),
                _ => None,
            },
            _ => None,
        };

        if let Some(values) = caps {
            let mut index = 0usize;

            while index < values.len() {
                let found = match entry {
                    JsonNode::Object(fields) => match fields.get(b"ReleasePoint".as_slice()) {
                        Some(JsonNode::Array(again)) => again.get(index),
                        _ => None,
                    },
                    _ => None,
                };
                let value = found.map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.release_points
                    .entry(point_id)
                    .or_default()
                    .caps
                    .push(value);

                index += 1;
            }
        }

        let popup = match entry {
            JsonNode::Object(fields) => match fields.get(b"Popup".as_slice()) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };

        ctx.release_points.entry(point_id).or_default().popup = popup;

        let headline = match entry {
            JsonNode::Object(fields) => {
                match fields.get(b"PointReleaseHeadlineLabel".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                }
            }
            _ => Vec::new(),
        };

        ctx.release_points
            .entry(point_id)
            .or_default()
            .headline_label = headline;

        let explanation = match entry {
            JsonNode::Object(fields) => {
                match fields.get(b"PointReleaseExplanationLabel".as_slice()) {
                    Some(JsonNode::String(text)) => json_string_as_string(text),
                    Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => {
                        json_container_as_string()
                    }
                    Some(found) => json_value_as_string(found),
                    None => Vec::new(),
                }
            }
            _ => Vec::new(),
        };

        ctx.release_points
            .entry(point_id)
            .or_default()
            .explanation_label = explanation;
    }

    Ok(())
}
