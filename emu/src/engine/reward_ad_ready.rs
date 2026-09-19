use crate::Fault;

use super::AppContext;

pub fn reward_ad_ready(ctx: &mut AppContext, kind: i32, flag: i32) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::HostMissing {
            site: "reward_ad_ready",
        })?
        .reward_ad_ready(kind, flag))
}
