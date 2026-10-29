use crate::Fault;

use super::{AppContext, TextBlock, message_layer_layout};

pub fn message_layer_set(
    ctx: &mut AppContext,
    layer: i32,
    text: &[u8],
    size: i32,
    width: i32,
) -> Result<(), Fault> {
    if ctx.text_blocks.contains_key(&layer) {
        return Ok(());
    }

    let mut block = TextBlock::default();
    let laid = message_layer_layout(ctx, &mut block, text, size, width);

    ctx.text_blocks.insert(layer, block);

    laid
}
