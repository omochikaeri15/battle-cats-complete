use crate::Fault;

use super::AppContext;

pub fn feature_enabled(ctx: &mut AppContext, feature: i32) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .feature_enabled(feature))
}
