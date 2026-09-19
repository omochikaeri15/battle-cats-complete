use crate::Fault;

use super::{AppContext, find_built_deck_4star, get_map_type};

pub fn has_built_deck(ctx: &mut AppContext, stage_key: i32) -> Result<bool, Fault> {
    let case = get_map_type(ctx, 0)?.wrapping_add(0x19) as u32;

    if case < 0x14 && 0x87c01u32 >> case & 1 != 0 {
        return Ok(false);
    }

    for stage_keys in ctx.built_deck_stages.values() {
        if stage_keys.contains(&stage_key) {
            return Ok(true);
        }
    }

    if stage_key < 1_000_000_000 {
        return Ok(false);
    }

    Ok(find_built_deck_4star(ctx, stage_key) >= 0)
}
