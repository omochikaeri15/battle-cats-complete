use super::AppContext;

pub fn find_built_deck_4crown(ctx: &AppContext, stage_key: i32) -> i32 {
    for (deck_id, stage_keys) in ctx.built_deck_stages.iter() {
        if stage_keys.contains(&stage_key.wrapping_sub(1_000_000_000)) {
            return *deck_id as i32;
        }
    }

    -1
}
