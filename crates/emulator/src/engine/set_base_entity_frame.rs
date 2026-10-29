use crate::Fault;

use super::{AppContext, set_entity_frame};

pub fn set_base_entity_frame(ctx: &mut AppContext, faction: i32, frame: i32) -> Result<(), Fault> {
    set_entity_frame(ctx, faction, 0, frame)
}
