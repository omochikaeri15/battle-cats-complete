use crate::Fault;

use super::{AppContext, Base};

pub fn set_castle_anim_frame(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    ctx.set_i32_at(
        AppContext::entity_field(faction, 0, Base::CASTLE_ANIM_FRAME),
        value,
    )
}
