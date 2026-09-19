use crate::Fault;

use super::AppContext;

pub fn save_battle_snapshot(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "save_battle_snapshot",
        })?
        .save_battle_snapshot();

    Ok(())
}
