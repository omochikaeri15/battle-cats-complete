use crate::Fault;

use super::AppContext;

pub fn inquiry_button_draw(ctx: &mut AppContext) -> Result<(), Fault> {
    ctx.ui().ok_or(Fault::host_missing())?.inquiry_button_draw();

    Ok(())
}
