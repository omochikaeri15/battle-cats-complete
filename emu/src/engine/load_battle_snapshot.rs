use crate::Fault;

use super::AppContext;

pub fn load_battle_snapshot(ctx: &mut AppContext, mode: i32) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::HostMissing { site: "load_battle_snapshot" })?.load_battle_snapshot(mode);

    Ok(())
}
