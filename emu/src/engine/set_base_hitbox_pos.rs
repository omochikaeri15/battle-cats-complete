use crate::Fault;

use super::{set_hitbox_pos, AppContext};

pub fn set_base_hitbox_pos(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_hitbox_pos(ctx, faction, 0, value)
}
