use crate::Fault;

use super::AppContext;

pub fn std_chrono_system_clock_now(ctx: &mut AppContext) -> Result<i64, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .system_clock_now())
}
