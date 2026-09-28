use crate::Fault;

use super::AppContext;

pub fn load_battle_snapshot(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .load_battle_snapshot();

    Ok(())
}
