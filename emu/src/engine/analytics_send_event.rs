use crate::Fault;

use super::{AppContext, FormatArg};

pub fn analytics_send_event(
    ctx: &mut AppContext,
    at: i32,
    name: &[u8],
    amount: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .analytics_send_event(at, name, amount, params);

    Ok(())
}
