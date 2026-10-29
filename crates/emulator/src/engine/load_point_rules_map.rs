use crate::Fault;

use super::{
    AppContext, EventItemStore, JsonNode, PointRuleTable, json_parse_object_document,
    json_source_from_string, open_asset_stream, parse_point_stage_map, string_to_int,
};

pub fn load_point_rules_map(ctx: &mut AppContext, store: &mut EventItemStore) -> Result<(), Fault> {
    let Some(bytes) = open_asset_stream(ctx, b"StagePointRulesMap.json", 0, 0)? else {
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

    for (map_key, map_node) in maps {
        let map_id = string_to_int(map_key)?;
        let mut table = PointRuleTable::default();
        let listed = match map_node {
            JsonNode::Object(_) => Some(map_node),
            _ => None,
        };

        if parse_point_stage_map(&mut table, listed)? {
            store.rule_tables.insert(map_id, table);
        }
    }

    Ok(())
}
