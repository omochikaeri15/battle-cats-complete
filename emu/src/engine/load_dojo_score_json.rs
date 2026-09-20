use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_container_as_string,
    json_parse_object_document, json_source_from_string, json_string_as_int, json_string_as_string,
    json_value_as_int, json_value_as_string, open_asset_stream, string_to_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DojoScoreBonus {
    pub name_label: Vec<u8>,
    pub explanation_label: Vec<u8>,
    pub bonuses: BTreeMap<i32, Vec<i32>>,
}

pub fn load_dojo_score_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.dojo_score_bonuses.clear();

    let Some(bytes) = open_asset_stream(ctx, b"ScoreBonusMap.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Ok(());
    };
    let Some(JsonNode::Object(maps)) = root.get(b"MapID".as_slice()) else {
        return Ok(());
    };

    for (map_key, entry) in maps {
        let map_id = string_to_int(map_key)?;
        let name_label = match entry {
            JsonNode::Object(fields) => match fields.get(b"BonusNameLabel".as_slice()) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };

        ctx.dojo_score_bonuses
            .entry(map_id)
            .or_default()
            .name_label = name_label;

        let explanation_label = match entry {
            JsonNode::Object(fields) => match fields.get(b"BonusExplanationLabel".as_slice()) {
                Some(JsonNode::String(text)) => json_string_as_string(text),
                Some(JsonNode::Array(_)) | Some(JsonNode::Object(_)) => json_container_as_string(),
                Some(found) => json_value_as_string(found),
                None => Vec::new(),
            },
            _ => Vec::new(),
        };

        ctx.dojo_score_bonuses
            .entry(map_id)
            .or_default()
            .explanation_label = explanation_label;

        let listed = match entry {
            JsonNode::Object(fields) => match fields.get(b"BonusType".as_slice()) {
                Some(JsonNode::Object(kinds)) => Some(kinds),
                _ => None,
            },
            _ => None,
        };
        let Some(kinds) = listed else {
            continue;
        };

        for (kind_key, kind_entry) in kinds {
            let kind = string_to_int(kind_key)?;
            let parameters = match kind_entry {
                JsonNode::Object(fields) => match fields.get(b"Parameters".as_slice()) {
                    Some(JsonNode::Array(values)) => Some(values),
                    _ => None,
                },
                _ => None,
            };
            let Some(values) = parameters else {
                continue;
            };
            let mut index = 0usize;

            while index < values.len() {
                let value = values.get(index).map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.dojo_score_bonuses
                    .entry(map_id)
                    .or_default()
                    .bonuses
                    .entry(kind)
                    .or_default()
                    .push(value);

                index += 1;
            }
        }
    }

    Ok(())
}
