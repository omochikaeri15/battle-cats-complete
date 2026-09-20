use crate::Fault;

use super::{
    AppContext, JsonNode, json_container_as_int, json_parse_object_document,
    json_source_from_string, json_string_as_int, json_value_as_int, open_asset_stream,
    string_to_int,
};

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventGatyaItem {
    pub kind: i32,
    pub id: i32,
    pub value: i32,
    pub weight: i32,
}

#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct EventGatyaGroup {
    pub min_count: i32,
    pub items: Vec<EventGatyaItem>,
}

pub fn load_event_item_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.event_gatya_items.clear();

    let Some(bytes) = open_asset_stream(ctx, b"EventGatyaMedamaItem.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::null_pointer());
    };
    let Some(JsonNode::Object(groups)) = root.get(b"GatyaID".as_slice()) else {
        return Ok(());
    };

    for (gatya_key, entry) in groups {
        let gatya_id = string_to_int(gatya_key)?;
        let min_count = match entry {
            JsonNode::Object(fields) => fields
                .get(b"minCount".as_slice())
                .map_or(Ok(0), |found| match found {
                    JsonNode::String(text) => json_string_as_int(text),
                    JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                    _ => Ok(json_value_as_int(found)),
                })?,
            _ => 0,
        } as i32;

        ctx.event_gatya_items
            .entry(gatya_id)
            .or_default()
            .min_count = min_count;

        let listed = match entry {
            JsonNode::Object(fields) => match fields.get(b"data".as_slice()) {
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
            ctx.event_gatya_items
                .entry(gatya_id)
                .or_default()
                .items
                .push(EventGatyaItem::default());

            let kind = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"type".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .event_gatya_items
                .entry(gatya_id)
                .or_default()
                .items
                .last_mut()
            {
                last.kind = kind;
            }

            let id = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"id".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .event_gatya_items
                .entry(gatya_id)
                .or_default()
                .items
                .last_mut()
            {
                last.id = id;
            }

            let value = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"value".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .event_gatya_items
                .entry(gatya_id)
                .or_default()
                .items
                .last_mut()
            {
                last.value = value;
            }

            let weight = match values.get(index) {
                Some(JsonNode::Object(fields)) => fields
                    .get(b"weight".as_slice())
                    .map_or(Ok(0), |found| match found {
                        JsonNode::String(text) => json_string_as_int(text),
                        JsonNode::Array(_) | JsonNode::Object(_) => Ok(json_container_as_int()),
                        _ => Ok(json_value_as_int(found)),
                    })?,
                _ => 0,
            } as i32;

            if let Some(last) = ctx
                .event_gatya_items
                .entry(gatya_id)
                .or_default()
                .items
                .last_mut()
            {
                last.weight = weight;
            }

            index += 1;
        }
    }

    Ok(())
}
