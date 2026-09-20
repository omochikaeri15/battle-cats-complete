use crate::Fault;

use super::AppContext;

pub fn pack_entry_text(ctx: &mut AppContext, name: &[u8]) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .assets()
        .ok_or(Fault::host_missing())?
        .pack_entry(name))
}
