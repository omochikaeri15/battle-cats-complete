use crate::Fault;

use super::AppContext;

pub fn labyrinth_unit_count(ctx: &mut AppContext, rarity: i32) -> Result<i32, Fault> {
    Ok(ctx
        .meta()
        .ok_or(Fault::HostMissing {
            site: "labyrinth_unit_count",
        })?
        .labyrinth_unit_count(rarity))
}
