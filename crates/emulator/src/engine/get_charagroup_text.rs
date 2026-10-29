use super::{AppContext, query_localizable};

pub fn get_charagroup_text(ctx: &AppContext, group_id: i32) -> Vec<u8> {
    let Some(group) = ctx
        .chara_groups
        .range(group_id..)
        .next()
        .filter(|(key, _)| **key <= group_id)
        .map(|(_, group)| group)
    else {
        return Vec::new();
    };

    query_localizable(ctx, &group.name_key)
}
