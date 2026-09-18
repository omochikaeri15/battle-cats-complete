use crate::Fault;

use super::{read_flag, AppContext};

pub fn set_metal(ctx: &mut AppContext, team: i32, slot: i32, value: i32) -> Result<(), Fault> {
    let team_index = team as usize;

    if read_flag(ctx, team_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83a20), value)
    } else {
        ctx.set_i32_at(AppContext::entity_field(team, slot, 0x839a4), value)
    }
}
