use crate::Fault;

use super::{AppContext, Pinch};

pub fn pinch_is_active(ctx: &AppContext, pinch: usize) -> Result<u8, Fault> {
    ctx.u8_at(pinch.wrapping_add(Pinch::ACTIVE))
}
