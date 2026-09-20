use crate::{Fault, operation};

use super::{
    AppContext, get_base_upgrade, get_cat_combo_bonus, get_powerup, get_stage_record,
    get_treasure_value, min_i32, validate_map_type, xor_row46_get,
};

pub fn compute_stage_xp(ctx: &mut AppContext) -> Result<i32, Fault> {
    let row = (AppContext::MAP_STAGE_ROWS as i64
        + (ctx.i32_at(AppContext::STAGE_ROW)? as i64) * AppContext::MAP_STAGE_ROW_STRIDE as i64)
        as usize;
    let mut xp;

    if ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63 {
        xp = xor_row46_get(ctx.bytes_from(row)?, 1).ok_or(Fault::index_out_of_range(1, 0x2e))? as i32;
    } else {
        let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
        let legend_bonus = (map_type.wrapping_add(0x16) as u32) <= 0x16
            && 0x402001u32 >> map_type.wrapping_add(0x16) & 1 != 0;

        xp = if ctx.i32_at(AppContext::CHAPTER_MODE)? != 3 {
            ctx.i32_at(AppContext::STAGE_ROW)?
                .wrapping_mul(0x1770)
                .wrapping_add(0x4e20)
                / 10
        } else if legend_bonus {
            (xor_row46_get(ctx.bytes_from(row)?, 1).ok_or(Fault::index_out_of_range(1, 0x2e))? as i32)
                .wrapping_mul(9)
        } else {
            xor_row46_get(ctx.bytes_from(row)?, 1).ok_or(Fault::index_out_of_range(1, 0x2e))? as i32
        };

        let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;

        if chapter != 3 {
            let chapter = ctx.i32_at(AppContext::CHAPTER_MODE)?;
            let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
            let records = ctx.bytes_from(
                (AppContext::STAGE_RECORD_CHAPTERS as i64 + (chapter as i64) * 0xd0) as usize,
            )?;
            let cleared = operation::xor_row_decode(records, 0x33, stage as i64 as usize).ok_or(
                Fault::index_out_of_range(stage as i64, 0x33),
            )? as i32;
            let scaled = min_i32(cleared, 0xd).wrapping_mul(xp);

            xp = (scaled / -14).wrapping_add(xp);
        } else if legend_bonus {
            let map_type = validate_map_type(ctx.i32_at(AppContext::SAVED_MAP_TYPE)?);
            let map_index = ctx.i32_at(AppContext::MAP_INDEX)?;
            let stage = ctx.i32_at(AppContext::STAGE_ROW)?;
            let star = ctx.i32_at(AppContext::STAR_LEVEL)?;
            let clears = get_stage_record(ctx, map_type, map_index, stage, star, 0)?;
            let scaled = min_i32(clears, 4).wrapping_mul(xp);

            xp = (scaled / -5).wrapping_add(xp);
        }
    }

    let upgrade = get_base_upgrade(ctx, 9)?.wrapping_mul(5);
    let treasure = get_treasure_value(ctx, &ctx.treasure_store, 6)?
        .wrapping_add(upgrade)
        .wrapping_add(0x64);
    let treasure = if treasure > 0 { treasure } else { 0 };
    let mut value = operation::div_100(treasure.wrapping_mul(xp));
    let combo = get_cat_combo_bonus(ctx, &ctx.combo_store, 0xd, -1)?;

    value = operation::div_100(combo.wrapping_add(0x64).wrapping_mul(value));

    if get_powerup(ctx, 4)? {
        value = operation::div_10(value.wrapping_mul(0xf));
    }

    Ok(if value >= 0 { value } else { 1 })
}
