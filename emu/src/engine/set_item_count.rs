use crate::Fault;

use super::AppContext;

pub fn set_item_count(ctx: &mut AppContext, item: i32, count: i32, flag: u8) -> Result<(), Fault> {
    ctx.meta()
        .ok_or(Fault::HostMissing {
            site: "set_item_count",
        })?
        .set_item_count(item, count, flag);

    Ok(())
}
