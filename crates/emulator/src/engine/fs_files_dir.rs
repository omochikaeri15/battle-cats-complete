use crate::Fault;

use super::AppContext;

pub fn fs_files_dir(ctx: &mut AppContext) -> Result<Vec<u8>, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.files_dir())
}
