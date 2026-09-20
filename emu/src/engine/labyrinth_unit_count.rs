use crate::Fault;

use super::AppContext;

pub fn labyrinth_unit_count(ctx: &mut AppContext, rarity: i32) -> Result<i32, Fault> {
    Ok(ctx
        .meta()
        .ok_or(Fault::host_missing())?
        .labyrinth_unit_count(rarity))
}
