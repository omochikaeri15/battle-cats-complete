use crate::Fault;

use super::{
    AppContext, get_global_map_id, get_stage_index, get_stage_variant, is_stage_cleared_session,
    stage_entry_enemy_id,
};

pub fn record_stage_played(ctx: &mut AppContext) -> Result<(), Fault> {
    let variant = get_stage_variant(ctx)?;
    let map = get_global_map_id(ctx, 0)?;
    let stage = get_stage_index(ctx)?;

    if !is_stage_cleared_session(ctx, map, stage, variant)? {
        let key = get_global_map_id(ctx, 0)?.wrapping_mul(0x3e8);
        let key = get_stage_index(ctx)?
            .wrapping_mul(5)
            .wrapping_mul(2)
            .wrapping_add(key.wrapping_add(variant));

        ctx.cleared_session_keys.push(key);
    }

    let mut row = 0usize;

    while row < ctx.stage_enemies.len() {
        let entry = ctx.stage_enemies.get(row).ok_or(Fault::index_out_of_range(row as i64, 0))?;
        let seen = ((stage_entry_enemy_id(entry) as i64) * 4 + AppContext::ENEMY_GUIDE_SEEN as i64)
            as usize;

        if ctx.i32_at(seen)? == 0 {
            let entry = ctx.stage_enemies.get(row).ok_or(Fault::index_out_of_range(row as i64, 0))?;
            let seen = ((stage_entry_enemy_id(entry) as i64) * 4
                + AppContext::ENEMY_GUIDE_SEEN as i64) as usize;

            ctx.set_i32_at(seen, 1)?;
        }

        row += 1;
    }

    Ok(())
}
