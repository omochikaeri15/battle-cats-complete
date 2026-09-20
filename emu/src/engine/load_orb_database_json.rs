use crate::Fault;

use super::{
    AppContext, JsonNode, OrbDef, json_parse_object_document, json_source_from_string,
    open_asset_stream, orb_parse_record,
};

const SITE: &str = "load_orb_database_json";

pub fn load_orb_database_json(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.orb_store.orbs.clear();

    let Some(bytes) = open_asset_stream(ctx, b"equipmentlist.json", 0, 0)? else {
        return Ok(());
    };

    let source = json_source_from_string(&bytes);
    let document = json_parse_object_document(Some(source))?;
    let Some(JsonNode::Object(root)) = document.as_ref() else {
        return Err(Fault::NullPointer { site: SITE });
    };

    let mut index = 0usize;

    loop {
        let Some(JsonNode::Array(entries)) = root.get(b"ID".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        if index >= entries.len() {
            return Ok(());
        }

        ctx.orb_store.orbs.push(OrbDef::default());

        let record = ctx
            .orb_store
            .orbs
            .last_mut()
            .ok_or(Fault::NullPointer { site: SITE })?;

        record.id = index as i32;

        let Some(JsonNode::Array(entries)) = root.get(b"ID".as_slice()) else {
            return Err(Fault::NullPointer { site: SITE });
        };

        let element = entries.get(index).ok_or(Fault::IndexOutOfRange {
            site: SITE,
            index: index as i64,
            limit: entries.len() as i64,
        })?;

        orb_parse_record(
            record,
            match element {
                JsonNode::Object(_) => Some(element),
                _ => None,
            },
        )?;

        index += 1;
    }
}
