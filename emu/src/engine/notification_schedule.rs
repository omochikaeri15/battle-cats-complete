use crate::Fault;

use super::AppContext;

pub fn notification_schedule(ctx: &mut AppContext, flag: u8, kind: i32) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::HostMissing {
            site: "notification_schedule",
        })?
        .notification_schedule(flag, kind);

    Ok(())
}
