use crate::Fault;

use super::AppContext;

pub fn has_castle_enemy(ctx: &AppContext) -> Result<bool, Fault> {
    Ok(ctx.i32_at(AppContext::CASTLE_ENEMY_ROW)? > 0)
}
