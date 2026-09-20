use crate::Fault;

use super::AppContext;

pub fn has_inquiry_code(ctx: &mut AppContext) -> Result<bool, Fault> {
    Ok(ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .has_inquiry_code())
}
