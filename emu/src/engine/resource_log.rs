use crate::Fault;

use super::AppContext;

pub fn resource_log(
    ctx: &mut AppContext,
    action: &[u8],
    item: &[u8],
    amount: i32,
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "resource_log",
        })?
        .resource_log(action, item, amount);

    Ok(())
}
