use crate::Fault;

use super::AppContext;

pub fn fs_remove_file(ctx: &mut AppContext, path: &[u8]) -> Result<bool, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.remove_file(path))
}
