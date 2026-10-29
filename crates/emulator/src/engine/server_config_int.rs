use crate::Fault;

use super::AppContext;

pub fn server_config_int(
    ctx: &mut AppContext,
    key: &[u8],
    min: i32,
    max: i32,
) -> Result<i32, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .config_int(key, min, max))
}
