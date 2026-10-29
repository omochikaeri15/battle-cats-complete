use std::collections::BTreeMap;

use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
    string_to_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct DailyLoginGrade {
    pub group_id: i32,
    pub grade: i32,
    pub conditions: BTreeMap<i32, Vec<i32>>,
}

pub fn load_daily_login_grade_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.daily_login_grades.clear();

    let Some(bytes) = open_asset_stream(ctx, b"DailyLoginEventGrade.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Ok(());
    };
    let Some(JsonNode::Object(stamps)) = root.get(b"DailyLoginStampID".as_slice()) else {
        return Ok(());
    };

    for (stamp_key, entry) in stamps {
        let stamp_id = string_to_int(stamp_key)?;
        let JsonNode::Object(fields) = entry else {
            return Err(Fault::null_pointer());
        };
        let group_id = fields
            .get(b"DailyLoginStampGroupID".as_slice())
            .map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

        ctx.daily_login_grades
            .entry(stamp_id)
            .or_default()
            .group_id = group_id;

        let grade = fields
            .get(b"DailyLoginStampGrade".as_slice())
            .map_or(Ok(0), |found| match found {
                JsonNode::String(text) => json_string_as_int(text),
                JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                _ => Ok(json_value_as_int(found)),
            })? as i32;

        ctx.daily_login_grades.entry(stamp_id).or_default().grade = grade;

        let Some(JsonNode::Object(kinds)) = fields.get(b"ConditionType".as_slice()) else {
            return Err(Fault::null_pointer());
        };

        for (kind_key, kind_entry) in kinds {
            let kind = string_to_int(kind_key)?;
            let JsonNode::Object(kind_fields) = kind_entry else {
                return Err(Fault::null_pointer());
            };
            let Some(JsonNode::Array(values)) = kind_fields.get(b"Parameters".as_slice()) else {
                return Err(Fault::null_pointer());
            };
            let mut index = 0usize;

            while index < values.len() {
                let value = values.get(index).map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

                ctx.daily_login_grades
                    .entry(stamp_id)
                    .or_default()
                    .conditions
                    .entry(kind)
                    .or_default()
                    .push(value);

                index += 1;
            }
        }
    }

    Ok(())
}
