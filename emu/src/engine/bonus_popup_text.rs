use crate::Fault;

use super::AppContext;

pub fn bonus_popup_text(ctx: &mut AppContext) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .meta()
        .ok_or(Fault::host_missing())?
        .bonus_popup_text())
}
