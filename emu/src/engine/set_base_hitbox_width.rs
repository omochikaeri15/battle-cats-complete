use crate::Fault;

use super::{set_hitbox_width, AppContext};

pub fn set_base_hitbox_width(ctx: &mut AppContext, faction: i32, value: i32) -> Result<(), Fault> {
    set_hitbox_width(ctx, faction, 0, value)
}
