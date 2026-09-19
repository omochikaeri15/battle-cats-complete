use crate::Fault;

use super::{
    AppContext, Entity, get_entity_base_idx, has_castle_enemy, is_boss_guarding_base,
    set_base_guard_notice,
};

pub fn attack_dmg_dispatch(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    damage: i32,
) -> Result<(), Fault> {
    if faction == 1 {
        let mut base_idx = 0i32;

        if has_castle_enemy(ctx)? {
            base_idx = get_entity_base_idx(ctx)?;
        }

        if base_idx == slot && is_boss_guarding_base(ctx)? {
            return set_base_guard_notice(ctx, AppContext::BASE_GUARD_NOTICE, 1);
        }
    }

    let frame_damage = AppContext::entity_field(faction, slot, Entity::FRAME_DAMAGE);
    ctx.set_i32_at(frame_damage, ctx.i32_at(frame_damage)?.wrapping_add(damage))?;

    let total_damage = AppContext::entity_field(faction, slot, Entity::TOTAL_DAMAGE_TAKEN);
    ctx.set_i32_at(total_damage, ctx.i32_at(total_damage)?.wrapping_add(damage))?;

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BARRIER_HP))? > 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::BARRIER_HP))? < damage
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::BARRIER_STATE,
        ))? == 0
    {
        ctx.set_i32_at(
            AppContext::entity_field(faction, slot, Entity::BARRIER_STATE),
            1,
        )?;
    }

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::TOOK_DAMAGE),
        1,
    )
}
