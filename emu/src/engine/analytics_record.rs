use crate::Fault;

use super::{AppContext, FormatArg};

pub fn analytics_record(
    ctx: &mut AppContext,
    code: i32,
    value: i32,
    flag: i32,
    params: &[(&[u8], FormatArg<'_>)],
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .analytics_record(code, value, flag, params);

    Ok(())
}
