use crate::Fault;

use super::{AppContext, Base};

pub fn get_cannon_wall_lifetime(ctx: &AppContext, faction: i32) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::entity_field(faction, 0, Base::CANNON_WALL_LIFETIME))
}
