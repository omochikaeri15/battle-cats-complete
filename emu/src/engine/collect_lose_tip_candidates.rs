use crate::Fault;

use super::{get_global_map_id, lose_tip_allowed, lose_tip_targets_stage, AppContext};

pub fn collect_lose_tip_candidates(ctx: &mut AppContext) -> Result<Vec<i32>, Fault> {
    let mut candidates = Vec::new();
    let mut targeted = false;
    let mut row = 0i64;

    while row < ctx.lose_row_settings.len() as i32 as i64 {
        if !targeted && lose_tip_allowed(ctx, ctx.lose_row_settings[row as usize])? {
            candidates.push(row as i32);
        }

        let id = ctx.lose_row_settings[row as usize];
        let map = get_global_map_id(ctx, 0)?;
        let stage = ctx.i32_at(AppContext::STAGE_ROW)?;

        if lose_tip_targets_stage(ctx, id, map, stage)? {
            if !targeted {
                candidates.clear();
                targeted = true;
            }

            candidates.push(row as i32);
        }

        row += 1;
    }

    Ok(candidates)
}
