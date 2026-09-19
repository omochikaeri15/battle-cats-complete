use crate::Fault;

use super::{get_map_type, AppContext};

pub fn is_space_map(ctx: &mut AppContext) -> Result<bool, Fault> {
    let index = get_map_type(ctx, 0)?.wrapping_add(0x19) as u32;

    Ok(index < 0x13 && (0x40c01u32 >> (index & 0x1f)) & 1 != 0)
}
