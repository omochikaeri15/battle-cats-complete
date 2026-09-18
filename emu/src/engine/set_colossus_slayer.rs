use crate::Fault;

use super::{read_flag, AppContext};

pub fn set_colossus_slayer(ctx: &mut AppContext, team: i32, slot: i32, value: u8) -> Result<(), Fault> {
    let team_index = team as usize;

    if read_flag(ctx, team_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::entity_field(team, slot, 0x83c0c), value as i32)
}
