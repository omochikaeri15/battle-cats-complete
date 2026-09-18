use crate::Fault;

use super::{set_entity_frame, AppContext};

pub fn set_base_entity_frame(ctx: &mut AppContext, faction: i32, frame: i32) -> Result<(), Fault> {
    set_entity_frame(ctx, faction, 0, frame)
}
