use crate::Fault;

use super::{AppContext, get_talent_icon_state, stat_colossus_slayer};

pub fn ability_icon_is_base(
    ctx: &mut AppContext,
    icon: i32,
    unit_id: i32,
    form: i32,
) -> Result<bool, Fault> {
    let abil = match icon {
        0x20 => 0x12,
        0x21 => 0x13,
        0x22 => 0x14,
        0x23 => 0x15,
        0x24 => 0x16,
        0x27 => 0x1a,
        0x2a => 0x1e,
        0x3f => 0x34,
        0x40 => 0x36,
        0x43 => return Ok(!stat_colossus_slayer(ctx, 0, unit_id, form)?),
        _ => return Ok(false),
    };

    Ok(get_talent_icon_state(ctx, 0, unit_id, form, abil)? == 0)
}
