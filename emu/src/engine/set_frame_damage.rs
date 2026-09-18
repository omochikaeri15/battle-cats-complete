use crate::Fault;

use super::AppContext;

pub fn set_frame_damage(ctx: &mut AppContext, team: i32, slot: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83944), value)?;

    let took_damage = value != 0;

    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83b80), took_damage as i32)
}
