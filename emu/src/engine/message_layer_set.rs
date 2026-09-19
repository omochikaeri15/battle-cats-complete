use crate::Fault;

use super::AppContext;

pub fn message_layer_set(
    ctx: &mut AppContext,
    layer: i32,
    text: &[u8],
    size: i32,
    width: i32,
) -> Result<(), Fault> {
    ctx.ui()
        .ok_or(Fault::HostMissing {
            site: "message_layer_set",
        })?
        .message_set(layer, text, size, width);

    Ok(())
}
