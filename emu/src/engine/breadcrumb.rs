use crate::Fault;

use super::AppContext;

pub fn breadcrumb(ctx: &mut AppContext, id: i32) -> Result<(), Fault> {
    if id as u32 > 0x57 {
        return Ok(());
    }

    ctx.meta()
        .ok_or(Fault::host_missing())?
        .breadcrumb(id);

    Ok(())
}
