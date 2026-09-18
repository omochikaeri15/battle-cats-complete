use crate::Fault;

use super::{stat_has_trait, AppContext};

pub fn stat_trait_mask(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32) -> Result<i32, Fault> {
    let red = stat_has_trait(ctx, faction, unit_id, form, 0x0)? as i32;
    let floating = stat_has_trait(ctx, faction, unit_id, form, 0x1)? as i32;
    let dark = stat_has_trait(ctx, faction, unit_id, form, 0x2)? as i32;
    let metal = stat_has_trait(ctx, faction, unit_id, form, 0x3)? as i32;
    let angel = stat_has_trait(ctx, faction, unit_id, form, 0x4)? as i32;
    let alien = stat_has_trait(ctx, faction, unit_id, form, 0x5)? as i32;
    let zombie = stat_has_trait(ctx, faction, unit_id, form, 0x6)? as i32;
    let relic = stat_has_trait(ctx, faction, unit_id, form, 0x7)? as i32;
    let aku = stat_has_trait(ctx, faction, unit_id, form, 0x8)? as i32;
    let traitless = stat_has_trait(ctx, faction, unit_id, form, 0x9)? as i32;
    let colossus = stat_has_trait(ctx, faction, unit_id, form, 0xa)? as i32;
    let behemoth = stat_has_trait(ctx, faction, unit_id, form, 0xb)? as i32;
    let sage = stat_has_trait(ctx, faction, unit_id, form, 0xc)? as i32;
    let witch = stat_has_trait(ctx, faction, unit_id, form, 0xd)? as i32;
    let eva = stat_has_trait(ctx, faction, unit_id, form, 0xe)? as i32;
    let kaijin = stat_has_trait(ctx, faction, unit_id, form, 0xf)? as i32;

    Ok(red.wrapping_add(floating << 1).wrapping_add(dark << 2).wrapping_add(metal << 3)
        | (angel << 0x4)
        | (alien << 0x5)
        | (zombie << 0x6)
        | (relic << 0x7)
        | (aku << 0x8)
        | (traitless << 0x9)
        | (colossus << 0xa)
        | (behemoth << 0xb)
        | (sage << 0xc)
        | (witch << 0xd)
        | (eva << 0xe)
        | (kaijin << 0xf))
}
