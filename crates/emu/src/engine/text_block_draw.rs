use std::collections::BTreeMap;

use crate::Fault;

use super::{DrawSink, TextBlock, text_block_render};

pub fn text_block_draw(
    sink: &mut Option<Box<dyn DrawSink>>,
    blocks: &mut BTreeMap<i32, TextBlock>,
    key: i32,
    x: i32,
    y: i32,
    align: i32,
    scale: f32,
) -> Result<(), Fault> {
    text_block_render(sink, blocks.entry(key).or_default(), x, y, align, scale)
}
