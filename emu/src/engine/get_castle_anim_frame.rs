use crate::Fault;

use super::{AppContext, Base};

pub fn get_castle_anim_frame(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(
        faction,
        0,
        Base::CASTLE_ANIM_FRAME,
    ))
}
