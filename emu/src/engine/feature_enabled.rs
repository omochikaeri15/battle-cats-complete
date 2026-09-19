use crate::Fault;

use super::AppContext;

pub fn feature_enabled(ctx: &mut AppContext, feature: i32) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::HostMissing {
            site: "feature_enabled",
        })?
        .feature_enabled(feature))
}
