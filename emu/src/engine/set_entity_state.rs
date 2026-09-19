use crate::Fault;

use super::{AppContext, Entity, call_rng, play_sound, read_flag, sound_manager};

pub fn set_entity_state(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    state: i32,
) -> Result<(), Fault> {
    let stored;

    if state == 0 {
        if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SPEED))? == 0 {
            stored = 1;
        } else if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
            stored = 0;
        } else {
            let revive_timer = ctx.i32_at(AppContext::entity_field(
                faction,
                slot,
                Entity::REVIVE_TIMER,
            ))?;

            stored = if revive_timer <= 0 { 0 } else { 0xe };
        }
    } else if state != 4 {
        stored = state;
    } else if ctx.i32_at(AppContext::entity_field(
        faction,
        slot,
        Entity::DEATH_SURGE_CHANCE,
    ))? == 0
    {
        stored = 4;
    } else {
        let roll = call_rng(ctx, 0x64);

        if ctx.i32_at(AppContext::entity_field(
            faction,
            slot,
            Entity::DEATH_SURGE_CHANCE,
        ))? <= roll
        {
            stored = 4;
        } else {
            play_sound(sound_manager(ctx)?, 0x8f, None);
            stored = 0x15;
        }
    }

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, Entity::STATE),
        stored,
    )
}
