use crate::{Fault, operation};

use super::{AppContext, get_base_upgrade, get_cat_combo_bonus, get_treasure_value};

pub fn get_cannon_charge_frames(ctx: &mut AppContext) -> Result<i32, Fault> {
    let upgrade = get_base_upgrade(ctx, 1)?;
    let treasure = get_treasure_value(ctx, &ctx.treasure_store, 0x11)?;
    let chapter = if ctx.i32_at(AppContext::CHAPTER_MODE)? > 2 {
        0x546
    } else {
        ctx.i32_at(AppContext::CHAPTER_MODE)?.wrapping_mul(0x1c2)
    };
    let reduced = upgrade.wrapping_mul(0x32).wrapping_sub(treasure);
    let reduced = if reduced > 0 { reduced } else { 0 };
    let frames = get_base_upgrade(ctx, 3)?
        .wrapping_mul(-0x32)
        .wrapping_add(chapter)
        .wrapping_add(reduced);
    let frames = frames
        .wrapping_sub(get_treasure_value(ctx, &ctx.treasure_store, 1)?)
        .wrapping_add(0x5dc);
    let bonus = get_cat_combo_bonus(ctx, &ctx.combo_store, 7, -1)?;
    let frames = operation::div_100(100i32.wrapping_sub(bonus).wrapping_mul(frames) as i64) as i32;

    Ok(if frames >= 0x3b7 { frames } else { 0x3b6 })
}
