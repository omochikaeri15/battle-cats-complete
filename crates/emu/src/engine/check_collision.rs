use crate::Fault;

use super::{
    AppContext, Entity, get_entity_base_idx, get_entity_state, get_hp, get_ld_anchor, get_ld_span,
    has_castle_enemy, is_attack_long_range, is_touchable, slot_occupied,
};

pub fn check_collision(
    ctx: &AppContext,
    faction: i32,
    slot: i32,
    target: i32,
    attack: i32,
) -> Result<bool, Fault> {
    let other = (faction == 0) as i32;

    if target | faction == 0 && has_castle_enemy(ctx)? && get_hp(ctx, 1, 0)? > 0 {
        return Ok(false);
    }

    let at_base = if target == 0 {
        true
    } else if faction != 0 {
        false
    } else {
        get_entity_base_idx(ctx)? == target
    };

    if at_base
        && get_entity_state(ctx, faction, slot)? != 2
        && is_attack_long_range(ctx, faction, slot, 0)?
    {
        let edge = if faction == 0 {
            ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?
                .wrapping_sub(get_ld_anchor(ctx, 0, slot, 0)?)
        } else {
            let own_x = ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?;

            get_ld_anchor(ctx, faction, slot, 0)?.wrapping_add(own_x)
        };

        if !is_touchable(ctx, other, target, slot)? {
            return Ok(false);
        }

        let target_x = ctx.i32_at(AppContext::entity_field(other, target, Entity::POS_X))?;

        if faction != 0 {
            return Ok(target_x <= edge);
        }

        return Ok(target_x >= edge);
    }

    if get_entity_state(ctx, faction, slot)? == 2
        && is_attack_long_range(ctx, faction, slot, attack)?
    {
        if slot_occupied(ctx, other, target)? == 0 {
            return Ok(false);
        }

        let first;
        let second;

        if faction != 0 {
            let span = get_ld_span(ctx, faction, slot, attack)?;
            let own_x = ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?;
            let anchored = get_ld_anchor(ctx, faction, slot, attack)?.wrapping_add(own_x);

            if span <= 0 {
                first = anchored.wrapping_add(get_ld_span(ctx, faction, slot, attack)?);

                let own_x = ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?;

                second = get_ld_anchor(ctx, faction, slot, attack)?.wrapping_add(own_x);
            } else {
                let own_x = ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?;
                let anchor = get_ld_anchor(ctx, faction, slot, attack)?;

                second = get_ld_span(ctx, faction, slot, attack)?
                    .wrapping_add(anchor)
                    .wrapping_add(own_x);
                first = anchored;
            }
        } else {
            let span = get_ld_span(ctx, 0, slot, attack)?;
            let own_x = ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?;
            let anchor = get_ld_anchor(ctx, 0, slot, attack)?;

            if span <= 0 {
                let reach = anchor.wrapping_add(get_ld_span(ctx, 0, slot, attack)?);

                first = own_x.wrapping_sub(reach);

                let own_x = ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?;

                second = own_x.wrapping_sub(get_ld_anchor(ctx, 0, slot, attack)?);
            } else {
                first = own_x.wrapping_sub(anchor);

                let own_x = ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?;
                let anchor = get_ld_anchor(ctx, 0, slot, attack)?;

                second =
                    own_x.wrapping_sub(get_ld_span(ctx, 0, slot, attack)?.wrapping_add(anchor));
            }
        }

        if !is_touchable(ctx, other, target, slot)? {
            return Ok(false);
        }

        let target_x = ctx.i32_at(AppContext::entity_field(other, target, Entity::POS_X))?;

        if target == 0 {
            if faction == 0 {
                return Ok(target_x >= second);
            }

            return Ok(target_x <= second);
        }

        if faction == 0 {
            if get_entity_base_idx(ctx)? == target {
                return Ok(target_x >= second);
            }

            if target_x < second {
                return Ok(false);
            }

            return Ok(target_x <= first);
        }

        if target_x < first {
            return Ok(false);
        }

        return Ok(target_x <= second);
    }

    let near;
    let far;

    if faction != 0 {
        near = ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?;
        far = ctx
            .i32_at(AppContext::entity_field(1, slot, Entity::STANDING_RANGE))?
            .wrapping_add(near);
    } else {
        far = ctx.i32_at(AppContext::entity_field(0, slot, Entity::POS_X))?;
        near = far.wrapping_sub(ctx.i32_at(AppContext::entity_field(
            0,
            slot,
            Entity::STANDING_RANGE,
        ))?);
    }

    if slot_occupied(ctx, other, target)? == 0 {
        return Ok(false);
    }

    if !is_touchable(ctx, other, target, slot)? {
        return Ok(false);
    }

    let edge;

    if faction != 0 {
        let target_x = ctx.i32_at(AppContext::entity_field(0, target, Entity::POS_X))?;
        let back = ctx
            .i32_at(AppContext::entity_field(0, target, Entity::HITBOX_WIDTH))?
            .wrapping_add(target_x);

        edge = target_x.wrapping_add(ctx.i32_at(AppContext::entity_field(
            0,
            target,
            Entity::HITBOX_POS,
        ))?);

        if near > back {
            return Ok(false);
        }
    } else {
        let target_x = ctx.i32_at(AppContext::entity_field(1, target, Entity::POS_X))?;
        let back = target_x.wrapping_sub(ctx.i32_at(AppContext::entity_field(
            1,
            target,
            Entity::HITBOX_POS,
        ))?);

        edge = target_x.wrapping_sub(ctx.i32_at(AppContext::entity_field(
            1,
            target,
            Entity::HITBOX_WIDTH,
        ))?);

        if near > back {
            return Ok(false);
        }
    }

    Ok(far >= edge)
}
