use crate::Fault;

use super::AppContext;

pub fn event_schedule_active(ctx: &mut AppContext, now: f64) -> Result<Vec<Vec<i32>>, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::HostMissing {
            site: "event_schedule_active",
        })?
        .event_schedule_active(now))
}
