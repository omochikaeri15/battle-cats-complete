use crate::Fault;

use super::AppContext;

pub fn message_layer_clear(ctx: &mut AppContext, layer: i32) -> Result<(), Fault> {
    ctx.ui()
        .ok_or(Fault::host_missing())?
        .message_clear(layer);

    Ok(())
}
