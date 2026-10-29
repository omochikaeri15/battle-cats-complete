use crate::Fault;

use super::AppContext;

pub fn analytics_named(
    ctx: &mut AppContext,
    code: i32,
    name: &[u8],
    detail: &[u8],
) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .analytics_named(code, name, detail);

    Ok(())
}
