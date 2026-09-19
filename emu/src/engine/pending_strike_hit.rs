use crate::Fault;

use super::{
    AppContext, Entity, attack_dmg_dispatch, call_rng, get_base_max_hp_div_20, get_dodge_chance,
    get_dodge_duration, get_dodge_timer, is_metal, is_touchable, is_touchable_thunk,
    set_dodge_timer, set_dodge_vfx_frame,
};

pub fn pending_strike_hit(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    striker: i32,
) -> Result<(), Fault> {
    let entity = AppContext::entity_field(faction, slot, 0);
    let mut strike = 0usize;

    loop {
        'strike: {
            if ctx.i32_at(AppContext::PENDING_STRIKE_TARGET.wrapping_add(strike * 4))? != slot {
                break 'strike;
            }

            if ctx.u8_at(AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike))? == 0 {
                break 'strike;
            }

            let touchable = is_touchable(ctx, faction, slot, -1)?;
            let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
            let trigger_x =
                ctx.i32_at(AppContext::PENDING_STRIKE_TRIGGER_X.wrapping_add(strike * 4))?;

            if x < trigger_x {
                break 'strike;
            }

            ctx.set_block_at(
                AppContext::PENDING_STRIKE_ACTIVE.wrapping_add(strike),
                [0u8; 1],
            )?;

            if touchable && is_touchable_thunk(ctx, faction, slot, -1)? {
                if get_dodge_timer(ctx, faction, slot)? > 0 {
                    break 'strike;
                }

                let roll = call_rng(ctx, 0x64);

                if roll < get_dodge_chance(ctx, faction, slot)? {
                    let duration = get_dodge_duration(ctx, faction, slot)?;

                    set_dodge_timer(ctx, faction, slot, duration)?;
                    set_dodge_vfx_frame(ctx, faction, slot, 1)?;

                    break 'strike;
                }

                if is_metal(ctx, faction, slot)? {
                    attack_dmg_dispatch(ctx, faction, slot, 1)?;
                } else {
                    let damage = get_base_max_hp_div_20(ctx, striker)?;

                    attack_dmg_dispatch(ctx, faction, slot, damage)?;
                    get_base_max_hp_div_20(ctx, striker)?;
                }

                ctx.set_i32_at(entity.wrapping_add(Entity::CANNON_BLAST_HIT), 1)?;
            }

            let sparks = AppContext::PENDING_STRIKE_SPARKS
                .wrapping_add(strike * AppContext::PENDING_STRIKE_SPARKS_STRIDE);

            ctx.set_i32_at(sparks, 0xc)?;

            let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
            let scatter = call_rng(ctx, 0x64);

            ctx.set_i32_at(
                sparks + 0x4,
                scatter
                    .wrapping_mul(2)
                    .wrapping_mul(5)
                    .wrapping_neg()
                    .wrapping_add(x)
                    .wrapping_add(-0xbb),
            )?;

            let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
            let scatter = call_rng(ctx, 0x3c);

            ctx.set_i32_at(
                sparks + 0x8,
                y.wrapping_add(scatter.wrapping_mul(5).wrapping_mul(2))
                    .wrapping_add(-0x5c3),
            )?;
            ctx.set_i32_at(sparks + 0xc, 0xc)?;

            let x = ctx.i32_at(entity.wrapping_add(Entity::POS_X))?;
            let scatter = call_rng(ctx, 0x64);

            ctx.set_i32_at(
                sparks + 0x10,
                scatter
                    .wrapping_mul(2)
                    .wrapping_mul(5)
                    .wrapping_neg()
                    .wrapping_add(x)
                    .wrapping_add(-0xbb),
            )?;

            let y = ctx.i32_at(entity.wrapping_add(Entity::POS_Y))?;
            let scatter = call_rng(ctx, 0x3c);

            ctx.set_i32_at(
                sparks + 0x14,
                y.wrapping_add(scatter.wrapping_mul(5).wrapping_mul(2))
                    .wrapping_add(-0x5c3),
            )?;
        }

        strike += 1;

        if strike == 50 {
            return Ok(());
        }
    }
}
