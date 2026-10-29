use crate::Fault;

use super::{AppContext, Entity, read_flag};

pub fn get_colossus_orb_atk_pct(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(0);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::COLOSSUS_SLAYER,
        ))? != 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::COLOSSUS_ORB_ATK_PCT,
        ))? == 0
    {
        return Ok(0xa0);
    }

    ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::COLOSSUS_ORB_ATK_PCT,
    ))
}
