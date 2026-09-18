use crate::fault::Fault;

use super::{call_rng, play_sound, read_flag, sound_manager, AppContext};

pub fn set_entity_state(ctx: &mut AppContext, team: i32, slot: i32, state: i32) -> Result<(), Fault> {
    let team_index = team as usize;
    let stored;

    if state == 0 {
        if ctx.i32_at(AppContext::entity_field(team, slot, 0x83920))? == 0 {
            stored = 1;
        } else if read_flag(ctx, team_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
            stored = 0;
        } else {
            let revive_timer = ctx.i32_at(AppContext::entity_field(team, slot, 0x83c2c))?;

            stored = if revive_timer <= 0 { 0 } else { 0xe };
        }
    } else if state != 4 {
        stored = state;
    } else if ctx.i32_at(AppContext::entity_field(team, slot, 0x83be0))? == 0 {
        stored = 4;
    } else {
        let roll = call_rng(ctx, 0x64);

        if ctx.i32_at(AppContext::entity_field(team, slot, 0x83be0))? <= roll {
            stored = 4;
        } else {
            play_sound(sound_manager(ctx)?, 0x8f, None);
            stored = 0x15;
        }
    }

    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x838fc), stored)
}
