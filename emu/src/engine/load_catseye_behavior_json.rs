use crate::{Fault, ops};

use super::{
    AppContext, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
    string_to_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CatseyeStep {
    pub catseyes: Vec<i32>,
    pub max: i32,
    pub value: i32,
}

pub fn load_catseye_behavior_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.catseye_behavior.clear();

    let Some(bytes) = open_asset_stream(ctx, b"unititempowerup.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(data)) = root.get(b"data".as_slice()) else {
        return Err(Fault::null_pointer());
    };

    for (unit_key, entry) in data {
        let unit_key = string_to_int(unit_key)?;
        let JsonNode::Object(fields) = entry else {
            return Err(Fault::null_pointer());
        };
        let Some(JsonNode::Array(values)) = fields.get(b"a".as_slice()) else {
            continue;
        };
        let unit_id = ops::div_10(unit_key);
        let form = unit_key
            .wrapping_sub(unit_id.wrapping_mul(10))
            .wrapping_sub(1);

        ctx.catseye_behavior
            .entry(unit_id)
            .or_default()
            .entry(form)
            .or_default()
            .resize(values.len(), CatseyeStep::default());

        if values.is_empty() {
            continue;
        }

        let mut index = 0usize;

        while index < values.len() {
            let Some(JsonNode::Object(step_fields)) = values.get(index) else {
                return Err(Fault::null_pointer());
            };
            let Some(JsonNode::Array(listed)) = step_fields.get(b"catseye".as_slice()) else {
                return Err(Fault::null_pointer());
            };

            for found in listed {
                let value = match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                }? as i32;

                if let Some(step) = ctx
                    .catseye_behavior
                    .entry(unit_id)
                    .or_default()
                    .entry(form)
                    .or_default()
                    .get_mut(index)
                {
                    step.catseyes.push(value);
                }
            }

            let max = step_fields
                .get(b"max".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            if let Some(step) = ctx
                .catseye_behavior
                .entry(unit_id)
                .or_default()
                .entry(form)
                .or_default()
                .get_mut(index)
            {
                step.max = max;
            }

            let value = step_fields
                .get(b"value".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })? as i32;

            if let Some(step) = ctx
                .catseye_behavior
                .entry(unit_id)
                .or_default()
                .entry(form)
                .or_default()
                .get_mut(index)
            {
                step.value = value;
            }

            index += 1;
        }
    }

    Ok(())
}
