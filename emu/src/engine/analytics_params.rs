use crate::Fault;

use super::{AppContext, FormatArg};

pub fn analytics_params(
    ctx: &mut AppContext,
    event: i32,
    value: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "analytics_params",
        })?
        .analytics_params(event, value, params);

    Ok(())
}
