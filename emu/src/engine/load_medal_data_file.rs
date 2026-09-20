use crate::Fault;

use super::{
    AppContext, AssetStream, JsonNode, Medal, json_parse_object_document, json_source_from_string,
    open_asset_stream, parse_medal_entry, read_medal_name_row,
};

pub fn load_medal_data_file(ctx: &mut AppContext) -> Result<(), Fault> {
    if let Some(bytes) = open_asset_stream(ctx, b"medallist.json", 0, 0)? {
        let source = json_source_from_string(&bytes);
        let document = json_parse_object_document(Some(source))?;

        ctx.medals.clear();
        ctx.medal_order.clear();

        if let Some(JsonNode::Object(root)) = document.as_ref() {
            let mut index = 0usize;

            loop {
                let listed = match root.get(b"iconID".as_slice()) {
                    Some(JsonNode::Array(values)) => Some(values),
                    _ => None,
                };
                let Some(values) = listed else {
                    break;
                };

                if values.len() <= index {
                    break;
                }

                ctx.medals.push(Medal::default());

                let entry = match values.get(index) {
                    Some(JsonNode::Object(_)) => values.get(index),
                    _ => None,
                };

                if let Some(medal) = ctx.medals.last_mut() {
                    parse_medal_entry(medal, entry)?;
                }

                let line = ctx.medals.last().map_or(0, |medal| medal.line);

                ctx.medal_order
                    .push(u64::from(line as u32) << 0x20 | index as u64);

                index += 1;
            }

            ctx.medal_order.sort();
        }
    }

    let Some(bytes) = open_asset_stream(ctx, b"medalname.tsv", 0, 0)? else {
        return Ok(());
    };
    let stm = &mut AssetStream::new(&bytes, b'\n');
    let mut index = 0usize;

    while index < ctx.medals.len() {
        if let Some(medal) = ctx.medals.get_mut(index) {
            read_medal_name_row(medal, stm);
        }

        index += 1;
    }

    Ok(())
}
