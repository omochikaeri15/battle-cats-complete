use crate::Fault;

use super::AppContext;

pub fn analytics_named(ctx: &mut AppContext, code: i32, name: &[u8], detail: &[u8]) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::HostMissing { site: "analytics_named" })?.analytics_named(code, name, detail);

    Ok(())
}
