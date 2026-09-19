use crate::Fault;

use super::AppContext;

pub fn request_save_data(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "request_save_data",
        })?
        .request_save();

    Ok(())
}
