use crate::Fault;

use super::{read_flag, AppContext, CatStats, EnemyStats};

pub fn stat_has_omni_strike(ctx: &AppContext, faction: i32, unit_id: i32, form: i32) -> Result<bool, Fault> {
    let ld1_anchor = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::LD1_ANCHOR)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::LD1_ANCHOR)
    };

    if ctx.i32_at(ld1_anchor)? != 0 {
        let ld1_span = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
            AppContext::cat_stat(unit_id, form, CatStats::LD1_SPAN)
        } else {
            AppContext::enemy_stat(unit_id, EnemyStats::LD1_SPAN)
        };

        if ctx.i32_at(ld1_span)? < 0 {
            return Ok(true);
        }
    }

    let ld2_anchor = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::LD2_ANCHOR)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::LD2_ANCHOR)
    };

    if ctx.i32_at(ld2_anchor)? & 0x3fffffff != 0 {
        let ld2_span = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
            AppContext::cat_stat(unit_id, form, CatStats::LD2_SPAN)
        } else {
            AppContext::enemy_stat(unit_id, EnemyStats::LD2_SPAN)
        };

        if ctx.i32_at(ld2_span)? & 0x20000000 != 0 {
            return Ok(true);
        }
    }

    let ld3_anchor = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        AppContext::cat_stat(unit_id, form, CatStats::LD3_ANCHOR)
    } else {
        AppContext::enemy_stat(unit_id, EnemyStats::LD3_ANCHOR)
    };

    if ctx.i32_at(ld3_anchor)? & 0x3fffffff != 0 {
        let ld3_span = if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
            AppContext::cat_stat(unit_id, form, CatStats::LD3_SPAN)
        } else {
            AppContext::enemy_stat(unit_id, EnemyStats::LD3_SPAN)
        };

        if ctx.i32_at(ld3_span)? & 0x20000000 != 0 {
            return Ok(true);
        }
    }

    Ok(false)
}
