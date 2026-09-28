use crate::Fault;

use super::AppContext;

pub fn cat_food_shop_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::host_missing())?.cat_food_shop_draw();

    Ok(())
}
