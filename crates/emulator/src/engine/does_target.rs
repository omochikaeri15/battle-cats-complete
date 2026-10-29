use crate::Fault;

use super::{AppContext, Entity, is_aku, is_relic, read_flag};

pub fn does_target(ctx: &AppContext, faction: i32, slot: i32, target: i32) -> Result<bool, Fault> {
    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::CURSE_TIMER))? > 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::CURSE_LENGTH,
        ))? > 0
    {
        return Ok(false);
    }

    let other = 1i32.wrapping_sub(faction);

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TARGETS_RED))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(other, target, Entity::IS_RED))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::TRAIT_FLOATING,
        ))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(
            other,
            target,
            Entity::TRAIT_FLOATING,
        ))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_DARK))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(other, target, Entity::TRAIT_DARK))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_METAL))? != 0
    {
        let metal = if read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0 {
            AppContext::entity_field(other, target, Entity::TRAIT_METAL)
        } else {
            AppContext::entity_field(other, target, Entity::METAL_CAT)
        };

        if ctx.i32_at(metal)? != 0 {
            return Ok(true);
        }
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0
        || ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::TRAIT_TRAITLESS,
        ))? != 0
    {
        if read_flag(ctx, AppContext::faction_flags(other))? & 1 != 0 {
            return Ok(true);
        }

        if ctx.i32_at(AppContext::entity_field(
            other,
            target,
            Entity::TRAIT_TRAITLESS,
        ))? != 0
        {
            return Ok(true);
        }
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_ANGEL))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(other, target, Entity::TRAIT_ANGEL))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_ALIEN))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(other, target, Entity::TRAIT_ALIEN))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::TRAIT_ZOMBIE,
        ))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(
            other,
            target,
            Entity::TRAIT_ZOMBIE,
        ))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_WITCH))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(other, target, Entity::TRAIT_WITCH))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::TRAIT_EVA_ANGEL,
        ))? != 0
        && read_flag(ctx, AppContext::faction_flags(other))? & 1 == 0
        && ctx.i32_at(AppContext::entity_field(
            other,
            target,
            Entity::TRAIT_EVA_ANGEL,
        ))? != 0
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
        && ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_RELIC))? != 0
        && is_relic(ctx, other, target)?
    {
        return Ok(true);
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(false);
    }

    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::TRAIT_AKU))? == 0 {
        return Ok(false);
    }

    is_aku(ctx, other, target)
}
