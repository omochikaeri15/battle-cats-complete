use crate::Fault;

use super::AppContext;

pub fn drop_popup_text(
    ctx: &mut AppContext,
    item: i32,
    first: u8,
    amount: i32,
) -> Result<Vec<u8>, Fault> {
    Ok(ctx
        .meta()
        .ok_or(Fault::HostMissing {
            site: "drop_popup_text",
        })?
        .drop_popup_text(item, first, amount))
}
