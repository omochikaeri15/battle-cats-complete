use crate::Fault;

use super::AppContext;

pub fn fs_file_exists(ctx: &mut AppContext, path: &[u8]) -> Result<bool, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.file_exists(path))
}
