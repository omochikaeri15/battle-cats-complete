use crate::{operation, Fault};

use super::{get_button_unit_form, get_entity_button, get_setting, has_orb, AppContext, Entity};

pub fn get_status_bits(ctx: &AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    let mut bits = 0i32;

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STRENGTHEN_THRESHOLD))? != 0 {
        let hp = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::HP))?;
        let max_hp = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::MAX_HP))?;
        let scaled = max_hp.wrapping_mul(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::STRENGTHEN_THRESHOLD))?);

        bits = (hp <= operation::div_100(scaled as i64) as i32) as i32;
    }

    if faction != 0 {
        return Ok(bits);
    }

    let occupant = ctx.i32_at(AppContext::entity_field(0, slot, Entity::OCCUPANT))?;

    if get_button_unit_form(ctx, 0, get_entity_button(ctx, 0, slot)?)? < 2 {
        return Ok(bits);
    }

    let kill_count = ctx.i32_at(AppContext::entity_field(0, slot, Entity::KILL_COUNT))?;

    if kill_count < get_setting(&ctx.settings, b"battle_powerup_kill_enemy", 0xf)? {
        return Ok(bits);
    }

    if has_orb(ctx, &ctx.orb_store, occupant.wrapping_add(-2), 0x12)? {
        bits |= 2;
    }

    Ok(bits)
}
