use crate::Fault;

use super::AppContext;

pub fn item_pass_active(ctx: &mut AppContext, item: i32) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .item_pass_active(item))
}
