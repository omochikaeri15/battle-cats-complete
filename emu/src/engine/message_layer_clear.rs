use super::AppContext;

pub fn message_layer_clear(ctx: &mut AppContext, layer: i32) {
    ctx.text_blocks.remove(&layer);
}
