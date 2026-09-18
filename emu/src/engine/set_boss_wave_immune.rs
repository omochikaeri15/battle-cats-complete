use crate::Fault;

use super::{AppContext, Entity};

pub fn set_boss_wave_immune(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::BOSS_WAVE_IMMUNE), value)
}
