use crate::Fault;

use super::AppContext;

pub fn shop_offer_start(ctx: &mut AppContext, map: i32) -> Result<(), Fault> {
    ctx.meta().ok_or(Fault::HostMissing { site: "shop_offer_start" })?.shop_offer_start(map);

    Ok(())
}
