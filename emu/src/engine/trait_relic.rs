use crate::Fault;

use super::{read_flag, talent_targets_trait, AppContext};

pub fn trait_relic(ctx: &mut AppContext, faction: i32, unit_id: i32, form: i32, allow_talent: u8) -> Result<bool, Fault> {
    if read_flag(ctx, (faction as usize).wrapping_mul(0x1f0).wrapping_add(0x2648))? & 1 == 0 {
        return Ok(ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x1c4 + 0x233228) as usize)? != 0);
    }

    if ctx.i32_at(((unit_id.wrapping_add(2) as i64) * 0x760 + (form as i64) * 0x1d8 + 0x9e6a0) as usize)? != 0 {
        return Ok(true);
    }

    if allow_talent == 0 {
        return Ok(false);
    }

    talent_targets_trait(ctx, faction, unit_id, form, 0x80)
}
