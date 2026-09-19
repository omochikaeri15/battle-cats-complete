use crate::Fault;

use super::AppContext;

pub fn breadcrumb_with(
    ctx: &mut AppContext,
    id: i32,
    params: &[(&[u8], &[u8])],
) -> Result<(), Fault> {
    if id as u32 > 0x57 {
        return Ok(());
    }

    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "breadcrumb_with",
        })?
        .breadcrumb_with(id, params);

    Ok(())
}
