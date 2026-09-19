use crate::{Fault, operation};

use super::{
    AppContext, CatStats, EnemyStats, get_cat_combo_bonus, get_global_map_id,
    get_special_rule_params, get_talent_value, read_flag,
};

pub fn stat_speed(
    ctx: &mut AppContext,
    faction: i32,
    unit_id: i32,
    form: i32,
) -> Result<i32, Fault> {
    let mut speed = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        ctx.i32_at(AppContext::enemy_stat(unit_id, EnemyStats::SPEED))?
    } else {
        let base = ctx.i32_at(AppContext::cat_stat(unit_id, form, CatStats::SPEED))?;
        let boosted = get_talent_value(ctx, faction, unit_id, form, 0x1b, 0)?
            .wrapping_mul(2)
            .wrapping_add(base);

        operation::div_100(
            get_cat_combo_bonus(ctx, &ctx.combo_store, 2, unit_id)?
                .wrapping_add(0x64)
                .wrapping_mul(boosted),
        )
    };

    let map_id = get_global_map_id(ctx, 0)?;
    let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 0xa)? else {
        return Ok(speed);
    };

    if speed == 0 {
        return Ok(speed);
    }

    let mode_cell = faction.wrapping_add(faction);
    let mode = *params
        .get(mode_cell as i64 as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "stat_speed",
            index: mode_cell as i64,
            limit: params.len() as i64,
        })?;

    if mode == 2 {
        let percent =
            *params
                .get((mode_cell | 1) as i64 as usize)
                .ok_or(Fault::IndexOutOfRange {
                    site: "stat_speed",
                    index: (mode_cell | 1) as i64,
                    limit: params.len() as i64,
                })?;

        speed = operation::div_100(speed.wrapping_mul(percent));
    } else if mode == 1 {
        let fixed = *params
            .get((mode_cell | 1) as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: "stat_speed",
                index: (mode_cell | 1) as i64,
                limit: params.len() as i64,
            })?;

        speed = fixed.wrapping_add(fixed);
    }

    Ok(speed)
}
