use crate::Fault;

use super::{read_flag, AppContext};

pub fn set_colossus_orb_pcts(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    attack_pct: i32,
    defense_pct: i32,
) -> Result<(), Fault> {
    let faction_index = faction as usize;

    if read_flag(ctx, faction_index.wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83c8c), attack_pct)?;
    ctx.set_i32_at(AppContext::entity_field(faction, slot, 0x83c90), defense_pct)
}
