use crate::Fault;

use super::{AppContext, Entity};

pub fn is_touchable(
    ctx: &AppContext,
    faction: i32,
    slot: i32,
    attacker: i32,
) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0 {
        return Ok(true);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 1 {
        return Ok(true);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 2 {
        return Ok(true);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0xb {
        return Ok(true);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0xd {
        return Ok(true);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0x10 {
        return Ok(true);
    }

    if attacker == -1 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::entity_field(
        1i32.wrapping_sub(faction),
        attacker,
        Entity::SOULSTRIKE,
    ))? == 0
    {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0xe {
        return Ok(true);
    }

    Ok(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STATE))? == 0xf)
}
