use crate::Fault;

use super::AppContext;

pub fn asset_stream_open(ctx: &mut AppContext, name: &[u8]) -> Result<bool, Fault> {
    Ok(ctx
        .assets()
        .ok_or(Fault::HostMissing {
            site: "asset_stream_open",
        })?
        .open(name, 1, 0)
        .is_some())
}
