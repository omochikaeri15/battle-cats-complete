use crate::Fault;

use super::{ATTACK_LD_FLAG_STORE_FIELDS, AppContext};

pub fn set_ld_flag(
    ctx: &mut AppContext,
    faction: i32,
    slot: i32,
    attack: i32,
    value: u8,
) -> Result<(), Fault> {
    if attack == 0 {
        return Ok(());
    }

    let field = *ATTACK_LD_FLAG_STORE_FIELDS
        .get((attack as isize).wrapping_sub(1) as usize)
        .ok_or(Fault::IndexOutOfRange {
            site: "set_ld_flag",
            index: attack as i64,
            limit: 3,
        })?;

    ctx.set_i32_at(
        AppContext::entity_field(faction, slot, 0).wrapping_add((field as usize).wrapping_mul(4)),
        value as i32,
    )
}
