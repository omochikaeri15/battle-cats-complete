use crate::Fault;

use super::AppContext;

pub fn get_castle_enemy_row(ctx: &AppContext) -> Result<i32, Fault> {
    ctx.i32_at(AppContext::CASTLE_ENEMY_ROW)
}
