use crate::Fault;

use super::{read_flag, AppContext};

pub fn set_trait_red(ctx: &mut AppContext, faction: i32, slot: i32, value: i32) -> Result<(), Fault> {
    let faction_index = faction as usize;

    if read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 != 0 {
        ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83938), value)
    } else {
        ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x8393c), value)
    }
}
