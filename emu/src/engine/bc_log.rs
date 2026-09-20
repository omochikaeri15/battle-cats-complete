use crate::Fault;

use super::AppContext;

pub fn bc_log(ctx: &mut AppContext, name: &[u8]) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::host_missing())?
        .bc_log(name);

    Ok(())
}
