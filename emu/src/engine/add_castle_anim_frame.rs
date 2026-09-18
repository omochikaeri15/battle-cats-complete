use crate::Fault;

use super::{AppContext, Base};

pub fn add_castle_anim_frame(ctx: &mut AppContext, faction: i32, delta: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::entity_field(faction, 0, Base::CASTLE_ANIM_FRAME), ctx.i32_at(AppContext::entity_field(faction, 0, Base::CASTLE_ANIM_FRAME))?.wrapping_add(delta))
}
