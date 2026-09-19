use crate::Fault;

use super::AppContext;

pub fn jni_share_image(
    ctx: &mut AppContext,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(), Fault> {
    ctx.platform()
        .ok_or(Fault::HostMissing {
            site: "jni_share_image",
        })?
        .share_image(x, y, width, height);

    Ok(())
}
