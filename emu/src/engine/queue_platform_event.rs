use crate::Fault;

use super::AppContext;

pub fn queue_platform_event(
    ctx: &mut AppContext,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::host_missing())?
        .share_image(x, y, width, height);

    Ok(())
}
