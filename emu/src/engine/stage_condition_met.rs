use crate::Fault;

use super::{
    AppContext, aku_list_key, aku_realm_final_redirect, get_aku_stage_list, get_cleared_count,
    get_stage_record, unlock_condition_met,
};

pub fn stage_condition_met(
    ctx: &mut AppContext,
    map_type: i32,
    map_idx: i32,
    stage: i32,
) -> Result<bool, Fault> {
    match map_type {
        -21 => Ok(get_cleared_count(ctx, 0xad0)? <= stage),
        -19 => {
            let listed = get_aku_stage_list(ctx, aku_list_key());

            if listed.contains(&stage) {
                return Ok(true);
            }

            if aku_realm_final_redirect(ctx, 1)? {
                return Ok(true);
            }

            Ok(get_stage_record(ctx, -19, map_idx, stage, 0, 0)? > 0)
        }
        -11 => {
            let missing = Fault::index_out_of_range(stage as i64, ctx.legend_stage_conditions.len() as i64);
            let mut met = true;

            if ctx
                .legend_stage_conditions
                .get(stage as i64 as usize)
                .ok_or(missing.clone())?[0]
                != 0
            {
                let first = ctx
                    .legend_stage_conditions
                    .get(stage as i64 as usize)
                    .ok_or(missing.clone())?[0];

                met = unlock_condition_met(ctx, first)?;
            }

            if ctx
                .legend_stage_conditions
                .get(stage as i64 as usize)
                .ok_or(missing.clone())?[1]
                != 0
            {
                let second = ctx
                    .legend_stage_conditions
                    .get(stage as i64 as usize)
                    .ok_or(missing)?[1];

                met &= unlock_condition_met(ctx, second)?;
            }

            Ok(met)
        }
        _ => {
            if !ctx.stage_conditions.contains_key(&map_type) {
                return Ok(true);
            }

            if !ctx
                .stage_conditions
                .entry(map_type)
                .or_default()
                .contains_key(&map_idx)
            {
                return Ok(true);
            }

            if !ctx
                .stage_conditions
                .entry(map_type)
                .or_default()
                .entry(map_idx)
                .or_default()
                .contains_key(&stage)
            {
                return Ok(true);
            }

            let id = *ctx
                .stage_conditions
                .entry(map_type)
                .or_default()
                .entry(map_idx)
                .or_default()
                .entry(stage)
                .or_insert(0);

            unlock_condition_met(ctx, id)
        }
    }
}
